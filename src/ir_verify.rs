//! Structural SSA/CFG and representation-aware value validation before LLVM lowering.
use crate::ir::*;
use std::collections::{ HashMap, HashSet };
pub fn verify(module: &ModuleIr) -> Result<(), String> {
    let functions: HashMap<_, _> = module.functions
        .iter()
        .map(|f| (f.name.as_str(), f))
        .collect();
    if functions.len() != module.functions.len() {
        return Err("Duplicate function names".into());
    }
    for f in &module.functions {
        let mut parameters = HashSet::new();
        for (reg, ty) in &f.args {
            if !parameters.insert(*reg) || module.globals.contains_key(reg) {
                return Err(format!("{}: Duplicate parameter/global register {reg}", f.name));
            }
            if *ty == IrType::Void {
                return Err(format!("{}: Void parameter", f.name));
            }
        }
        if f.blocks.is_empty() {
            continue;
        }
        let fail = |msg: String| format!("{}: {}", f.name, msg);
        let ids: HashSet<_> = f.blocks
            .iter()
            .map(|b| b.id)
            .collect();
        if ids.len() != f.blocks.len() {
            return Err(fail("Duplicate block IDs".into()));
        }
        let mut definitions = HashMap::new();
        let mut available: HashSet<_> = module.globals.keys().copied().collect();
        available.extend(f.args.iter().map(|(r, _)| *r));
        let mut predecessors = HashMap::<BlockId, Vec<BlockId>>::new();
        let mut successors = HashMap::<BlockId, Vec<BlockId>>::new();
        for b in &f.blocks {
            for (n, i) in b.instructions.iter().enumerate() {
                let terminal = matches!(
                    i,
                    Instruction::Br { .. } |
                        Instruction::CondBr { .. } |
                        Instruction::Ret { .. } |
                        Instruction::Unreachable
                );
                if terminal != (n + 1 == b.instructions.len()) {
                    return Err(fail(format!("Malformed terminator in {:?}", b.id)));
                }
                if let Some(d) = i.destination() {
                    if
                        parameters.contains(&d) ||
                        (module.globals.contains_key(&d) &&
                            !(f.name == "__global_init" && matches!(i, Instruction::Alloca { .. })))
                    {
                        return Err(
                            fail(format!("Definition reuses parameter/global register {d}"))
                        );
                    }
                    if definitions.insert(d, (b.id, n)).is_some() {
                        return Err(fail(format!("Duplicate definition {d}")));
                    }
                }
                if let Instruction::Call { func_name, args, .. } = i {
                    let callee = functions
                        .get(func_name.as_str())
                        .ok_or_else(|| fail(format!("Unknown function {func_name}")))?;
                    if
                        args.len() < callee.args.len() ||
                        (!callee.is_variadic && args.len() != callee.args.len())
                    {
                        return Err(fail(format!("Wrong argument count for {func_name}")));
                    }
                }
                if let Instruction::Ret { value } = i {
                    if value.is_none() != (f.ret_type == IrType::Void) {
                        return Err(fail("Return value/signature mismatch".into()));
                    }
                }
            }
            if b.instructions.is_empty() {
                return Err(fail("Empty unterminated block".into()));
            }
            let targets = match b.instructions.last().unwrap() {
                Instruction::Br { target } => vec![*target],

                Instruction::CondBr { if_true, if_false, .. } => vec![*if_true, *if_false],
                _ => vec![],
            };
            for t in &targets {
                if !ids.contains(t) {
                    return Err(fail(format!("Missing branch target {t:?}")));
                }
                predecessors.entry(*t).or_default().push(b.id);
            }
            successors.insert(b.id, targets);
        }
        let entry = f.blocks[0].id;
        let mut reachable = HashSet::new();
        let mut postorder = Vec::new();
        let mut work = vec![(entry, false)];
        while let Some((b, expanded)) = work.pop() {
            if expanded {
                postorder.push(b);
            } else if reachable.insert(b) {
                work.push((b, true));
                work.extend(
                    successors[&b]
                        .iter()
                        .rev()
                        .map(|b| (*b, false))
                );
            }
        }
        postorder.reverse();
        let mut dom: HashMap<_, HashSet<_>> = reachable
            .iter()
            .map(|b| { (*b, if *b == entry { HashSet::from([entry]) } else { reachable.clone() }) })
            .collect();
        loop {
            let mut changed = false;
            // Reverse postorder converges rapidly for normal structured CFGs.
            // The same fixed point and dominance checks apply even to loops
            // and block vectors deliberately shuffled by compiler tests.
            for b in postorder.iter().filter(|b| **b != entry) {
                let mut incoming = predecessors
                    .get(b)
                    .into_iter()
                    .flatten()
                    .filter(|p| reachable.contains(p));
                let mut d = incoming
                    .next()
                    .map(|p| dom[p].clone())
                    .unwrap_or_default();
                for p in incoming {
                    d.retain(|x| dom[p].contains(x));
                }
                d.insert(*b);
                if dom[b] != d {
                    dom.insert(*b, d);
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        for b in &f.blocks {
            for (n, i) in b.instructions.iter().enumerate() {
                for r in i.operands() {
                    if available.contains(&r) {
                        continue;
                    }
                    let (db, dn) = definitions
                        .get(&r)
                        .ok_or_else(|| fail(format!("Undefined register {r}")))?;
                    if
                        reachable.contains(&b.id) &&
                        ((*db == b.id && *dn >= n) || (*db != b.id && !dom[&b.id].contains(db)))
                    {
                        return Err(fail(format!("Definition of {r} does not dominate its use")));
                    }
                }
            }
        }
        verify_value_types(module, f, &functions)?;
    }
    for (name, entries) in &module.vtables {
        for entry in entries.iter().filter(|name| !name.is_empty()) {
            if !functions.contains_key(entry.as_str()) {
                return Err(format!("{name}: Unknown vtable function {entry}"));
            }
        }
    }
    Ok(())
}

// IR transfers normalize integer widths at stores, calls and returns. Scalar
// casts record interpretation, while Array is a managed data handle (length 0
// denotes a slice). These checks preserve that contract; they do not impose
// LLVM's physical register widths or an ownership/lifetime analysis.
fn integer(ty: &IrType) -> bool {
    matches!(
        ty,
        IrType::U8 |
            IrType::U16 |
            IrType::U32 |
            IrType::U64 |
            IrType::I8 |
            IrType::I16 |
            IrType::I32 |
            IrType::I64 |
            IrType::Bool
    )
}
fn floating(ty: &IrType) -> bool {
    matches!(ty, IrType::F16 | IrType::F32 | IrType::F64)
}
fn address(ty: &IrType) -> bool {
    matches!(
        ty,
        IrType::Ptr(_) | IrType::Array(..) | IrType::FatPtr | IrType::Closure(..) | IrType::Any
    )
}
// Storage elements and callable declarations retain their widths. Scalar
// register normalization is a separate rule applied only at value transfers.
fn same_storage(actual: &IrType, expected: &IrType) -> bool {
    if actual == expected || matches!(actual, IrType::Any) || matches!(expected, IrType::Any) {
        return true;
    }
    match (actual, expected) {
        (IrType::Ptr(a), IrType::Ptr(b)) => {
            matches!(a.as_ref(), IrType::Any | IrType::Void) ||
                matches!(b.as_ref(), IrType::Any | IrType::Void) ||
                (matches!(a.as_ref(), IrType::I8 | IrType::U8) &&
                    matches!(b.as_ref(), IrType::I8 | IrType::U8)) ||
                same_storage(a, b)
        }
        (IrType::Array(_, a), IrType::Array(_, b)) => same_storage(a, b),
        // A variant payload allocation can be used as its nominal enum handle.
        (IrType::Struct(a, _), IrType::Struct(b, _)) => a.split('#').next() == b.split('#').next(),
        (IrType::Closure(a, r), IrType::Closure(b, s)) => {
            a.len() == b.len() &&
                a
                    .iter()
                    .zip(b)
                    .all(|(a, b)| same_storage(a, b)) &&
                same_storage(r, s)
        }
        _ => false,
    }
}
fn compatible(actual: &IrType, expected: &IrType) -> bool {
    // Runtime byte-buffer/destructor interfaces erase handle pointee types.
    if
        matches!(expected,IrType::Ptr(t) if matches!(t.as_ref(),IrType::U8|IrType::I8|IrType::Void|IrType::Any)) &&
        address(actual)
    {
        return true;
    }
    (integer(actual) && integer(expected)) || same_storage(actual, expected)
}

fn verify_value_types(
    module: &ModuleIr,
    f: &FunctionIr,
    functions: &HashMap<&str, &FunctionIr>
) -> Result<(), String> {
    let fail = |msg: String| format!("{}: {}", f.name, msg);
    let instructions = f.blocks
        .iter()
        .flat_map(|b| &b.instructions)
        .collect::<Vec<_>>();
    let constants = instructions
        .iter()
        .filter_map(|i| {
            match i {
                Instruction::ConstInt { dest, value } => Some((*dest, *value)),
                _ => None,
            }
        })
        .collect::<HashMap<_, _>>();
    let mut types = module.globals
        .iter()
        .map(|(r, t)| (*r, IrType::Ptr(Box::new(t.clone()))))
        .collect::<HashMap<_, _>>();
    types.extend(f.args.iter().cloned());
    // Resolve types independently of the block vector's listing order.
    loop {
        let mut changed = false;
        for i in &instructions {
            let Some(dest) = i.destination() else {
                continue;
            };
            if types.contains_key(&dest) {
                continue;
            }
            let ty = match i {
                Instruction::Alloca { ty, .. } => Some(IrType::Ptr(Box::new(ty.clone()))),
                Instruction::AllocArray { ty, .. } => Some(IrType::Array(0, Box::new(ty.clone()))),
                Instruction::AllocStruct { class_name, .. } =>
                    Some(IrType::Ptr(Box::new(IrType::Struct(class_name.clone(), vec![])))),
                Instruction::ConstInt { .. } => Some(IrType::I64),
                Instruction::SizeOf { .. } => Some(IrType::U64),
                Instruction::ConstFloat { ty, .. } | Instruction::Load { ty, .. } => {
                    Some(ty.clone())
                }
                Instruction::ConstBool { .. } => Some(IrType::Bool),
                Instruction::ConstString { .. } => Some(IrType::Ptr(Box::new(IrType::I8))),
                Instruction::Cast { target_ty, .. } => Some(target_ty.clone()),
                Instruction::Call { func_name, .. } =>
                    functions.get(func_name.as_str()).map(|f| f.ret_type.clone()),
                | Instruction::DynamicCall { ret_type, .. }
                | Instruction::CallClosure { ret_type, .. }
                | Instruction::IndirectCall { ret_type, .. } => Some(ret_type.clone()),
                Instruction::MakeFatPtr { .. } => Some(IrType::FatPtr),
                Instruction::LoadFnPtr { .. } => Some(IrType::Ptr(Box::new(IrType::Any))),
                Instruction::MakeClosure { fn_name, .. } => {
                    functions.get(fn_name.as_str()).map(|f| {
                        IrType::Closure(
                            f.args
                                .iter()
                                .skip(1)
                                .map(|(_, t)| t.clone())
                                .collect(),
                            Box::new(f.ret_type.clone())
                        )
                    })
                }
                | Instruction::CmpEq { .. }
                | Instruction::CmpNeq { .. }
                | Instruction::CmpLt { .. }
                | Instruction::CmpLe { .. }
                | Instruction::CmpGt { .. }
                | Instruction::CmpGe { .. } => Some(IrType::Bool),
                Instruction::Div { ty, .. } | Instruction::Mod { ty, .. } => Some(ty.clone()),
                | Instruction::Add { left, right, .. }
                | Instruction::Sub { left, right, .. }
                | Instruction::Mul { left, right, .. } => {
                    types
                        .get(left)
                        .zip(types.get(right))
                        .map(|(a, b)| {
                            if floating(a) {
                                a.clone()
                            } else if floating(b) {
                                b.clone()
                            } else {
                                IrType::I64
                            }
                        })
                }
                Instruction::BitAnd { .. } | Instruction::BitOr { .. } => Some(IrType::I64),
                Instruction::GetElementPtr { base_ty, indices, .. } => {
                    let element = if indices.len() == 2 {
                        if let IrType::Struct(name, fields) = base_ty {
                            if let Some(layout) = module.structs.get(name) {
                                if !fields.is_empty() && fields != layout {
                                    return Err(
                                        fail(format!("Conflicting field layout for {name}"))
                                    );
                                }
                            }
                            let fields = if fields.is_empty() {
                                module.structs.get(name).unwrap_or(fields)
                            } else {
                                fields
                            };
                            let index = constants
                                .get(&indices[1])
                                .ok_or_else(|| {
                                    fail("Struct field index must be constant".into())
                                })?;
                            fields
                                .get(usize::try_from(*index).unwrap_or(usize::MAX))
                                .cloned()
                                .ok_or_else(|| {
                                    fail(format!("Struct field index out of bounds for {name}"))
                                })?
                        } else {
                            return Err(fail("Multiple GEP indices require a struct layout".into()));
                        }
                    } else if indices.len() == 1 {
                        base_ty.clone()
                    } else {
                        return Err(
                            fail("GEP requires one element index or two field indices".into())
                        );
                    };
                    Some(IrType::Ptr(Box::new(element)))
                }
                _ => None,
            };
            if let Some(ty) = ty {
                types.insert(dest, ty);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    // Alloca and GEP establish explicit memory views. Raw payload handles may
    // also be read through separate header/tag views, so do not infer a whole
    // memory object solely from a pointer-valued register's apparent pointee.
    let mut memory = module.globals.clone();
    for i in &instructions {
        match i {
            Instruction::Alloca { dest, ty, .. } => {
                if let Some(previous) = memory.insert(*dest, ty.clone()) {
                    if previous != *ty {
                        return Err(fail("Global allocation type mismatch".into()));
                    }
                }
            }
            Instruction::GetElementPtr { dest, .. } => {
                if let Some(IrType::Ptr(ty)) = types.get(dest) {
                    memory.insert(*dest, ty.as_ref().clone());
                }
            }
            _ => {}
        }
    }
    let memory_type = |ptr: &VReg, access: &IrType| -> Result<(), String> {
        if let Some(stored) = memory.get(ptr) {
            if !same_storage(stored, access) {
                return Err(
                    fail(
                        format!(
                            "Memory access type mismatch at {ptr}: storage {stored:?}, access {access:?}"
                        )
                    )
                );
            }
        }
        Ok(())
    };
    let ty = |r: &VReg| {
        types.get(r).ok_or_else(|| fail(format!("Cannot determine type of {r}")))
    };
    let transfer = |r: &VReg, expected: &IrType| -> Result<(), String> {
        let actual = ty(r)?;
        if *actual == IrType::Void {
            return Err(fail(format!("Void result used as a value: {r}")));
        }
        // The builder explicitly uses integer zero to initialize any storage
        // representation, and to represent null handles/empty environments.
        if constants.get(r) == Some(&0) || compatible(actual, expected) {
            Ok(())
        } else {
            Err(fail(format!("Type mismatch for {r}: expected {expected:?}, found {actual:?}")))
        }
    };
    let pointer = |r: &VReg| -> Result<(), String> {
        if address(ty(r)?) || constants.get(r) == Some(&0) {
            Ok(())
        } else {
            Err(fail(format!("Expected address for {r}, found {:?}", ty(r)?)))
        }
    };
    let arguments = |args: &[VReg], expected: &[IrType]| -> Result<(), String> {
        if args.len() != expected.len() {
            return Err(fail("Indirect call argument count/signature mismatch".into()));
        }
        for (arg, expected) in args.iter().zip(expected) {
            if *expected == IrType::Void {
                return Err(fail("Void indirect-call parameter".into()));
            }
            transfer(arg, expected)?;
        }
        Ok(())
    };
    for i in &instructions {
        match i {
            Instruction::ConstFloat { ty, .. } if !floating(ty) => {
                return Err(fail("Floating constant requires a floating type".into()));
            }
            | Instruction::SizeOf { ty: IrType::Void, .. }
            | Instruction::Alloca { ty: IrType::Void, .. }
            | Instruction::AllocArray { ty: IrType::Void, .. }
            | Instruction::Load { ty: IrType::Void, .. }
            | Instruction::Store { ty: IrType::Void, .. } => {
                return Err(fail("Void has no storage representation".into()));
            }
            Instruction::AllocStruct { class_name, .. } if
                !module.structs.contains_key(class_name)
            => {
                return Err(fail(format!("Unknown allocation layout {class_name}")));
            }
            Instruction::Call { func_name, args, .. } => {
                let callee = functions[func_name.as_str()];
                for (arg, (_, expected)) in args.iter().zip(&callee.args) {
                    transfer(arg, expected)?;
                }
                for arg in args {
                    if *ty(arg)? == IrType::Void {
                        return Err(fail("Void call argument".into()));
                    }
                }
            }
            Instruction::Ret { value: Some(value) } => transfer(value, &f.ret_type)?,
            Instruction::Load { src_ptr, ty, .. } => {
                pointer(src_ptr)?;
                memory_type(src_ptr, ty)?;
            }
            Instruction::Store { ty, value, ptr } => {
                pointer(ptr)?;
                memory_type(ptr, ty)?;
                transfer(value, ty)?;
            }
            Instruction::GetElementPtr { base_ptr, indices, .. } => {
                pointer(base_ptr)?;
                for index in indices {
                    if !integer(ty(index)?) {
                        return Err(fail("GEP index must be integer".into()));
                    }
                }
            }
            Instruction::AllocArray { size, .. } => {
                if !integer(ty(size)?) {
                    return Err(fail("Array allocation size must be integer".into()));
                }
            }
            Instruction::CondBr { cond, .. } => {
                if !integer(ty(cond)?) {
                    return Err(
                        fail("Branch condition must be bool or a normalized integer".into())
                    );
                }
            }

            Instruction::MakeClosure { fn_name, env_ptr, .. } => {
                let target = functions
                    .get(fn_name.as_str())
                    .ok_or_else(|| fail(format!("Unknown closure function {fn_name}")))?;
                let Some((_, environment)) = target.args.first() else {
                    return Err(fail("Closure target requires an environment parameter".into()));
                };
                if !address(environment) {
                    return Err(fail("Closure environment parameter must be an address".into()));
                }
                pointer(env_ptr)?;
            }
            Instruction::LoadFnPtr { fn_name, .. } if !functions.contains_key(fn_name.as_str()) => {
                return Err(fail(format!("Unknown function value {fn_name}")));
            }
            Instruction::CallClosure { closure_ptr, args, arg_types, ret_type, .. } => {
                pointer(closure_ptr)?;
                arguments(args, arg_types)?;
                if
                    let IrType::Closure(ps, ret) = (match ty(closure_ptr)? {
                        s => s,
                    })
                {
                    if
                        ps.len() != arg_types.len() ||
                        !ps
                            .iter()
                            .zip(arg_types)
                            .all(|(a, b)| same_storage(a, b)) ||
                        !same_storage(ret, ret_type)
                    {
                        return Err(fail("Closure call signature mismatch".into()));
                    }
                }
            }
            Instruction::IndirectCall { fn_ptr, args, arg_types, .. } => {
                if !address(ty(fn_ptr)?) && !integer(ty(fn_ptr)?) {
                    return Err(fail("Indirect call requires a function address".into()));
                }
                arguments(args, arg_types)?;
            }
            Instruction::DynamicCall { fat_ptr, args, arg_types, receiver_mode, .. } => {
                transfer(
                    fat_ptr,
                    &(if
                        matches!(
                            receiver_mode,
                            crate::ir::ReceiverMode::Shared | crate::ir::ReceiverMode::Exclusive
                        )
                    {
                        IrType::Ptr(Box::new(IrType::FatPtr))
                    } else {
                        IrType::FatPtr
                    })
                )?;
                arguments(args, arg_types)?;
            }
            Instruction::MakeFatPtr { data_ptr, vtable_name, .. } => {
                pointer(data_ptr)?;
                if !module.vtables.contains_key(vtable_name) {
                    return Err(fail(format!("Unknown vtable {vtable_name}")));
                }
            }
            Instruction::Retain { ptr } | Instruction::Release { ptr } => pointer(ptr)?,
            | Instruction::Add { left, right, .. }
            | Instruction::Sub { left, right, .. }
            | Instruction::Mul { left, right, .. } => {
                let (a, b) = (ty(left)?, ty(right)?);
                if
                    !(integer(a) || floating(a)) ||
                    !(integer(b) || floating(b)) ||
                    (floating(a) && floating(b) && a != b)
                {
                    return Err(fail("Arithmetic operand type mismatch".into()));
                }
            }
            Instruction::BitAnd { left, right, .. } | Instruction::BitOr { left, right, .. } => {
                if !integer(ty(left)?) || !integer(ty(right)?) {
                    return Err(fail("Bitwise operands must be integers".into()));
                }
            }
            | Instruction::Div { left, right, ty: expected, .. }
            | Instruction::Mod { left, right, ty: expected, .. }
            | Instruction::CmpLt { left, right, ty: expected, .. }
            | Instruction::CmpLe { left, right, ty: expected, .. }
            | Instruction::CmpGt { left, right, ty: expected, .. }
            | Instruction::CmpGe { left, right, ty: expected, .. } => {
                if !integer(expected) && !floating(expected) {
                    return Err(fail("Numeric operation requires numeric type".into()));
                }
                transfer(left, expected)?;
                transfer(right, expected)?;
            }
            Instruction::CmpEq { left, right, .. } | Instruction::CmpNeq { left, right, .. } => {
                let (a, b) = (ty(left)?, ty(right)?);
                if
                    *a == IrType::Void ||
                    *b == IrType::Void ||
                    (!compatible(a, b) &&
                        constants.get(left) != Some(&0) &&
                        constants.get(right) != Some(&0))
                {
                    return Err(fail("Comparison operand type mismatch".into()));
                }
            }
            Instruction::Cast { value, target_ty, .. } => {
                if *ty(value)? == IrType::Void || *target_ty == IrType::Void {
                    return Err(fail("Cannot cast void value".into()));
                }
            }
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn module(instructions: Vec<Instruction>) -> ModuleIr {
        ModuleIr {
            resources: HashMap::new(),

            name: "test".into(),
            globals: HashMap::new(),
            structs: HashMap::new(),
            vtables: HashMap::new(),
            functions: vec![FunctionIr {
                name: "main".into(),
                args: vec![],
                ret_type: IrType::I64,
                is_variadic: false,
                blocks: vec![BasicBlock {
                    id: BlockId(0),
                    name: "entry".into(),
                    instructions,
                }],
            }],
        }
    }

    #[test]
    fn missing_register_and_after_terminator_are_rejected() {
        assert!(
            verify(
                &module(
                    vec![Instruction::Ret {
                        value: Some(VReg(7)),
                    }]
                )
            )
                .unwrap_err()
                .contains("Undefined register")
        );
        assert!(
            verify(
                &module(
                    vec![
                        Instruction::Ret {
                            value: Some(VReg(0)),
                        },
                        Instruction::ConstInt {
                            dest: VReg(0),
                            value: 0,
                        }
                    ]
                )
            )
                .unwrap_err()
                .contains("terminator")
        );
    }
    #[test]
    fn nondominating_definition_is_rejected() {
        let mut m = module(
            vec![
                Instruction::ConstBool {
                    dest: VReg(0),
                    value: true,
                },
                Instruction::CondBr {
                    cond: VReg(0),
                    if_true: BlockId(1),
                    if_false: BlockId(2),
                }
            ]
        );
        m.functions[0].blocks.extend([
            BasicBlock {
                id: BlockId(1),
                name: "then".into(),
                instructions: vec![
                    Instruction::ConstInt {
                        dest: VReg(1),
                        value: 7,
                    },
                    Instruction::Br { target: BlockId(2) }
                ],
            },
            BasicBlock {
                id: BlockId(2),
                name: "exit".into(),
                instructions: vec![Instruction::Ret {
                    value: Some(VReg(1)),
                }],
            },
        ]);
        assert!(verify(&m).unwrap_err().contains("dominate"));
    }

    #[test]
    fn shuffled_loop_dominance_preserves_rejections() {
        let mut m = module(
            vec![
                Instruction::ConstBool { dest: VReg(0), value: true },
                Instruction::ConstInt { dest: VReg(1), value: 7 },
                Instruction::CondBr { cond: VReg(0), if_true: BlockId(1), if_false: BlockId(2) }
            ]
        );
        // A diamond feeds a back edge. Listing order is deliberately unrelated
        // to traversal order; the fixed point must still reject a branch-local
        // definition used at the common exit.
        for id in [6, 3, 1, 5, 2, 4] {
            let instructions = match id {
                1 => vec![Instruction::Br { target: BlockId(3) }],
                2 => vec![Instruction::Br { target: BlockId(4) }],
                3 | 4 => vec![Instruction::Br { target: BlockId(5) }],
                5 =>
                    vec![Instruction::CondBr {
                        cond: VReg(0),
                        if_true: BlockId(0),
                        if_false: BlockId(6),
                    }],
                _ => vec![Instruction::Ret { value: Some(VReg(1)) }],
            };
            m.functions[0].blocks.push(BasicBlock {
                id: BlockId(id),
                name: format!("b{id}"),
                instructions,
            });
        }
        verify(&m).unwrap();
        let definition = m.functions[0].blocks[0].instructions.remove(1);
        m.functions[0].blocks
            .iter_mut()
            .find(|b| b.id == BlockId(1))
            .unwrap()
            .instructions.insert(0, definition);
        assert!(verify(&m).unwrap_err().contains("dominate"));
    }
}

/// Representation-aware SSA typing shared with suspension lowering.
pub fn value_types(module: &ModuleIr, f: &FunctionIr) -> Result<HashMap<VReg, IrType>, String> {
    let functions = module.functions
        .iter()
        .map(|f| (f.name.as_str(), f))
        .collect::<HashMap<_, _>>();
    let fail = |msg: String| format!("{}: {}", f.name, msg);
    let instructions = f.blocks
        .iter()
        .flat_map(|b| &b.instructions)
        .collect::<Vec<_>>();
    let constants = instructions
        .iter()
        .filter_map(|i| {
            match i {
                Instruction::ConstInt { dest, value } => Some((*dest, *value)),
                _ => None,
            }
        })
        .collect::<HashMap<_, _>>();
    let mut types = module.globals
        .iter()
        .map(|(r, t)| (*r, IrType::Ptr(Box::new(t.clone()))))
        .collect::<HashMap<_, _>>();
    types.extend(f.args.iter().cloned());
    // Resolve types independently of the block vector's listing order.
    loop {
        let mut changed = false;
        for i in &instructions {
            let Some(dest) = i.destination() else {
                continue;
            };
            if types.contains_key(&dest) {
                continue;
            }
            let ty = match i {
                Instruction::Alloca { ty, .. } => Some(IrType::Ptr(Box::new(ty.clone()))),
                Instruction::AllocArray { ty, .. } => Some(IrType::Array(0, Box::new(ty.clone()))),
                Instruction::AllocStruct { class_name, .. } =>
                    Some(IrType::Ptr(Box::new(IrType::Struct(class_name.clone(), vec![])))),
                Instruction::ConstInt { .. } => Some(IrType::I64),
                Instruction::SizeOf { .. } => Some(IrType::U64),
                Instruction::ConstFloat { ty, .. } | Instruction::Load { ty, .. } => {
                    Some(ty.clone())
                }
                Instruction::ConstBool { .. } => Some(IrType::Bool),
                Instruction::ConstString { .. } => Some(IrType::Ptr(Box::new(IrType::I8))),
                Instruction::Cast { target_ty, .. } => Some(target_ty.clone()),
                Instruction::Call { func_name, .. } =>
                    functions.get(func_name.as_str()).map(|f| f.ret_type.clone()),
                | Instruction::DynamicCall { ret_type, .. }
                | Instruction::CallClosure { ret_type, .. }
                | Instruction::IndirectCall { ret_type, .. } => Some(ret_type.clone()),
                Instruction::MakeFatPtr { .. } => Some(IrType::FatPtr),
                Instruction::LoadFnPtr { .. } => Some(IrType::Ptr(Box::new(IrType::Any))),
                Instruction::MakeClosure { fn_name, .. } => {
                    functions.get(fn_name.as_str()).map(|f| {
                        IrType::Closure(
                            f.args
                                .iter()
                                .skip(1)
                                .map(|(_, t)| t.clone())
                                .collect(),
                            Box::new(f.ret_type.clone())
                        )
                    })
                }
                | Instruction::CmpEq { .. }
                | Instruction::CmpNeq { .. }
                | Instruction::CmpLt { .. }
                | Instruction::CmpLe { .. }
                | Instruction::CmpGt { .. }
                | Instruction::CmpGe { .. } => Some(IrType::Bool),
                Instruction::Div { ty, .. } | Instruction::Mod { ty, .. } => Some(ty.clone()),
                | Instruction::Add { left, right, .. }
                | Instruction::Sub { left, right, .. }
                | Instruction::Mul { left, right, .. } => {
                    types
                        .get(left)
                        .zip(types.get(right))
                        .map(|(a, b)| {
                            if floating(a) {
                                a.clone()
                            } else if floating(b) {
                                b.clone()
                            } else {
                                IrType::I64
                            }
                        })
                }
                Instruction::BitAnd { .. } | Instruction::BitOr { .. } => Some(IrType::I64),
                Instruction::GetElementPtr { base_ty, indices, .. } => {
                    let element = if indices.len() == 2 {
                        if let IrType::Struct(name, fields) = base_ty {
                            if let Some(layout) = module.structs.get(name) {
                                if !fields.is_empty() && fields != layout {
                                    return Err(
                                        fail(format!("Conflicting field layout for {name}"))
                                    );
                                }
                            }
                            let fields = if fields.is_empty() {
                                module.structs.get(name).unwrap_or(fields)
                            } else {
                                fields
                            };
                            let index = constants
                                .get(&indices[1])
                                .ok_or_else(|| {
                                    fail("Struct field index must be constant".into())
                                })?;
                            fields
                                .get(usize::try_from(*index).unwrap_or(usize::MAX))
                                .cloned()
                                .ok_or_else(|| {
                                    fail(format!("Struct field index out of bounds for {name}"))
                                })?
                        } else {
                            return Err(fail("Multiple GEP indices require a struct layout".into()));
                        }
                    } else if indices.len() == 1 {
                        base_ty.clone()
                    } else {
                        return Err(
                            fail("GEP requires one element index or two field indices".into())
                        );
                    };
                    Some(IrType::Ptr(Box::new(element)))
                }
                _ => None,
            };
            if let Some(ty) = ty {
                types.insert(dest, ty);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    Ok(types)
}

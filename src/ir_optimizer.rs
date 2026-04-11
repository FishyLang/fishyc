use crate::ir::{FunctionIr, Instruction, IrType, ModuleIr, VReg};
use std::collections::{HashMap, HashSet};

pub struct IrOptimizer;

impl IrOptimizer {
    pub fn optimize(module: &mut ModuleIr) {
        Self::inline_functions(module);

        for func in &mut module.functions {
            let mut changed = true;
            while changed {
                changed = false;
                changed |= Self::fold_constants(func);
                changed |= Self::simplify_branches(func);
                changed |= Self::eliminate_dead_code(func);
            }
        }

        Self::eliminate_dead_functions(module);
    }

    // --- stage 1: constant folding & load forwarding ---
    fn fold_constants(func: &mut FunctionIr) -> bool {
        let mut changed = false;
        for block in &mut func.blocks {
            let mut new_instructions = Vec::new();

            let mut known_ints = HashMap::new();
            let mut known_bools = HashMap::new();
            let mut known_floats: HashMap<VReg, f64> = HashMap::new();

            for inst in &block.instructions {
                let mut optimized_inst = inst.clone();

                match inst {
                    Instruction::ConstInt { dest, value } => {
                        known_ints.insert(*dest, *value);
                    }
                    Instruction::ConstFloat { dest, value, .. } => {
                        known_floats.insert(*dest, *value);
                    }
                    Instruction::ConstBool { dest, value } => {
                        known_bools.insert(*dest, *value);
                    }

                    Instruction::Load { dest, src_ptr, ty, .. } => {
                        if let Some(&val) = known_ints.get(src_ptr) {
                            optimized_inst = Instruction::ConstInt {
                                dest: *dest,
                                value: val,
                            };
                            known_ints.insert(*dest, val);
                            changed = true;
                        } else if let Some(&val) = known_floats.get(src_ptr) {
                            optimized_inst = Instruction::ConstFloat {
                                dest: *dest,
                                value: val,
                                ty: ty.clone(),
                            };
                            known_floats.insert(*dest, val);
                            changed = true;
                        } else if let Some(&val) = known_bools.get(src_ptr) {
                            optimized_inst = Instruction::ConstBool {
                                dest: *dest,
                                value: val,
                            };
                            known_bools.insert(*dest, val);
                            changed = true;
                        }
                    }

                    Instruction::Add { dest, left, right } => {
                        if let (Some(&l), Some(&r)) = (known_ints.get(left), known_ints.get(right)) {
                            let result = l + r;
                            optimized_inst = Instruction::ConstInt {
                                dest: *dest,
                                value: result,
                            };
                            known_ints.insert(*dest, result);
                            changed = true;
                        } else if let (Some(&l), Some(&r)) = (known_floats.get(left), known_floats.get(right)) {
                            let result = l + r;
                            optimized_inst = Instruction::ConstFloat {
                                dest: *dest,
                                value: result,
                                ty: IrType::F64,
                            };
                            known_floats.insert(*dest, result);
                            changed = true;
                        }
                    }
                    Instruction::Sub { dest, left, right } => {
                        if let (Some(&l), Some(&r)) = (known_ints.get(left), known_ints.get(right)) {
                            let result = l - r;
                            optimized_inst = Instruction::ConstInt {
                                dest: *dest,
                                value: result,
                            };
                            known_ints.insert(*dest, result);
                            changed = true;
                        } else if let (Some(&l), Some(&r)) = (known_floats.get(left), known_floats.get(right)) {
                            let result = l - r;
                            optimized_inst = Instruction::ConstFloat {
                                dest: *dest,
                                value: result,
                                ty: IrType::F64,
                            };
                            known_floats.insert(*dest, result);
                            changed = true;
                        }
                    }
                    Instruction::Mul { dest, left, right } => {
                        if let (Some(&l), Some(&r)) = (known_ints.get(left), known_ints.get(right)) {
                            let result = l * r;
                            optimized_inst = Instruction::ConstInt {
                                dest: *dest,
                                value: result,
                            };
                            known_ints.insert(*dest, result);
                            changed = true;
                        } else if let (Some(&l), Some(&r)) = (known_floats.get(left), known_floats.get(right)) {
                            let result = l * r;
                            optimized_inst = Instruction::ConstFloat {
                                dest: *dest,
                                value: result,
                                ty: IrType::F64,
                            };
                            known_floats.insert(*dest, result);
                            changed = true;
                        }
                    }
                    Instruction::Div { dest, left, right } => {
                        if let (Some(&l), Some(&r)) = (known_ints.get(left), known_ints.get(right)) {
                            if r != 0 {
                                let result = l / r;
                                optimized_inst = Instruction::ConstInt {
                                    dest: *dest,
                                    value: result,
                                };
                                known_ints.insert(*dest, result);
                                changed = true;
                            }
                        } else if let (Some(&l), Some(&r)) = (known_floats.get(left), known_floats.get(right)) {
                            if r != 0.0 {
                                let result = l / r;
                                optimized_inst = Instruction::ConstFloat {
                                    dest: *dest,
                                    value: result,
                                    ty: IrType::F64,
                                };
                                known_floats.insert(*dest, result);
                                changed = true;
                            }
                        }
                    }
                    Instruction::Mod { dest, left, right } => {
                        if let (Some(&l), Some(&r)) = (known_ints.get(left), known_ints.get(right)) {
                            if r != 0 {
                                let result = l % r;
                                optimized_inst = Instruction::ConstInt {
                                    dest: *dest,
                                    value: result,
                                };
                                known_ints.insert(*dest, result);
                                changed = true;
                            }
                        } else if let (Some(&l), Some(&r)) = (known_floats.get(left), known_floats.get(right)) {
                            if r != 0.0 {
                                let result = l % r;
                                optimized_inst = Instruction::ConstFloat {
                                    dest: *dest,
                                    value: result,
                                    ty: IrType::F64,
                                };
                                known_floats.insert(*dest, result);
                                changed = true;
                            }
                        }
                    }

                    Instruction::CmpLt { dest, left, right } => {
                        if left == right {
                            optimized_inst = Instruction::ConstBool {
                                dest: *dest,
                                value: false,
                            };
                            known_bools.insert(*dest, false);
                            changed = true;
                        } else if let (Some(&l), Some(&r)) = (
                            known_ints.get(left),
                            known_ints.get(right),
                        ) {
                            let result = l < r;
                            optimized_inst = Instruction::ConstBool {
                                dest: *dest,
                                value: result,
                            };
                            known_bools.insert(*dest, result);
                            changed = true;
                        } else if let (Some(&l), Some(&r)) = (
                            known_floats.get(left),
                            known_floats.get(right),
                        ) {
                            let result = l < r;
                            optimized_inst = Instruction::ConstBool {
                                dest: *dest,
                                value: result,
                            };
                            known_bools.insert(*dest, result);
                            changed = true;
                        }
                    }
                    Instruction::CmpGt { dest, left, right } => {
                        if left == right {
                            optimized_inst = Instruction::ConstBool {
                                dest: *dest,
                                value: false,
                            };
                            known_bools.insert(*dest, false);
                            changed = true;
                        } else if let (Some(&l), Some(&r)) = (
                            known_ints.get(left),
                            known_ints.get(right),
                        ) {
                            let result = l > r;
                            optimized_inst = Instruction::ConstBool {
                                dest: *dest,
                                value: result,
                            };
                            known_bools.insert(*dest, result);
                            changed = true;
                        } else if let (Some(&l), Some(&r)) = (
                            known_floats.get(left),
                            known_floats.get(right),
                        ) {
                            let result = l > r;
                            optimized_inst = Instruction::ConstBool {
                                dest: *dest,
                                value: result,
                            };
                            known_bools.insert(*dest, result);
                            changed = true;
                        }
                    }
                    Instruction::CmpEq { dest, left, right } => {
                        if left == right {
                            optimized_inst = Instruction::ConstBool {
                                dest: *dest,
                                value: true,
                            };
                            known_bools.insert(*dest, true);
                            changed = true;
                        } else if let Some(&l) = known_ints.get(left) {
                            if let Some(&r) = known_ints.get(right) {
                                let result = l == r;
                                optimized_inst = Instruction::ConstBool {
                                    dest: *dest,
                                    value: result,
                                };
                                known_bools.insert(*dest, result);
                                changed = true;
                            }
                        } else if let Some(&l) = known_floats.get(left) {
                            if let Some(&r) = known_floats.get(right) {
                                let result = l == r;
                                optimized_inst = Instruction::ConstBool {
                                    dest: *dest,
                                    value: result,
                                };
                                known_bools.insert(*dest, result);
                                changed = true;
                            }
                        } else if let Some(&l) = known_bools.get(left) {
                            if let Some(&r) = known_bools.get(right) {
                                let result = l == r;
                                optimized_inst = Instruction::ConstBool {
                                    dest: *dest,
                                    value: result,
                                };
                                known_bools.insert(*dest, result);
                                changed = true;
                            }
                        }
                    }
                    Instruction::CmpNeq { dest, left, right } => {
                        if left == right {
                            optimized_inst = Instruction::ConstBool {
                                dest: *dest,
                                value: false,
                            };
                            known_bools.insert(*dest, false);
                            changed = true;
                        } else if let Some(&l) = known_ints.get(left) {
                            if let Some(&r) = known_ints.get(right) {
                                let result = l != r;
                                optimized_inst = Instruction::ConstBool {
                                    dest: *dest,
                                    value: result,
                                };
                                known_bools.insert(*dest, result);
                                changed = true;
                            }
                        } else if let Some(&l) = known_floats.get(left) {
                            if let Some(&r) = known_floats.get(right) {
                                let result = l != r;
                                optimized_inst = Instruction::ConstBool {
                                    dest: *dest,
                                    value: result,
                                };
                                known_bools.insert(*dest, result);
                                changed = true;
                            }
                        } else if let Some(&l) = known_bools.get(left) {
                            if let Some(&r) = known_bools.get(right) {
                                let result = l != r;
                                optimized_inst = Instruction::ConstBool {
                                    dest: *dest,
                                    value: result,
                                };
                                known_bools.insert(*dest, result);
                                changed = true;
                            }
                        }
                    }
                    Instruction::CmpLe { dest, left, right } => {
                        if left == right {
                            optimized_inst = Instruction::ConstBool {
                                dest: *dest,
                                value: true,
                            };
                            known_bools.insert(*dest, true);
                            changed = true;
                        } else if let (Some(&l), Some(&r)) = (
                            known_ints.get(left),
                            known_ints.get(right),
                        ) {
                            let result = l <= r;
                            optimized_inst = Instruction::ConstBool {
                                dest: *dest,
                                value: result,
                            };
                            known_bools.insert(*dest, result);
                            changed = true;
                        }
                    }
                    Instruction::CmpGe { dest, left, right } => {
                        if left == right {
                            optimized_inst = Instruction::ConstBool {
                                dest: *dest,
                                value: true,
                            };
                            known_bools.insert(*dest, true);
                            changed = true;
                        } else if let (Some(&l), Some(&r)) = (
                            known_ints.get(left),
                            known_ints.get(right),
                        ) {
                            let result = l >= r;
                            optimized_inst = Instruction::ConstBool {
                                dest: *dest,
                                value: result,
                            };
                            known_bools.insert(*dest, result);
                            changed = true;
                        }
                    }

                    Instruction::CondBr {
                        cond,
                        if_true,
                        if_false,
                    } => {
                        if let Some(&cond_val) = known_bools.get(cond) {
                            let target = if cond_val { *if_true } else { *if_false };
                            optimized_inst = Instruction::Br { target };
                            changed = true;
                        }
                    }

                    Instruction::Cast { dest, value, target_ty } => {
                        if let Some(&val) = known_ints.get(value) {
                            if matches!(target_ty, &IrType::I64 | &IrType::I32 | &IrType::I16 | &IrType::I8 | &IrType::Any) {
                                optimized_inst = Instruction::ConstInt {
                                    dest: *dest,
                                    value: val,
                                };
                                known_ints.insert(*dest, val);
                                changed = true;
                            } else if matches!(target_ty, &IrType::F64 | &IrType::F32 | &IrType::F16) {
                                optimized_inst = Instruction::ConstFloat {
                                    dest: *dest,
                                    value: val as f64,
                                    ty: IrType::F64,
                                };
                                known_floats.insert(*dest, val as f64);
                                changed = true;
                            }
                        } else if let Some(&val) = known_floats.get(value) {
                            if matches!(target_ty, &IrType::F64 | &IrType::F32 | &IrType::F16 | &IrType::Any) {
                                optimized_inst = Instruction::ConstFloat {
                                    dest: *dest,
                                    value: val,
                                    ty: IrType::F64,
                                };
                                known_floats.insert(*dest, val);
                                changed = true;
                            }
                        } else if let Some(&val) = known_bools.get(value) {
                            if matches!(target_ty, &IrType::Bool | &IrType::Any) {
                                optimized_inst = Instruction::ConstBool {
                                    dest: *dest,
                                    value: val,
                                };
                                known_bools.insert(*dest, val);
                                changed = true;
                            }
                        }
                    }

                    _ => {}
                }

                new_instructions.push(optimized_inst);
            }
            block.instructions = new_instructions;
        }
        changed
    }

    fn simplify_branches(func: &mut FunctionIr) -> bool {
        let mut changed = false;

        for block in &mut func.blocks {
            for inst in &mut block.instructions {
                if let Instruction::CondBr {
                    cond: _,
                    if_true,
                    if_false,
                } = inst
                {
                    if if_true == if_false {
                        *inst = Instruction::Br { target: *if_true };
                        changed = true;
                    }
                }
            }
        }

        changed
    }

    // --- stage 2: dce ---
    fn eliminate_dead_code(func: &mut FunctionIr) -> bool {
        let mut changed = false;
        if func.blocks.is_empty() {
            return false;
        }

        // step a: identify unreachable blocks
        let mut reachable = HashSet::new();
        let mut worklist = vec![func.blocks[0].id];
        reachable.insert(func.blocks[0].id);

        while let Some(block_id) = worklist.pop() {
            if let Some(block) = func.blocks.iter().find(|b| b.id == block_id) {
                if let Some(last_inst) = block.instructions.last() {
                    match last_inst {
                        Instruction::Br { target } => {
                            if reachable.insert(*target) {
                                worklist.push(*target);
                            }
                        }
                        Instruction::CondBr {
                            if_true, if_false, ..
                        } => {
                            if reachable.insert(*if_true) {
                                worklist.push(*if_true);
                            }
                            if reachable.insert(*if_false) {
                                worklist.push(*if_false);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        let original_block_count = func.blocks.len();
        func.blocks.retain(|b| reachable.contains(&b.id));
        if func.blocks.len() < original_block_count {
            changed = true;
        }

        // step b: count uses and track what pointers are really read
        let mut uses = HashMap::new();
        let mut ptr_reads = HashSet::new();

        for block in &func.blocks {
            for inst in &block.instructions {
                match inst {
                    Instruction::GetElementPtr {
                        base_ptr, indices, ..
                    } => {
                        *uses.entry(*base_ptr).or_insert(0) += 1;
                        ptr_reads.insert(*base_ptr);

                        for idx in indices {
                            *uses.entry(*idx).or_insert(0) += 1;
                        }
                    }

                    Instruction::Load { src_ptr, .. } => {
                        *uses.entry(*src_ptr).or_insert(0) += 1;
                        ptr_reads.insert(*src_ptr); // pointer was read
                    }

                    Instruction::Call { args, .. } => {
                        for arg in args {
                            *uses.entry(*arg).or_insert(0) += 1;
                            ptr_reads.insert(*arg); // assume functions read arguments
                        }
                    }

                    Instruction::Store { ptr, value, .. } => {
                        *uses.entry(*ptr).or_insert(0) += 1;
                        *uses.entry(*value).or_insert(0) += 1;
                    }

                    Instruction::Add { left, right, .. }
                    | Instruction::Sub { left, right, .. }
                    | Instruction::Mul { left, right, .. }
                    | Instruction::Div { left, right, .. }
                    | Instruction::Mod { left, right, .. }
                    | Instruction::CmpEq { left, right, .. }
                    | Instruction::CmpNeq { left, right, .. }
                    | Instruction::CmpLt { left, right, .. }
                    | Instruction::CmpLe { left, right, .. }
                    | Instruction::CmpGt { left, right, .. }
                    | Instruction::CmpGe { left, right, .. } => {
                        *uses.entry(*left).or_insert(0) += 1;
                        *uses.entry(*right).or_insert(0) += 1;
                    }

                    Instruction::CondBr { cond, .. } => {
                        *uses.entry(*cond).or_insert(0) += 1;
                    }

                    Instruction::Ret { value: Some(v) } => {
                        *uses.entry(*v).or_insert(0) += 1;
                    }

                    Instruction::AllocArray { size, .. } => {
                        *uses.entry(*size).or_insert(0) += 1;
                    }

                    Instruction::AllocStruct { .. } => {}

                    Instruction::Cast { value, .. } => {
                        *uses.entry(*value).or_insert(0) += 1;
                    }

                    Instruction::MakeFatPtr { data_ptr, .. } => {
                        *uses.entry(*data_ptr).or_insert(0) += 1;
                    }

                    Instruction::DynamicCall { fat_ptr, args, .. } => {
                        *uses.entry(*fat_ptr).or_insert(0) += 1;
                        ptr_reads.insert(*fat_ptr); // fatptr is read to access the vtable
                        for arg in args {
                            *uses.entry(*arg).or_insert(0) += 1;
                            ptr_reads.insert(*arg);
                        }
                    }

                    Instruction::CallClosure {
                        closure_ptr, args, ..
                    } => {
                        *uses.entry(*closure_ptr).or_insert(0) += 1;
                        for arg in args {
                            *uses.entry(*arg).or_insert(0) += 1;
                        }
                    }
                    Instruction::MakeClosure { env_ptr, .. } => {
                        *uses.entry(*env_ptr).or_insert(0) += 1;
                    }
                    Instruction::Retain { ptr } | Instruction::Release { ptr } => {
                        *uses.entry(*ptr).or_insert(0) += 1;
                    }

                    _ => {}
                }
            }
        }

        // identify which locals (alloca and objects) are never read
        let mut dead_allocas = HashSet::new();
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Instruction::Alloca { dest, .. } | Instruction::AllocStruct { dest, .. } =
                    inst
                {
                    if !ptr_reads.contains(dest) {
                        dead_allocas.insert(*dest);
                    }
                }
            }
        }

        // step c: filter instructions
        for block in &mut func.blocks {
            let orig_len = block.instructions.len();
            block.instructions.retain(|inst| match inst {
                Instruction::Alloca { dest, .. } | Instruction::AllocStruct { dest, .. } => {
                    !dead_allocas.contains(dest)
                }
                Instruction::Store { ptr, .. } => !dead_allocas.contains(ptr),

                Instruction::ConstInt { dest, .. }
                | Instruction::ConstFloat { dest, .. }
                | Instruction::ConstBool { dest, .. }
                | Instruction::ConstString { dest, .. }
                | Instruction::Add { dest, .. }
                | Instruction::Sub { dest, .. }
                | Instruction::Mul { dest, .. }
                | Instruction::Div { dest, .. }
                | Instruction::Mod { dest, .. }
                | Instruction::CmpEq { dest, .. }
                | Instruction::CmpNeq { dest, .. }
                | Instruction::CmpLt { dest, .. }
                | Instruction::CmpLe { dest, .. }
                | Instruction::CmpGt { dest, .. }
                | Instruction::CmpGe { dest, .. }
                | Instruction::Load { dest, .. }
                | Instruction::AllocArray { dest, .. }
                | Instruction::GetElementPtr { dest, .. }
                | Instruction::MakeFatPtr { dest, .. }
                | Instruction::LoadFnPtr { dest, .. }
                | Instruction::MakeClosure { dest, .. } => uses.get(dest).copied().unwrap_or(0) > 0,

                Instruction::Br { .. }
                | Instruction::CondBr { .. }
                | Instruction::Ret { .. }
                | Instruction::Call { .. }
                | Instruction::DynamicCall { .. }
                | Instruction::IndirectCall { .. }
                | Instruction::CallClosure { .. }
                | Instruction::Unreachable
                | Instruction::Retain { .. }
                | Instruction::Release { .. } => true,

                Instruction::Cast { dest, .. } => uses.get(dest).copied().unwrap_or(0) > 0,
            });
            if block.instructions.len() < orig_len {
                changed = true;
            }
        }
        changed
    }

    fn inline_functions(module: &mut ModuleIr) -> bool {
        let mut changed = false;

        let mut next_reg: usize = 0;
        for func in &module.functions {
            for (reg, _) in &func.args {
                next_reg = next_reg.max(reg.0 + 1);
            }
            for block in &func.blocks {
                for inst in &block.instructions {
                    let dest_id = match inst {
                        Instruction::Alloca { dest, .. }
                        | Instruction::AllocArray { dest, .. }
                        | Instruction::AllocStruct { dest, .. }
                        | Instruction::GetElementPtr { dest, .. }
                        | Instruction::Load { dest, .. }
                        | Instruction::ConstInt { dest, .. }
                        | Instruction::ConstBool { dest, .. }
                        | Instruction::ConstString { dest, .. }
                        | Instruction::Add { dest, .. }
                        | Instruction::Sub { dest, .. }
                        | Instruction::Mul { dest, .. }
                        | Instruction::Div { dest, .. }
                        | Instruction::Mod { dest, .. }
                        | Instruction::CmpEq { dest, .. }
                        | Instruction::CmpNeq { dest, .. }
                        | Instruction::CmpLt { dest, .. }
                        | Instruction::CmpLe { dest, .. }
                        | Instruction::CmpGt { dest, .. }
                        | Instruction::CmpGe { dest, .. }
                        | Instruction::Call { dest, .. }
                        | Instruction::Cast { dest, .. }
                        | Instruction::MakeFatPtr { dest, .. }
                        | Instruction::DynamicCall { dest, .. } => Some(dest.0),
                        _ => None,
                    };
                    if let Some(id) = dest_id {
                        next_reg = next_reg.max(id + 1);
                    }
                }
            }
        }

        // prevent that single-line recursive functions are inlined
        let mut inlinables = HashMap::new();
        for func in &module.functions {
            if func.blocks.len() == 1 {
                let is_recursive = func.blocks[0].instructions.iter().any(|inst| {
                    if let Instruction::Call { func_name, .. } = inst {
                        func_name == &func.name
                    } else {
                        false
                    }
                });

                if !is_recursive {
                    inlinables.insert(func.name.clone(), func.clone());
                }
            }
        }

        for func in &mut module.functions {
            for block in &mut func.blocks {
                let mut new_instructions = Vec::new();

                for inst in &block.instructions {
                    if let Instruction::Call {
                        dest: call_dest,
                        func_name,
                        args,
                    } = inst
                    {
                        if let Some(target) = inlinables.get(func_name) {
                            let mut reg_map = HashMap::new();

                            for (i, (param_reg, _)) in target.args.iter().enumerate() {
                                if i < args.len() {
                                    reg_map.insert(*param_reg, args[i]);
                                }
                            }

                            for t_inst in &target.blocks[0].instructions {
                                let mut cloned = t_inst.clone();

                                let mut get_new_reg = |old_reg: VReg| -> VReg {
                                    if let Some(&r) = reg_map.get(&old_reg) {
                                        r
                                    } else {
                                        let nr = VReg(next_reg);
                                        next_reg += 1;
                                        reg_map.insert(old_reg, nr);
                                        nr
                                    }
                                };
                                let remap = |r: VReg, map: &HashMap<VReg, VReg>| -> VReg {
                                    *map.get(&r).unwrap_or(&r)
                                };

                                match &mut cloned {
                                    Instruction::ConstInt { dest, .. }
                                    | Instruction::ConstBool { dest, .. }
                                    | Instruction::ConstString { dest, .. } => {
                                        *dest = get_new_reg(*dest);
                                    }

                                    Instruction::GetElementPtr {
                                        dest,
                                        base_ptr,
                                        indices,
                                        ..
                                    } => {
                                        *dest = get_new_reg(*dest);
                                        *base_ptr = remap(*base_ptr, &reg_map);

                                        for idx in indices.iter_mut() {
                                            *idx = remap(*idx, &reg_map);
                                        }
                                    }

                                    Instruction::Alloca { dest, .. }
                                    | Instruction::AllocStruct { dest, .. } => {
                                        *dest = get_new_reg(*dest);
                                    }

                                    Instruction::AllocArray { dest, size, .. } => {
                                        *dest = get_new_reg(*dest);
                                        *size = remap(*size, &reg_map);
                                    }

                                    Instruction::Load { dest, src_ptr, .. } => {
                                        *dest = get_new_reg(*dest);
                                        *src_ptr = remap(*src_ptr, &reg_map);
                                    }

                                    Instruction::Store { ptr, value, .. } => {
                                        *ptr = remap(*ptr, &reg_map);
                                        *value = remap(*value, &reg_map);
                                    }

                                    Instruction::Cast { dest, value, .. } => {
                                        *dest = get_new_reg(*dest);
                                        *value = remap(*value, &reg_map);
                                    }

                                    Instruction::Add {
                                        dest, left, right, ..
                                    }
                                    | Instruction::Sub {
                                        dest, left, right, ..
                                    }
                                    | Instruction::Mul {
                                        dest, left, right, ..
                                    }
                                    | Instruction::Div {
                                        dest, left, right, ..
                                    }
                                    | Instruction::Mod {
                                        dest, left, right, ..
                                    }
                                    | Instruction::CmpEq {
                                        dest, left, right, ..
                                    }
                                    | Instruction::CmpNeq {
                                        dest, left, right, ..
                                    }
                                    | Instruction::CmpLt {
                                        dest, left, right, ..
                                    }
                                    | Instruction::CmpLe {
                                        dest, left, right, ..
                                    }
                                    | Instruction::CmpGt {
                                        dest, left, right, ..
                                    }
                                    | Instruction::CmpGe {
                                        dest, left, right, ..
                                    } => {
                                        *dest = get_new_reg(*dest);
                                        *left = remap(*left, &reg_map);
                                        *right = remap(*right, &reg_map);
                                    }

                                    Instruction::Call { dest, args, .. } => {
                                        *dest = get_new_reg(*dest);
                                        for arg in args.iter_mut() {
                                            *arg = remap(*arg, &reg_map);
                                        }
                                    }

                                    Instruction::CondBr { cond, .. } => {
                                        *cond = remap(*cond, &reg_map);
                                    }

                                    Instruction::Ret {
                                        value: Some(ret_val),
                                    } => {
                                        let mapped_ret = remap(*ret_val, &reg_map);
                                        if target.ret_type != IrType::Void {
                                            new_instructions.push(Instruction::Cast {
                                                dest: *call_dest,
                                                value: mapped_ret,
                                                target_ty: target.ret_type.clone(),
                                            });
                                        }
                                        continue;
                                    }

                                    Instruction::Ret { value: None } => {
                                        continue;
                                    }

                                    _ => {}
                                }
                                new_instructions.push(cloned);
                            }
                            changed = true;
                            continue;
                        }
                    }
                    new_instructions.push(inst.clone());
                }
                block.instructions = new_instructions;
            }
        }
        changed
    }

    // tree shaking (global dead function elimination)
    fn eliminate_dead_functions(module: &mut crate::ir::ModuleIr) {
        let mut reachable = HashSet::new();
        let mut worklist = vec!["main".to_string(), "__global_init".to_string()];

        // 1. protect functions in vtables (we really do not want to get rid of them)
        for method_names in module.vtables.values() {
            for m_name in method_names {
                if !m_name.is_empty() {
                    // put them on the worklist so that the functions that methods call also live
                    worklist.push(m_name.clone());
                }
            }
        }

        // 2. protect native functions (we REALLY DO NOT WANT these to be deleted, even more than the vtable functions. some are vital for the runtime)
        let hardcoded_externs = vec![
            "malloc", "free", "realloc", "printf", "exit", "strlen", "strcat", "memcpy", "fopen",
            "fclose", "fputs", "fprintf", "panic",
        ];
        for ext in hardcoded_externs {
            reachable.insert(ext.to_string());
        }

        // 3. reachability algorithm (call graph)
        while let Some(func_name) = worklist.pop() {
            // if we visited this function, ignore (avoids infinite loops)
            if !reachable.insert(func_name.clone()) {
                continue;
            }

            // we search for the function and see what it calls
            if let Some(func) = module.functions.iter().find(|f| f.name == func_name) {
                for block in &func.blocks {
                    for inst in &block.instructions {
                        match inst {
                            // direct calls
                            crate::ir::Instruction::Call {
                                func_name: target, ..
                            } => {
                                worklist.push(target.clone());
                            }

                            // closure creation
                            crate::ir::Instruction::MakeClosure {
                                fn_name: target, ..
                            } => {
                                worklist.push(target.clone());
                            }

                            // passing function pointers
                            crate::ir::Instruction::LoadFnPtr {
                                fn_name: target, ..
                            } => {
                                worklist.push(target.clone());
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // 4. cut functions that are not on the reachable vec
        let original_count = module.functions.len();
        module.functions.retain(|f| reachable.contains(&f.name));
        let new_count = module.functions.len();

        if original_count != new_count {
            println!(
                "Optimizer: removed {} dead functions from the module",
                original_count - new_count
            );
        }
    }
}

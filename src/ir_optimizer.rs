use crate::ir::{ BasicBlock, BlockId, FunctionIr, Instruction, IrType, ModuleIr, VReg };
use std::collections::{ HashMap, HashSet };

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
                changed |= Self::simplify_arithmetics(func);
                changed |= Self::eliminate_common_subexpressions(func);
                changed |= Self::simplify_identity_casts(func);
                changed |= Self::merge_blocks(func);
                changed |= Self::eliminate_empty_branches(func);
                changed |= Self::eliminate_dead_code(func);
            }
        }

        Self::eliminate_dead_functions(module);
    }

    // Constants are SSA values. Memory forwarding needs alias/escape analysis,
    // and narrow/float folding needs exact typed constants; leave these to LLVM.
    fn fold_constants(func: &mut FunctionIr) -> bool {
        let mut changed=false;
        for block in &mut func.blocks {
            let mut ints=HashMap::<VReg,i64>::new();
            let mut bools=HashMap::<VReg,bool>::new();
            for inst in &mut block.instructions {
                let replacement=match inst {
                    Instruction::Add {dest,left,right} | Instruction::Sub {dest,left,right} |
                    Instruction::Mul {dest,left,right} => {
                        let (d,l,r)=(*dest,*left,*right);
                        ints.get(&l).zip(ints.get(&r)).map(|(&l,&r)| Instruction::ConstInt {dest:d,value:match inst {
                            Instruction::Add {..}=>l.wrapping_add(r), Instruction::Sub {..}=>l.wrapping_sub(r), _=>l.wrapping_mul(r)
                        }})
                    }
                    Instruction::Div {dest,left,right,ty}|Instruction::Mod {dest,left,right,ty} if *ty==IrType::I64 || *ty==IrType::U64 => {
                        let (d,l,r,signed)=(*dest,*left,*right,*ty==IrType::I64);
                        ints.get(&l).zip(ints.get(&r)).and_then(|(&l,&r)| {
                            let value=match inst {
                                Instruction::Div {..} if signed=>l.checked_div(r),
                                Instruction::Mod {..} if signed=>l.checked_rem(r),
                                Instruction::Div {..}=>(l as u64).checked_div(r as u64).map(|v|v as i64),
                                _=>(l as u64).checked_rem(r as u64).map(|v|v as i64),
                            }?;
                            Some(Instruction::ConstInt {dest:d,value})
                        })
                    }
                    Instruction::BitAnd {dest,left,right}|Instruction::BitOr {dest,left,right}=>{
                        let (d,l,r)=(*dest,*left,*right);
                        ints.get(&l).zip(ints.get(&r)).map(|(&l,&r)|Instruction::ConstInt {dest:d,value:if matches!(inst,Instruction::BitAnd {..}) {l&r}else{l|r}})
                    }
                    Instruction::CondBr {cond,if_true,if_false}=>bools.get(cond).map(|v|Instruction::Br {target:if *v {*if_true} else {*if_false}}),
                    _=>None,
                };
                if let Some(new)=replacement {*inst=new;changed=true;}
                match inst {
                    Instruction::ConstInt {dest,value}=>{ints.insert(*dest,*value);}
                    Instruction::ConstBool {dest,value}=>{bools.insert(*dest,*value);}
                    _=>{}
                }
            }
        }
        changed
    }

    fn simplify_branches(func: &mut FunctionIr) -> bool {
        let mut changed = false;

        for block in &mut func.blocks {
            for inst in &mut block.instructions {
                if let Instruction::CondBr { cond: _, if_true, if_false } = inst {
                    if if_true == if_false {
                        *inst = Instruction::Br { target: *if_true };
                        changed = true;
                    }
                }
            }
        }

        changed
    }

    fn simplify_arithmetics(func: &mut FunctionIr) -> bool {
        let mut aliases=HashMap::new();
        let ints: HashMap<_,_>=func.blocks.iter().flat_map(|b| &b.instructions).filter_map(|i| {
            if let Instruction::ConstInt {dest,value}=i {Some((*dest,*value))} else {None}
        }).collect();
        for block in &func.blocks {
            for inst in &block.instructions {
                let alias=match inst {
                    Instruction::Add {dest,left,right} if ints.get(right)==Some(&0)=>Some((*dest,*left)),
                    Instruction::Add {dest,left,right} if ints.get(left)==Some(&0)=>Some((*dest,*right)),
                    Instruction::Sub {dest,left,right} if ints.get(right)==Some(&0)=>Some((*dest,*left)),
                    Instruction::Mul {dest,left,right} if ints.get(right)==Some(&1)=>Some((*dest,*left)),
                    Instruction::Mul {dest,left,right} if ints.get(left)==Some(&1)=>Some((*dest,*right)),
                    _=>None,
                };
                if let Some((d,v))=alias {
                    // Never apply integer identities to floating arithmetic (signed zero/NaN).
                    if matches!(Self::infer_register_type(func,v),Some(IrType::I64|IrType::U64)) {aliases.insert(d,v);}
                }
            }
        }
        Self::apply_aliases(func,&aliases);
        !aliases.is_empty()
    }

    fn apply_aliases(func:&mut FunctionIr,aliases:&HashMap<VReg,VReg>) {
        for b in &mut func.blocks {
            b.instructions.retain(|i| Self::instruction_dest(i).is_none_or(|d|!aliases.contains_key(&VReg(d))));
            for i in &mut b.instructions {Self::remap_instruction_operands(i,aliases);}
        }
    }

    fn eliminate_common_subexpressions(func: &mut FunctionIr) -> bool {
        let mut aliases=HashMap::new();
        for block in &func.blocks {
            let mut expressions=HashMap::new();
            for inst in &block.instructions {
                let mut key=inst.clone();
                let dest=match &mut key {
                    Instruction::Add {dest,..}|Instruction::Sub {dest,..}|Instruction::Mul {dest,..}|
                    Instruction::BitAnd {dest,..}|Instruction::BitOr {dest,..}|
                Instruction::Div {dest,..}|Instruction::Mod {dest,..}|
                    Instruction::CmpEq {dest,..}|Instruction::CmpNeq {dest,..}|
                    Instruction::CmpLt {dest,..}|Instruction::CmpLe {dest,..}|
                    Instruction::CmpGt {dest,..}|Instruction::CmpGe {dest,..}|
                    Instruction::SizeOf {dest,..}|Instruction::LoadFnPtr {dest,..} => {let d=*dest;*dest=VReg(0);d}
                    _=>continue,
                };
                Self::remap_instruction_operands(&mut key,&aliases);
                // Includes opcode, ordered operands and signedness/type.
                let key=format!("{:?}",key);
                if let Some(existing)=expressions.get(&key) {aliases.insert(dest,*existing);} else {expressions.insert(key,dest);}
            }
        }
        // A block-local definition can have uses in dominated successor blocks.
        Self::apply_aliases(func,&aliases);
        !aliases.is_empty()
    }

    fn remap_instruction_operands(inst: &mut Instruction, alias_map: &HashMap<VReg, VReg>) {
        let resolve = |reg: VReg| {
            let mut reg = reg;

            while let Some(&alias) = alias_map.get(&reg) {
                reg = alias;
            }

            reg
        };

        match inst {
            | Instruction::Add { left, right, .. }
            | Instruction::Sub { left, right, .. }
            | Instruction::Mul { left, right, .. }
            | Instruction::BitAnd { left, right, .. }
            | Instruction::BitOr { left, right, .. }
            | Instruction::Div { left, right, .. }
            | Instruction::Mod { left, right, .. }
            | Instruction::CmpEq { left, right, .. }
            | Instruction::CmpNeq { left, right, .. }
            | Instruction::CmpLt { left, right, .. }
            | Instruction::CmpLe { left, right, .. }
            | Instruction::CmpGt { left, right, .. }
            | Instruction::CmpGe { left, right, .. } => {
                *left = resolve(*left);
                *right = resolve(*right);
            }

            | Instruction::Cast { value, .. }
            | Instruction::CondBr { cond: value, .. }
            | Instruction::MakeFatPtr { data_ptr: value, .. }
            | Instruction::MakeClosure { env_ptr: value, .. }
            | Instruction::Retain { ptr: value }
            | Instruction::Release { ptr: value } => {
                *value = resolve(*value);
            }

            Instruction::DynamicCall {fat_ptr,args,..}|Instruction::CallClosure {closure_ptr:fat_ptr,args,..} => {
                *fat_ptr=resolve(*fat_ptr);
                for arg in args {*arg=resolve(*arg);}
            }
            Instruction::Call { args, .. } => {
                for arg in args {
                    *arg = resolve(*arg);
                }
            }

            Instruction::IndirectCall { fn_ptr, args, .. } => {
                *fn_ptr = resolve(*fn_ptr);
                for arg in args {
                    *arg = resolve(*arg);
                }
            }

            Instruction::Load { src_ptr, .. } => {
                *src_ptr = resolve(*src_ptr);
            }

            Instruction::Store { ptr, value, .. } => {
                *ptr = resolve(*ptr);
                *value = resolve(*value);
            }

            Instruction::Ret { value } => {
                if let Some(v) = value {
                    *v = resolve(*v);
                }
            }

            Instruction::AllocArray { size, .. } => {
                *size = resolve(*size);
            }

            Instruction::GetElementPtr { base_ptr, indices, .. } => {
                *base_ptr = resolve(*base_ptr);
                for idx in indices {
                    *idx = resolve(*idx);
                }
            }

            _ => {}
        }
    }

    fn eliminate_empty_branches(func: &mut FunctionIr) -> bool {
        let mut changed = false;

        loop {
            let entry = func.blocks.first().map(|b| b.id);
            let mut removed = false;

            let block_index = func.blocks.iter().position(|block| {
                if let Some(entry_id) = entry {
                    if block.id == entry_id {
                        return false;
                    }
                }

                matches!(block.instructions.as_slice(), [Instruction::Br { target }] if *target != block.id)
            });

            if let Some(index) = block_index {
                let target = if let Instruction::Br { target } = func.blocks[index].instructions[0] {
                    target
                } else {
                    break;
                };

                let removed_block = func.blocks.remove(index);
                Self::replace_block_target(func, removed_block.id, target);

                changed = true;
                removed = true;
            }

            if !removed {
                break;
            }
        }

        changed
    }

    fn replace_block_target(func: &mut FunctionIr, from: BlockId, to: BlockId) {
        for block in &mut func.blocks {
            for inst in &mut block.instructions {
                match inst {
                    Instruction::Br { target } => {
                        if *target == from {
                            *target = to;
                        }
                    }

                    Instruction::CondBr { if_true, if_false, .. } => {
                        if *if_true == from {
                            *if_true = to;
                        }

                        if *if_false == from {
                            *if_false = to;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn simplify_identity_casts(func:&mut FunctionIr)->bool {
        let mut aliases=HashMap::new();
        for b in &func.blocks {for i in &b.instructions {
            if let Instruction::Cast {dest,value,target_ty,..}=i {
                // Loads normalize narrow integers to i64; their semantic type is
                // not their SSA representation. Only remove identities on casts
                // and literal values whose physical width is explicit.
                let physical=func.blocks.iter().flat_map(|b|&b.instructions).find_map(|i|match i {
                    Instruction::Cast {dest,target_ty,..} if dest==value=>Some(target_ty.clone()),
                    Instruction::ConstInt {dest,..} if dest==value=>Some(IrType::I64),
                    Instruction::SizeOf {dest,..} if dest==value=>Some(IrType::U64),
                    Instruction::ConstFloat {dest,ty,..} if dest==value=>Some(ty.clone()),
                    _=>None,
                });
                if physical.as_ref()==Some(target_ty) {aliases.insert(*dest,*value);}
            }
        }}
        Self::apply_aliases(func,&aliases);!aliases.is_empty()
    }

    fn merge_blocks(func: &mut FunctionIr) -> bool {
        let mut changed = false;

        loop {
            let predecessor_counts = Self::compute_predecessor_counts(func);
            let entry=func.blocks.first().map(|b|b.id);
            let mut merged = false;

            let block_indices: HashMap<BlockId, usize> = func.blocks
                .iter()
                .enumerate()
                .map(|(idx, block)| (block.id, idx))
                .collect();

            for idx in 0..func.blocks.len() {
                let block_id = func.blocks[idx].id;
                if let Some(Instruction::Br { target }) = func.blocks[idx].instructions.last() {
                    if *target != block_id && Some(*target)!=entry {
                        if predecessor_counts.get(target).copied().unwrap_or(0) == 1 {
                            if let Some(&target_idx) = block_indices.get(target) {
                                if target_idx != idx {
                                    let target_block = func.blocks.remove(target_idx);
                                    if
                                        let Some(block) = func.blocks
                                            .iter_mut()
                                            .find(|b| b.id == block_id)
                                    {
                                        if
                                            matches!(
                                                block.instructions.last(),
                                                Some(Instruction::Br { .. })
                                            )
                                        {
                                            block.instructions.pop();
                                        }
                                        block.instructions.extend(
                                            target_block.instructions.into_iter()
                                        );
                                    }

                                    changed = true;
                                    merged = true;

                                    break;
                                }
                            }
                        }
                    }
                }
            }

            if !merged {
                break;
            }
        }

        changed
    }

    fn compute_predecessor_counts(func: &FunctionIr) -> HashMap<BlockId, usize> {
        let mut counts = HashMap::new();
        for block in &func.blocks {
            counts.entry(block.id).or_insert(0);
        }

        for block in &func.blocks {
            if let Some(last_inst) = block.instructions.last() {
                match last_inst {
                    Instruction::Br { target } => {
                        *counts.entry(*target).or_insert(0) += 1;
                    }

                    Instruction::CondBr { if_true, if_false, .. } => {
                        *counts.entry(*if_true).or_insert(0) += 1;
                        *counts.entry(*if_false).or_insert(0) += 1;
                    }

                    _ => {}
                }
            }
        }

        counts
    }

    fn infer_register_type(func: &FunctionIr, reg: VReg) -> Option<IrType> {
        for block in &func.blocks {
            for inst in &block.instructions {
                match inst {
                    Instruction::Alloca { dest, ty, .. } if *dest == reg => {
                        return Some(IrType::Ptr(Box::new(ty.clone())));
                    }

                    Instruction::AllocArray { dest, ty, .. } if *dest == reg => {
                        return Some(IrType::Ptr(Box::new(ty.clone())));
                    }

                    Instruction::GetElementPtr { dest, base_ty, .. } if *dest == reg => {
                        return Some(IrType::Ptr(Box::new(base_ty.clone())));
                    }

                    Instruction::Load { dest, ty, .. } if *dest == reg => {
                        return Some(ty.clone());
                    }

                    Instruction::ConstInt { dest, .. } if *dest == reg => {
                        return Some(IrType::I64);
                    }

                    Instruction::ConstFloat { dest, ty, .. } if *dest == reg => {
                        return Some(ty.clone());
                    }

                    Instruction::SizeOf {dest,..} if *dest==reg => {return Some(IrType::U64);}
                    Instruction::ConstBool { dest, .. } if *dest == reg => {
                        return Some(IrType::Bool);
                    }

                    Instruction::ConstString { dest, .. } if *dest == reg => {
                        return Some(IrType::Ptr(Box::new(IrType::Any)));
                    }

                    | Instruction::CmpEq { dest, .. }
                    | Instruction::CmpNeq { dest, .. }
                    | Instruction::CmpLt { dest, .. }
                    | Instruction::CmpLe { dest, .. }
                    | Instruction::CmpGt { dest, .. }
                    | Instruction::CmpGe { dest, .. } if *dest == reg => {
                        return Some(IrType::Bool);
                    }

                    Instruction::Cast { dest, target_ty, .. } if *dest == reg => {
                        return Some(target_ty.clone());
                    }

                    Instruction::MakeFatPtr { dest, .. } if *dest == reg => {
                        return Some(IrType::FatPtr);
                    }

                    Instruction::LoadFnPtr { dest, .. } if *dest == reg => {
                        return Some(IrType::Ptr(Box::new(IrType::Any)));
                    }

                    Instruction::MakeClosure { dest, .. } if *dest == reg => {
                        return Some(IrType::Ptr(Box::new(IrType::Any)));
                    }

                    Instruction::Call { dest, .. } if *dest == reg => {
                        return None;
                    }

                    Instruction::IndirectCall { dest, ret_type, .. } if *dest == reg => {
                        return Some(ret_type.clone());
                    }

                    Instruction::DynamicCall { dest, ret_type, .. } if *dest == reg => {
                        return Some(ret_type.clone());
                    }

                    Instruction::CallClosure { dest, ret_type, .. } if *dest == reg => {
                        return Some(ret_type.clone());
                    }

                    Instruction::AllocStruct { dest, .. } if *dest == reg => {
                        return Some(IrType::Ptr(Box::new(IrType::Any)));
                    }

                    _ => {}
                }
            }
        }
        None
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

                        Instruction::CondBr { if_true, if_false, .. } => {
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

        // Only remove unused pure SSA computations. Stores and allocations may
        // escape through casts, closures, returns and other stores.
        let mut uses=HashSet::new();
        for b in &func.blocks {for i in &b.instructions {uses.extend(i.operands());}}
        for b in &mut func.blocks {
            let before=b.instructions.len();
            b.instructions.retain(|i| match i {
                Instruction::ConstInt {dest,..}|Instruction::ConstFloat {dest,..}|
                Instruction::ConstBool {dest,..}|Instruction::ConstString {dest,..}|
                Instruction::Add {dest,..}|Instruction::Sub {dest,..}|Instruction::Mul {dest,..}|
                Instruction::BitAnd {dest,..}|Instruction::BitOr {dest,..}|
                Instruction::Div {dest,..}|Instruction::Mod {dest,..}|
                Instruction::CmpEq {dest,..}|Instruction::CmpNeq {dest,..}|
                Instruction::CmpLt {dest,..}|Instruction::CmpLe {dest,..}|
                Instruction::CmpGt {dest,..}|Instruction::CmpGe {dest,..}|
                Instruction::Cast {dest,..}|Instruction::GetElementPtr {dest,..}|
                Instruction::LoadFnPtr {dest,..}=>uses.contains(dest),
                _=>true,
            });
            changed|=before!=b.instructions.len();
        }
        changed
    }

    // --- inlining constants ---
    const INLINE_TINY_THRESHOLD: usize = 8;
    const INLINE_SMALL_THRESHOLD: usize = 30;
    const INLINE_MAX_EXPANSION: usize = 60;

    fn instruction_dest(inst:&Instruction)->Option<usize> {inst.destination().map(|r|r.0)}

    fn compute_next_vreg(module: &ModuleIr) -> usize {
        let mut max_id = module.globals.keys().map(|r|r.0+1).max().unwrap_or(0);
        for func in &module.functions {
            for (reg, _) in &func.args {
                max_id = max_id.max(reg.0 + 1);
            }
            for block in &func.blocks {
                for inst in &block.instructions {
                    if let Some(id) = Self::instruction_dest(inst) {
                        max_id = max_id.max(id + 1);
                    }
                }
            }
        }
        max_id
    }

    fn compute_next_block_id(module: &ModuleIr) -> usize {
        let mut max_id: usize = 0;
        for func in &module.functions {
            for block in &func.blocks {
                max_id = max_id.max(block.id.0 + 1);
            }
        }
        max_id
    }

    fn count_function_calls(module: &ModuleIr) -> HashMap<String, usize> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for func in &module.functions {
            for block in &func.blocks {
                for inst in &block.instructions {
                    if let Instruction::Call { func_name, .. } = inst {
                        *counts.entry(func_name.clone()).or_insert(0) += 1;
                    }
                }
            }
        }
        counts
    }

    fn total_instruction_count(func: &FunctionIr) -> usize {
        func.blocks.iter().map(|b| b.instructions.len()).sum()
    }

    fn is_directly_recursive(func: &FunctionIr) -> bool {
        func.blocks.iter().any(|block| {
            block.instructions.iter().any(|inst| {
                matches!(inst, Instruction::Call { func_name, .. } if func_name == &func.name)
            })
        })
    }

    fn should_inline(func: &FunctionIr, call_count: usize) -> bool {
        if func.name=="__global_init" {return false;}
        if func.is_variadic || Self::is_directly_recursive(func) {
            return false;
        }
        let size = Self::total_instruction_count(func);
        if size <= Self::INLINE_TINY_THRESHOLD {
            return true;
        }
        if size > Self::INLINE_SMALL_THRESHOLD {
            return false;
        }
        if call_count <= 1 {
            return true;
        }
        let expansion = (call_count - 1) * size;
        expansion <= Self::INLINE_MAX_EXPANSION
    }

    fn fresh_reg(next_reg: &mut usize) -> VReg {
        let r = VReg(*next_reg);
        *next_reg += 1;
        r
    }

    fn get_or_create_reg(old: VReg, reg_map: &mut HashMap<VReg, VReg>, next_reg: &mut usize) -> VReg {
        if let Some(&r) = reg_map.get(&old) { r } else {
            let nr = Self::fresh_reg(next_reg);
            reg_map.insert(old, nr);
            nr
        }
    }

    fn remap_reg(r: VReg, reg_map: &HashMap<VReg, VReg>) -> VReg {
        *reg_map.get(&r).unwrap_or(&r)
    }

    fn remap_block(b: BlockId, block_map: &HashMap<BlockId, BlockId>) -> BlockId {
        *block_map.get(&b).unwrap_or(&b)
    }

    /// Remap a callee instruction for inlining. Returns None for Ret (handled by caller).
    fn remap_callee_instruction(
        inst: &Instruction,
        reg_map: &mut HashMap<VReg, VReg>,
        block_map: &HashMap<BlockId, BlockId>,
        next_reg: &mut usize,
    ) -> Option<Instruction> {
        match inst {
            Instruction::ConstInt { dest, value } => {
                let d = Self::get_or_create_reg(*dest, reg_map, next_reg);
                Some(Instruction::ConstInt { dest: d, value: *value })
            }
            Instruction::ConstFloat { dest, value, ty } => {
                let d = Self::get_or_create_reg(*dest, reg_map, next_reg);
                Some(Instruction::ConstFloat { dest: d, value: *value, ty: ty.clone() })
            }
            Instruction::ConstBool { dest, value } => {
                let d = Self::get_or_create_reg(*dest, reg_map, next_reg);
                Some(Instruction::ConstBool { dest: d, value: *value })
            }
            Instruction::ConstString { dest, value } => {
                let d = Self::get_or_create_reg(*dest, reg_map, next_reg);
                Some(Instruction::ConstString { dest: d, value: value.clone() })
            }
            Instruction::Alloca { dest, name, ty } => {
                let d = Self::get_or_create_reg(*dest, reg_map, next_reg);
                Some(Instruction::Alloca { dest: d, name: name.clone(), ty: ty.clone() })
            }
            Instruction::AllocStruct { dest, class_name } => {
                let d = Self::get_or_create_reg(*dest, reg_map, next_reg);
                Some(Instruction::AllocStruct { dest: d, class_name: class_name.clone() })
            }
            Instruction::AllocArray { dest, size, ty } => {
                let d = Self::get_or_create_reg(*dest, reg_map, next_reg);
                let s = Self::remap_reg(*size, reg_map);
                Some(Instruction::AllocArray { dest: d, size: s, ty: ty.clone() })
            }
            Instruction::GetElementPtr { dest, base_ty, base_ptr, indices } => {
                let d = Self::get_or_create_reg(*dest, reg_map, next_reg);
                let bp = Self::remap_reg(*base_ptr, reg_map);
                let idx = indices.iter().map(|i| Self::remap_reg(*i, reg_map)).collect();
                Some(Instruction::GetElementPtr { dest: d, base_ty: base_ty.clone(), base_ptr: bp, indices: idx })
            }
            Instruction::Load { dest, ty, src_ptr } => {
                let d = Self::get_or_create_reg(*dest, reg_map, next_reg);
                let sp = Self::remap_reg(*src_ptr, reg_map);
                Some(Instruction::Load { dest: d, ty: ty.clone(), src_ptr: sp })
            }
            Instruction::Store { ty, value, ptr } => {
                let v = Self::remap_reg(*value, reg_map);
                let p = Self::remap_reg(*ptr, reg_map);
                Some(Instruction::Store { ty: ty.clone(), value: v, ptr: p })
            }
            Instruction::Cast { dest, value, source_ty, target_ty } => {
                let d = Self::get_or_create_reg(*dest, reg_map, next_reg);
                let v = Self::remap_reg(*value, reg_map);
                Some(Instruction::Cast { dest: d, value: v, source_ty: source_ty.clone(), target_ty: target_ty.clone() })
            }
            Instruction::Add { dest, left, right } => {
                let d = Self::get_or_create_reg(*dest, reg_map, next_reg);
                Some(Instruction::Add { dest: d, left: Self::remap_reg(*left, reg_map), right: Self::remap_reg(*right, reg_map) })
            }
            Instruction::SizeOf {dest,ty} => {
                let dest=Self::get_or_create_reg(*dest,reg_map,next_reg);
                Some(Instruction::SizeOf {dest,ty:ty.clone()})
            }
            Instruction::Sub { dest, left, right } => {
                let d = Self::get_or_create_reg(*dest, reg_map, next_reg);
                Some(Instruction::Sub { dest: d, left: Self::remap_reg(*left, reg_map), right: Self::remap_reg(*right, reg_map) })
            }
            Instruction::Mul { dest, left, right } => {
                let d = Self::get_or_create_reg(*dest, reg_map, next_reg);
                Some(Instruction::Mul { dest: d, left: Self::remap_reg(*left, reg_map), right: Self::remap_reg(*right, reg_map) })
            }
            Instruction::BitAnd {dest,left,right}|Instruction::BitOr {dest,left,right}=>{
                let dest=Self::get_or_create_reg(*dest,reg_map,next_reg);
                let left=Self::remap_reg(*left,reg_map);let right=Self::remap_reg(*right,reg_map);
                Some(if matches!(inst,Instruction::BitAnd {..}) {Instruction::BitAnd {dest,left,right}}else{Instruction::BitOr {dest,left,right}})
            }
            Instruction::Div { dest, left, right, ty } => {
                let d = Self::get_or_create_reg(*dest, reg_map, next_reg);
                Some(Instruction::Div { dest: d, left: Self::remap_reg(*left, reg_map), right: Self::remap_reg(*right, reg_map), ty: ty.clone() })
            }
            Instruction::Mod { dest, left, right, ty } => {
                let d = Self::get_or_create_reg(*dest, reg_map, next_reg);
                Some(Instruction::Mod { dest: d, left: Self::remap_reg(*left, reg_map), right: Self::remap_reg(*right, reg_map), ty: ty.clone() })
            }
            Instruction::CmpEq { dest, left, right } => {
                let d = Self::get_or_create_reg(*dest, reg_map, next_reg);
                Some(Instruction::CmpEq { dest: d, left: Self::remap_reg(*left, reg_map), right: Self::remap_reg(*right, reg_map) })
            }
            Instruction::CmpNeq { dest, left, right } => {
                let d = Self::get_or_create_reg(*dest, reg_map, next_reg);
                Some(Instruction::CmpNeq { dest: d, left: Self::remap_reg(*left, reg_map), right: Self::remap_reg(*right, reg_map) })
            }
            Instruction::CmpLt { dest, left, right, ty } => {
                let d = Self::get_or_create_reg(*dest, reg_map, next_reg);
                Some(Instruction::CmpLt { dest: d, left: Self::remap_reg(*left, reg_map), right: Self::remap_reg(*right, reg_map), ty: ty.clone() })
            }
            Instruction::CmpLe { dest, left, right, ty } => {
                let d = Self::get_or_create_reg(*dest, reg_map, next_reg);
                Some(Instruction::CmpLe { dest: d, left: Self::remap_reg(*left, reg_map), right: Self::remap_reg(*right, reg_map), ty: ty.clone() })
            }
            Instruction::CmpGt { dest, left, right, ty } => {
                let d = Self::get_or_create_reg(*dest, reg_map, next_reg);
                Some(Instruction::CmpGt { dest: d, left: Self::remap_reg(*left, reg_map), right: Self::remap_reg(*right, reg_map), ty: ty.clone() })
            }
            Instruction::CmpGe { dest, left, right, ty } => {
                let d = Self::get_or_create_reg(*dest, reg_map, next_reg);
                Some(Instruction::CmpGe { dest: d, left: Self::remap_reg(*left, reg_map), right: Self::remap_reg(*right, reg_map), ty: ty.clone() })
            }
            Instruction::Call { dest, func_name, args } => {
                let d = Self::get_or_create_reg(*dest, reg_map, next_reg);
                let a = args.iter().map(|a| Self::remap_reg(*a, reg_map)).collect();
                Some(Instruction::Call { dest: d, func_name: func_name.clone(), args: a })
            }
            Instruction::MakeFatPtr { dest, data_ptr, vtable_name } => {
                let d = Self::get_or_create_reg(*dest, reg_map, next_reg);
                let dp = Self::remap_reg(*data_ptr, reg_map);
                Some(Instruction::MakeFatPtr { dest: d, data_ptr: dp, vtable_name: vtable_name.clone() })
            }
            Instruction::DynamicCall { dest, vtable_index, fat_ptr, args, arg_types, ret_type } => {
                let d = Self::get_or_create_reg(*dest, reg_map, next_reg);
                let fp = Self::remap_reg(*fat_ptr, reg_map);
                let a = args.iter().map(|a| Self::remap_reg(*a, reg_map)).collect();
                Some(Instruction::DynamicCall { dest: d, vtable_index: *vtable_index, fat_ptr: fp, args: a, arg_types: arg_types.clone(), ret_type: ret_type.clone() })
            }
            Instruction::LoadFnPtr { dest, fn_name } => {
                let d = Self::get_or_create_reg(*dest, reg_map, next_reg);
                Some(Instruction::LoadFnPtr { dest: d, fn_name: fn_name.clone() })
            }
            Instruction::IndirectCall { dest, fn_ptr, args, arg_types, ret_type } => {
                let d = Self::get_or_create_reg(*dest, reg_map, next_reg);
                let fp = Self::remap_reg(*fn_ptr, reg_map);
                let a = args.iter().map(|a| Self::remap_reg(*a, reg_map)).collect();
                Some(Instruction::IndirectCall { dest: d, fn_ptr: fp, args: a, arg_types: arg_types.clone(), ret_type: ret_type.clone() })
            }
            Instruction::MakeClosure { dest, fn_name, env_ptr } => {
                let d = Self::get_or_create_reg(*dest, reg_map, next_reg);
                let ep = Self::remap_reg(*env_ptr, reg_map);
                Some(Instruction::MakeClosure { dest: d, fn_name: fn_name.clone(), env_ptr: ep })
            }
            Instruction::CallClosure { dest, closure_ptr, args, arg_types, ret_type } => {
                let d = Self::get_or_create_reg(*dest, reg_map, next_reg);
                let cp = Self::remap_reg(*closure_ptr, reg_map);
                let a = args.iter().map(|a| Self::remap_reg(*a, reg_map)).collect();
                Some(Instruction::CallClosure { dest: d, closure_ptr: cp, args: a, arg_types: arg_types.clone(), ret_type: ret_type.clone() })
            }
            Instruction::Retain { ptr } => Some(Instruction::Retain { ptr: Self::remap_reg(*ptr, reg_map) }),
            Instruction::Release { ptr } => Some(Instruction::Release { ptr: Self::remap_reg(*ptr, reg_map) }),
            Instruction::Br { target } => Some(Instruction::Br { target: Self::remap_block(*target, block_map) }),
            Instruction::CondBr { cond, if_true, if_false } => {
                Some(Instruction::CondBr {
                    cond: Self::remap_reg(*cond, reg_map),
                    if_true: Self::remap_block(*if_true, block_map),
                    if_false: Self::remap_block(*if_false, block_map),
                })
            }
            Instruction::Unreachable => Some(Instruction::Unreachable),
            Instruction::Ret { .. } => None, // handled by caller
        }
    }

    fn recursive_functions(module: &ModuleIr) -> HashSet<String> {
        let edges=module.functions.iter().map(|f| {
            let callees=f.blocks.iter().flat_map(|b|&b.instructions).filter_map(|i| {
                if let Instruction::Call {func_name,..}=i {Some(func_name.clone())}else{None}
            }).collect::<Vec<_>>();
            (f.name.clone(),callees)
        }).collect::<HashMap<_,_>>();
        let mut recursive=HashSet::new();
        for name in edges.keys() {
            let mut seen=HashSet::new();
            let mut work=edges[name].clone();
            while let Some(callee)=work.pop() {
                if &callee==name {recursive.insert(name.clone());break;}
                if seen.insert(callee.clone()) {
                    if let Some(next)=edges.get(&callee) {work.extend(next.iter().cloned());}
                }
            }
        }
        recursive
    }

    fn inline_register_map(target: &FunctionIr, args: &[VReg], next_reg: &mut usize) -> HashMap<VReg,VReg> {
        let mut registers=target.args.iter().zip(args).map(|((param,_),arg)|(*param,*arg)).collect::<HashMap<_,_>>();
        // Block listing order does not establish dominance. Every local value
        // must have an identity before copying any operand or return.
        for inst in target.blocks.iter().flat_map(|b|&b.instructions) {
            if let Some(dest)=inst.destination() {Self::get_or_create_reg(dest,&mut registers,next_reg);}
        }
        registers
    }

    fn inline_functions(module: &mut ModuleIr) -> bool {
        let mut changed = false;

        let mut next_reg = Self::compute_next_vreg(module);
        let mut next_block_id = Self::compute_next_block_id(module);

        let call_counts = Self::count_function_calls(module);
        let recursive=Self::recursive_functions(module);

        // build the inlinable set using cost heuristics
        let mut inlinables: HashMap<String, FunctionIr> = HashMap::new();
        for func in &module.functions {
            let count = call_counts.get(&func.name).copied().unwrap_or(0);
            if !recursive.contains(&func.name) && Self::should_inline(func, count) {
                inlinables.insert(func.name.clone(), func.clone());
            }
        }

        if inlinables.is_empty() {
            return false;
        }

        for func_idx in 0..module.functions.len() {
            let func_name = module.functions[func_idx].name.clone();

            // pass 1: inline single-block callees in-place
            for block in &mut module.functions[func_idx].blocks {
                let mut new_instructions = Vec::new();
                for inst in &block.instructions {
                    if let Instruction::Call { dest: call_dest, func_name: callee_name, args } = inst {
                        if callee_name != &func_name {
                            if let Some(target) = inlinables.get(callee_name) {
                                if target.blocks.len() == 1 {
                                    let mut reg_map=Self::inline_register_map(target,args,&mut next_reg);
                                    let empty_block_map: HashMap<BlockId, BlockId> = HashMap::new();
                                    for t_inst in &target.blocks[0].instructions {
                                        if let Instruction::Ret { value } = t_inst {
                                            if let Some(ret_val) = value {
                                                if target.ret_type != IrType::Void {
                                                    let mapped = Self::remap_reg(*ret_val, &reg_map);
                                                    new_instructions.push(Instruction::Cast {
                                                        dest: *call_dest,
                                                        value: mapped,
                                                        source_ty: target.ret_type.clone(),
                                                        target_ty: target.ret_type.clone(),
                                                    });
                                                }
                                            }
                                            continue;
                                        }
                                        if let Some(r) = Self::remap_callee_instruction(t_inst, &mut reg_map, &empty_block_map, &mut next_reg) {
                                            new_instructions.push(r);
                                        }
                                    }
                                    changed = true;
                                    // A diverging callee terminates the caller's block too.
                                    // Following instructions and successor uses are unreachable.
                                    if matches!(new_instructions.last(),Some(Instruction::Unreachable)) {break;}
                                    continue;
                                }
                            }
                        }
                    }
                    new_instructions.push(inst.clone());
                }
                block.instructions = new_instructions;
            }

            // pass 2: inline multi-block callees via block splitting
            loop {
                let mut found = None;
                'outer: for (bi, block) in module.functions[func_idx].blocks.iter().enumerate() {
                    for (ii, inst) in block.instructions.iter().enumerate() {
                        if let Instruction::Call { dest, func_name: callee_name, args } = inst {
                            if callee_name != &func_name {
                                if let Some(target) = inlinables.get(callee_name) {
                                    if target.blocks.len() > 1 {
                                        found = Some((bi, ii, *dest, callee_name.clone(), args.clone()));
                                        break 'outer;
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some((block_idx, inst_idx, call_dest, callee_name, args)) = found {
                    let target = inlinables[&callee_name].clone();

                    let mut reg_map=Self::inline_register_map(&target,&args,&mut next_reg);
                    let mut block_map: HashMap<BlockId, BlockId> = HashMap::new();
                    for callee_block in &target.blocks {
                        let new_id = BlockId(next_block_id);
                        next_block_id += 1;
                        block_map.insert(callee_block.id, new_id);
                    }

                    let cont_block_id = BlockId(next_block_id);
                    next_block_id += 1;

                    let ret_alloca = if target.ret_type != IrType::Void {
                        Some(Self::fresh_reg(&mut next_reg))
                    } else {
                        None
                    };

                    let orig_name = module.functions[func_idx].blocks[block_idx].name.clone();
                    let pre_insts: Vec<Instruction> = module.functions[func_idx].blocks[block_idx].instructions[..inst_idx].to_vec();
                    let post_insts: Vec<Instruction> = module.functions[func_idx].blocks[block_idx].instructions[inst_idx + 1..].to_vec();

                    let first_inlined_id = block_map[&target.blocks[0].id];
                    let mut pre_block_insts = pre_insts;
                    if let Some(ra) = ret_alloca {
                        pre_block_insts.push(Instruction::Alloca { dest: ra, name: "__inline_ret".to_string(), ty: target.ret_type.clone() });
                    }
                    pre_block_insts.push(Instruction::Br { target: first_inlined_id });
                    module.functions[func_idx].blocks[block_idx].instructions = pre_block_insts;

                    let mut inlined_blocks = Vec::new();
                    for callee_block in &target.blocks {
                        let new_block_id = block_map[&callee_block.id];
                        let mut new_insts = Vec::new();
                        for t_inst in &callee_block.instructions {
                            if let Instruction::Ret { value } = t_inst {
                                if let (Some(ret_val), Some(ra)) = (value, ret_alloca) {
                                    let mapped = Self::remap_reg(*ret_val, &reg_map);
                                    new_insts.push(Instruction::Store { ty: target.ret_type.clone(), value: mapped, ptr: ra });
                                }
                                new_insts.push(Instruction::Br { target: cont_block_id });
                                continue;
                            }
                            if let Some(r) = Self::remap_callee_instruction(t_inst, &mut reg_map, &block_map, &mut next_reg) {
                                new_insts.push(r);
                            }
                        }
                        inlined_blocks.push(BasicBlock { id: new_block_id, name: format!("inline_{}", callee_block.name), instructions: new_insts });
                    }

                    let mut cont_insts = Vec::new();
                    if let Some(ra) = ret_alloca {
                        cont_insts.push(Instruction::Load { dest: call_dest, ty: target.ret_type.clone(), src_ptr: ra });
                    }
                    cont_insts.extend(post_insts);
                    inlined_blocks.push(BasicBlock { id: cont_block_id, name: format!("{}_cont", orig_name), instructions: cont_insts });

                    let insert_pos = block_idx + 1;
                    for (i, blk) in inlined_blocks.into_iter().enumerate() {
                        module.functions[func_idx].blocks.insert(insert_pos + i, blk);
                    }
                    changed = true;
                } else {
                    break;
                }
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
            "malloc",
            "free",
            "realloc",
            "printf",
            "exit",
            "strlen",
            "strcat",
            "memcpy",
            "fopen",
            "fclose",
            "fputs",
            "fprintf",
            "panic"
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
                            crate::ir::Instruction::Call { func_name: target, .. } => {
                                worklist.push(target.clone());
                            }

                            // closure creation
                            crate::ir::Instruction::MakeClosure { fn_name: target, .. } => {
                                worklist.push(target.clone());
                            }

                            // passing function pointers
                            crate::ir::Instruction::LoadFnPtr { fn_name: target, .. } => {
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inlining_divergence_discards_the_unreachable_caller_tail() {
        let mut main = function(vec![
            Instruction::Call { dest: VReg(0), func_name: "stop".into(), args: vec![] },
            Instruction::Ret { value: Some(VReg(0)) },
        ]);
        main.name = "main".into();
        let mut stop = function(vec![
            Instruction::Call { dest: VReg(1), func_name: "sink".into(), args: vec![] },
            Instruction::Unreachable,
        ]);
        stop.name = "stop".into();
        let mut sink = function(vec![]);
        sink.name = "sink".into();
        sink.ret_type = IrType::Void;
        sink.blocks.clear();
        let mut module = ModuleIr {
            name: "test".into(), globals: HashMap::new(), structs: HashMap::new(),
            vtables: HashMap::new(), functions: vec![main, stop, sink],
        };
        crate::ir_verify::verify(&module).unwrap();
        IrOptimizer::optimize(&mut module);
        crate::ir_verify::verify(&module).unwrap();
        let main = module.functions.iter().find(|f| f.name == "main").unwrap();
        assert!(matches!(main.blocks[0].instructions.last(), Some(Instruction::Unreachable)));
    }

    #[test]
    fn bitwise_operations_keep_operands_through_inlining_and_folding() {
        let instruction = Instruction::BitAnd { dest: VReg(2), left: VReg(0), right: VReg(1) };
        let mut registers = HashMap::from([(VReg(0), VReg(10)), (VReg(1), VReg(11))]);
        let mut next = 12;
        let mapped = IrOptimizer::remap_callee_instruction(&instruction, &mut registers, &HashMap::new(), &mut next).unwrap();
        assert_eq!(mapped.operands(), vec![VReg(10), VReg(11)]);
        assert_eq!(mapped.destination(), Some(VReg(12)));
        let mut f = function(vec![
            Instruction::ConstInt { dest: VReg(10), value: -2 },
            Instruction::ConstInt { dest: VReg(11), value: 3 },
            mapped,
            Instruction::BitOr { dest: VReg(13), left: VReg(10), right: VReg(11) },
        ]);
        IrOptimizer::fold_constants(&mut f);
        assert!(matches!(f.blocks[0].instructions[2], Instruction::ConstInt { value: 2, .. }));
        assert!(matches!(f.blocks[0].instructions[3], Instruction::ConstInt { value: -1, .. }));
    }
    #[test]
    fn storage_size_preserves_layout_through_inlining_and_cse() {
        let layout = IrType::Struct("__fishy_range_i64".into(), vec![IrType::I64; 2]);
        let mut size = function(vec![
            Instruction::SizeOf { dest: VReg(0), ty: layout.clone() },
            Instruction::Ret { value: Some(VReg(0)) },
        ]);
        size.name = "stride".into();
        size.ret_type = IrType::U64;
        let mut main = function(vec![
            Instruction::Call { dest: VReg(1), func_name: "stride".into(), args: vec![] },
            Instruction::SizeOf { dest: VReg(2), ty: layout.clone() },
            Instruction::Add { dest: VReg(3), left: VReg(1), right: VReg(2) },
            Instruction::Ret { value: Some(VReg(3)) },
        ]);
        main.name = "main".into();
        main.ret_type = IrType::U64;
        let mut module = ModuleIr {
            name: "layout".into(), globals: HashMap::new(), structs: HashMap::new(),
            vtables: HashMap::new(), functions: vec![main, size],
        };
        IrOptimizer::optimize(&mut module);
        crate::ir_verify::verify(&module).unwrap();
        let instructions = &module.functions.iter().find(|f| f.name == "main").unwrap().blocks[0].instructions;
        assert_eq!(instructions.iter().filter(|i| matches!(i, Instruction::SizeOf {ty,..} if ty == &layout)).count(), 1);
        assert!(instructions.iter().any(|i| matches!(i, Instruction::Add {left,right,..} if left == right)));
    }

    fn function(instructions:Vec<Instruction>)->FunctionIr {
        FunctionIr {name:"test".into(),ret_type:IrType::I64,args:vec![],blocks:vec![BasicBlock {id:BlockId(0),name:"entry".into(),instructions}],is_variadic:false}
    }
    #[test]
    fn constant_fold_wraps_without_host_panic() {
        let mut f=function(vec![Instruction::ConstInt {dest:VReg(0),value:i64::MAX},Instruction::ConstInt {dest:VReg(1),value:1},Instruction::Add {dest:VReg(2),left:VReg(0),right:VReg(1)}]);
        IrOptimizer::fold_constants(&mut f);
        assert!(matches!(f.blocks[0].instructions[2],Instruction::ConstInt {value:i64::MIN,..}));
    }
    #[test]
    fn division_poison_is_not_host_panic_or_constant() {
        for right in [0,-1] {
            let mut f=function(vec![Instruction::ConstInt {dest:VReg(0),value:i64::MIN},Instruction::ConstInt {dest:VReg(1),value:right},Instruction::Div {dest:VReg(2),left:VReg(0),right:VReg(1),ty:IrType::I64}]);
            IrOptimizer::fold_constants(&mut f);assert!(matches!(f.blocks[0].instructions[2],Instruction::Div {..}));
        }
    }
    #[test]
    fn cast_alias_chains_update_return_transitively() {
        let mut f=function(vec![Instruction::ConstInt {dest:VReg(0),value:7},Instruction::Cast {dest:VReg(1),value:VReg(0),source_ty:IrType::I64,target_ty:IrType::I64},Instruction::Cast {dest:VReg(2),value:VReg(1),source_ty:IrType::I64,target_ty:IrType::I64},Instruction::Ret {value:Some(VReg(2))}]);
        IrOptimizer::simplify_identity_casts(&mut f);
        assert!(matches!(f.blocks[0].instructions.last(),Some(Instruction::Ret {value:Some(VReg(0))})));
    }
    #[test]
    fn cse_remaps_successor_uses_and_preserves_signedness() {
        let mut f=function(vec![Instruction::ConstInt {dest:VReg(0),value:-1},Instruction::ConstInt {dest:VReg(1),value:2},Instruction::Div {dest:VReg(2),left:VReg(0),right:VReg(1),ty:IrType::I64},Instruction::Div {dest:VReg(3),left:VReg(0),right:VReg(1),ty:IrType::U64},Instruction::Div {dest:VReg(4),left:VReg(0),right:VReg(1),ty:IrType::I64},Instruction::Br {target:BlockId(1)}]);
        f.blocks.push(BasicBlock {id:BlockId(1),name:"exit".into(),instructions:vec![Instruction::Ret {value:Some(VReg(4))}]});
        assert!(IrOptimizer::eliminate_common_subexpressions(&mut f));
        assert!(f.blocks[0].instructions.iter().any(|i|matches!(i,Instruction::Div {dest:VReg(3),..})));
        assert!(matches!(f.blocks[1].instructions[0],Instruction::Ret {value:Some(VReg(2))}));
    }
    #[test]
    fn escaping_alloca_and_store_remain_live() {
        let mut f=function(vec![Instruction::Alloca {dest:VReg(0),name:"x".into(),ty:IrType::I64},Instruction::ConstInt {dest:VReg(1),value:7},Instruction::Store {ty:IrType::I64,value:VReg(1),ptr:VReg(0)},Instruction::Cast {dest:VReg(2),value:VReg(0),source_ty:IrType::Ptr(Box::new(IrType::I64)),target_ty:IrType::I64},Instruction::Ret {value:Some(VReg(2))}]);
        IrOptimizer::eliminate_dead_code(&mut f);assert_eq!(f.blocks[0].instructions.len(),5);
    }
}

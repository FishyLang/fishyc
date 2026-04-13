use crate::ir::{ BlockId, FunctionIr, Instruction, IrType, ModuleIr, VReg };
use std::collections::{ HashMap, HashSet };

#[derive(Clone)]
enum ConstValue {
    Int(i64),
    Float(f64),
    Bool(bool),
}

#[derive(Clone, PartialEq, Eq, Hash)]
enum ExprKey {
    Add(VReg, VReg),
    Sub(VReg, VReg),
    Mul(VReg, VReg),
    Div(VReg, VReg),
    Mod(VReg, VReg),
    CmpEq(VReg, VReg),
    CmpNeq(VReg, VReg),
    CmpLt(VReg, VReg),
    CmpLe(VReg, VReg),
    CmpGt(VReg, VReg),
    CmpGe(VReg, VReg),
    LoadFnPtr(String),
}

impl ConstValue {
    fn matches_ty(&self, ty: &IrType) -> bool {
        match (self, ty) {
            | (ConstValue::Int(_), IrType::I64)
            | (ConstValue::Int(_), IrType::I32)
            | (ConstValue::Int(_), IrType::I16)
            | (ConstValue::Int(_), IrType::I8)
            | (ConstValue::Int(_), IrType::Any)
            | (ConstValue::Float(_), IrType::F64)
            | (ConstValue::Float(_), IrType::F32)
            | (ConstValue::Float(_), IrType::F16)
            | (ConstValue::Float(_), IrType::Any)
            | (ConstValue::Bool(_), IrType::Bool)
            | (ConstValue::Bool(_), IrType::Any) => true,

            _ => false,
        }
    }

    fn to_instruction(&self, dest: VReg, ty: &IrType) -> Instruction {
        match self {
            ConstValue::Int(value) => Instruction::ConstInt { dest, value: *value },

            ConstValue::Float(value) =>
                Instruction::ConstFloat {
                    dest,
                    value: *value,
                    ty: ty.clone(),
                },

            ConstValue::Bool(value) => Instruction::ConstBool { dest, value: *value },
        }
    }
}

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

    // --- stage 1: constant folding & load forwarding ---
    fn fold_constants(func: &mut FunctionIr) -> bool {
        let mut changed = false;
        for block in &mut func.blocks {
            let mut new_instructions = Vec::new();

            let mut known_ints = HashMap::new();
            let mut known_bools = HashMap::new();
            let mut known_floats = HashMap::new();
            let mut known_ptr_consts: HashMap<VReg, ConstValue> = HashMap::new();

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
                        if let Some(const_val) = known_ptr_consts.get(src_ptr) {
                            if const_val.matches_ty(ty) {
                                optimized_inst = const_val.to_instruction(*dest, ty);
                                match optimized_inst {
                                    Instruction::ConstInt { value, .. } => {
                                        known_ints.insert(*dest, value);
                                    }

                                    Instruction::ConstFloat { value, .. } => {
                                        known_floats.insert(*dest, value);
                                    }

                                    Instruction::ConstBool { value, .. } => {
                                        known_bools.insert(*dest, value);
                                    }

                                    _ => {}
                                }
                                changed = true;
                            }
                        } else if let Some(&val) = known_ints.get(src_ptr) {
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

                    Instruction::Store { ptr, value, .. } => {
                        if let Some(&val) = known_ints.get(value) {
                            known_ptr_consts.insert(*ptr, ConstValue::Int(val));
                        } else if let Some(&val) = known_floats.get(value) {
                            known_ptr_consts.insert(*ptr, ConstValue::Float(val));
                        } else if let Some(&val) = known_bools.get(value) {
                            known_ptr_consts.insert(*ptr, ConstValue::Bool(val));
                        } else {
                            known_ptr_consts.remove(ptr);
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
                        } else if
                            let (Some(&l), Some(&r)) = (
                                known_floats.get(left),
                                known_floats.get(right),
                            )
                        {
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
                        } else if
                            let (Some(&l), Some(&r)) = (
                                known_floats.get(left),
                                known_floats.get(right),
                            )
                        {
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
                        } else if
                            let (Some(&l), Some(&r)) = (
                                known_floats.get(left),
                                known_floats.get(right),
                            )
                        {
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
                        } else if
                            let (Some(&l), Some(&r)) = (
                                known_floats.get(left),
                                known_floats.get(right),
                            )
                        {
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
                        } else if
                            let (Some(&l), Some(&r)) = (
                                known_floats.get(left),
                                known_floats.get(right),
                            )
                        {
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
                        } else if
                            let (Some(&l), Some(&r)) = (known_ints.get(left), known_ints.get(right))
                        {
                            let result = l < r;
                            optimized_inst = Instruction::ConstBool {
                                dest: *dest,
                                value: result,
                            };

                            known_bools.insert(*dest, result);
                            changed = true;
                        } else if
                            let (Some(&l), Some(&r)) = (
                                known_floats.get(left),
                                known_floats.get(right),
                            )
                        {
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
                        } else if
                            let (Some(&l), Some(&r)) = (known_ints.get(left), known_ints.get(right))
                        {
                            let result = l > r;
                            optimized_inst = Instruction::ConstBool {
                                dest: *dest,
                                value: result,
                            };

                            known_bools.insert(*dest, result);
                            changed = true;
                        } else if
                            let (Some(&l), Some(&r)) = (
                                known_floats.get(left),
                                known_floats.get(right),
                            )
                        {
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
                        } else if
                            let (Some(&l), Some(&r)) = (known_ints.get(left), known_ints.get(right))
                        {
                            let result = l <= r;
                            optimized_inst = Instruction::ConstBool {
                                dest: *dest,
                                value: result,
                            };

                            known_bools.insert(*dest, result);
                            changed = true;
                        } else if
                            let (Some(&l), Some(&r)) = (
                                known_floats.get(left),
                                known_floats.get(right),
                            )
                        {
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
                        } else if
                            let (Some(&l), Some(&r)) = (known_ints.get(left), known_ints.get(right))
                        {
                            let result = l >= r;
                            optimized_inst = Instruction::ConstBool {
                                dest: *dest,
                                value: result,
                            };

                            known_bools.insert(*dest, result);
                            changed = true;
                        } else if
                            let (Some(&l), Some(&r)) = (
                                known_floats.get(left),
                                known_floats.get(right),
                            )
                        {
                            let result = l >= r;
                            optimized_inst = Instruction::ConstBool {
                                dest: *dest,
                                value: result,
                            };

                            known_bools.insert(*dest, result);
                            changed = true;
                        }
                    }

                    Instruction::CondBr { cond, if_true, if_false } => {
                        if let Some(&cond_val) = known_bools.get(cond) {
                            let target = if cond_val { *if_true } else { *if_false };
                            optimized_inst = Instruction::Br { target };
                            changed = true;
                        }
                    }

                    Instruction::Cast { dest, value, target_ty } => {
                        if let Some(&val) = known_ints.get(value) {
                            if
                                matches!(
                                    target_ty,
                                    &IrType::I64 |
                                        &IrType::I32 |
                                        &IrType::I16 |
                                        &IrType::I8 |
                                        &IrType::Any
                                )
                            {
                                optimized_inst = Instruction::ConstInt {
                                    dest: *dest,
                                    value: val,
                                };

                                known_ints.insert(*dest, val);
                                changed = true;
                            } else if
                                matches!(target_ty, &IrType::F64 | &IrType::F32 | &IrType::F16)
                            {
                                optimized_inst = Instruction::ConstFloat {
                                    dest: *dest,
                                    value: val as f64,
                                    ty: IrType::F64,
                                };

                                known_floats.insert(*dest, val as f64);
                                changed = true;
                            }
                        } else if let Some(&val) = known_floats.get(value) {
                            if
                                matches!(
                                    target_ty,
                                    &IrType::F64 | &IrType::F32 | &IrType::F16 | &IrType::Any
                                )
                            {
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
        let mut changed = false;

        for block in &mut func.blocks {
            let mut new_instructions = Vec::new();
            let mut known_ints = HashMap::new();
            let mut known_floats = HashMap::new();
            let mut known_bools = HashMap::new();
            let mut alias_map = HashMap::new();

            // Agora passamos o map como argumento para não o "prender" (capture) permanentemente no closure
            let resolve = |mut reg: VReg, map: &HashMap<VReg, VReg>| -> VReg {
                while let Some(&alias) = map.get(&reg) {
                    reg = alias;
                }
                reg
            };

            for inst in &block.instructions {
                let inst = inst.clone();
                match inst {
                    Instruction::ConstInt { dest, value } => {
                        known_ints.insert(dest, value);
                        new_instructions.push(Instruction::ConstInt { dest, value });
                    }

                    Instruction::ConstFloat { dest, value, ty } => {
                        known_floats.insert(dest, value);
                        new_instructions.push(Instruction::ConstFloat { dest, value, ty });
                    }

                    Instruction::ConstBool { dest, value } => {
                        known_bools.insert(dest, value);
                        new_instructions.push(Instruction::ConstBool { dest, value });
                    }

                    Instruction::Alloca { dest, name, ty } => {
                        new_instructions.push(Instruction::Alloca { dest, name, ty });
                    }

                    Instruction::Add { dest, left, right } => {
                        let left = resolve(left, &alias_map);
                        let right = resolve(right, &alias_map);

                        if let Some(0) = known_ints.get(&left).copied() {
                            alias_map.insert(dest, right);
                            changed = true;
                            continue;
                        }

                        if let Some(0) = known_ints.get(&right).copied() {
                            alias_map.insert(dest, left);
                            changed = true;
                            continue;
                        }

                        if let Some(1) = known_ints.get(&left).copied() {
                            alias_map.insert(dest, right);
                            changed = true;
                            continue;
                        }

                        if let Some(1) = known_ints.get(&right).copied() {
                            alias_map.insert(dest, left);
                            changed = true;
                            continue;
                        }

                        if let Some(0.0) = known_floats.get(&left).copied() {
                            alias_map.insert(dest, right);
                            changed = true;
                            continue;
                        }

                        if let Some(0.0) = known_floats.get(&right).copied() {
                            alias_map.insert(dest, left);
                            changed = true;
                            continue;
                        }

                        if let Some(1.0) = known_floats.get(&left).copied() {
                            alias_map.insert(dest, right);
                            changed = true;
                            continue;
                        }

                        if let Some(1.0) = known_floats.get(&right).copied() {
                            alias_map.insert(dest, left);
                            changed = true;
                            continue;
                        }

                        new_instructions.push(Instruction::Add { dest, left, right });
                    }

                    Instruction::Sub { dest, left, right } => {
                        let left = resolve(left, &alias_map);
                        let right = resolve(right, &alias_map);

                        if let Some(0) = known_ints.get(&right).copied() {
                            alias_map.insert(dest, left);
                            changed = true;
                            continue;
                        }

                        if let Some(0.0) = known_floats.get(&right).copied() {
                            alias_map.insert(dest, left);
                            changed = true;
                            continue;
                        }

                        if left == right {
                            if known_ints.contains_key(&left) {
                                new_instructions.push(Instruction::ConstInt { dest, value: 0 });
                                known_ints.insert(dest, 0);
                                changed = true;
                                continue;
                            }

                            if known_floats.contains_key(&left) {
                                new_instructions.push(Instruction::ConstFloat {
                                    dest,
                                    value: 0.0,
                                    ty: IrType::F64,
                                });

                                known_floats.insert(dest, 0.0);
                                changed = true;
                                continue;
                            }
                        }

                        new_instructions.push(Instruction::Sub { dest, left, right });
                    }

                    Instruction::Mul { dest, left, right } => {
                        let left = resolve(left, &alias_map);
                        let right = resolve(right, &alias_map);

                        if let Some(0) = known_ints.get(&left).copied() {
                            new_instructions.push(Instruction::ConstInt { dest, value: 0 });
                            known_ints.insert(dest, 0);
                            changed = true;
                            continue;
                        }

                        if let Some(0) = known_ints.get(&right).copied() {
                            new_instructions.push(Instruction::ConstInt { dest, value: 0 });
                            known_ints.insert(dest, 0);
                            changed = true;
                            continue;
                        }

                        if let Some(1) = known_ints.get(&left).copied() {
                            alias_map.insert(dest, right);
                            changed = true;
                            continue;
                        }

                        if let Some(1) = known_ints.get(&right).copied() {
                            alias_map.insert(dest, left);
                            changed = true;
                            continue;
                        }

                        if let Some(0.0) = known_floats.get(&left).copied() {
                            new_instructions.push(Instruction::ConstFloat {
                                dest,
                                value: 0.0,
                                ty: IrType::F64,
                            });

                            known_floats.insert(dest, 0.0);
                            changed = true;
                            continue;
                        }

                        if let Some(0.0) = known_floats.get(&right).copied() {
                            new_instructions.push(Instruction::ConstFloat {
                                dest,
                                value: 0.0,
                                ty: IrType::F64,
                            });

                            known_floats.insert(dest, 0.0);
                            changed = true;
                            continue;
                        }

                        if let Some(1.0) = known_floats.get(&left).copied() {
                            alias_map.insert(dest, right);
                            changed = true;
                            continue;
                        }

                        if let Some(1.0) = known_floats.get(&right).copied() {
                            alias_map.insert(dest, left);
                            changed = true;
                            continue;
                        }

                        new_instructions.push(Instruction::Mul { dest, left, right });
                    }

                    Instruction::Div { dest, left, right } => {
                        let left = resolve(left, &alias_map);
                        let right = resolve(right, &alias_map);

                        if let Some(1) = known_ints.get(&right).copied() {
                            alias_map.insert(dest, left);
                            changed = true;
                            continue;
                        }

                        if let Some(1.0) = known_floats.get(&right).copied() {
                            alias_map.insert(dest, left);
                            changed = true;
                            continue;
                        }

                        new_instructions.push(Instruction::Div { dest, left, right });
                    }

                    Instruction::Mod { dest, left, right } => {
                        let left = resolve(left, &alias_map);
                        let right = resolve(right, &alias_map);

                        if let Some(1) = known_ints.get(&right).copied() {
                            new_instructions.push(Instruction::ConstInt { dest, value: 0 });
                            known_ints.insert(dest, 0);
                            changed = true;
                            continue;
                        }

                        if let Some(1.0) = known_floats.get(&right).copied() {
                            new_instructions.push(Instruction::ConstFloat {
                                dest,
                                value: 0.0,
                                ty: IrType::F64,
                            });

                            known_floats.insert(dest, 0.0);
                            changed = true;
                            continue;
                        }

                        if left == right {
                            if known_ints.contains_key(&left) {
                                new_instructions.push(Instruction::ConstInt { dest, value: 0 });
                                known_ints.insert(dest, 0);
                                changed = true;
                                continue;
                            }

                            if known_floats.contains_key(&left) {
                                new_instructions.push(Instruction::ConstFloat {
                                    dest,
                                    value: 0.0,
                                    ty: IrType::F64,
                                });

                                known_floats.insert(dest, 0.0);
                                changed = true;
                                continue;
                            }
                        }

                        new_instructions.push(Instruction::Mod { dest, left, right });
                    }

                    Instruction::CmpEq { dest, left, right } => {
                        let left = resolve(left, &alias_map);
                        let right = resolve(right, &alias_map);

                        if left == right {
                            new_instructions.push(Instruction::ConstBool { dest, value: true });
                            known_bools.insert(dest, true);
                            changed = true;
                            continue;
                        }

                        new_instructions.push(Instruction::CmpEq { dest, left, right });
                    }

                    Instruction::CmpNeq { dest, left, right } => {
                        let left = resolve(left, &alias_map);
                        let right = resolve(right, &alias_map);

                        if left == right {
                            new_instructions.push(Instruction::ConstBool { dest, value: false });
                            known_bools.insert(dest, false);
                            changed = true;
                            continue;
                        }

                        new_instructions.push(Instruction::CmpNeq { dest, left, right });
                    }

                    Instruction::CmpLt { dest, left, right } => {
                        let left = resolve(left, &alias_map);
                        let right = resolve(right, &alias_map);

                        if left == right {
                            new_instructions.push(Instruction::ConstBool { dest, value: false });
                            known_bools.insert(dest, false);
                            changed = true;
                            continue;
                        }

                        new_instructions.push(Instruction::CmpLt { dest, left, right });
                    }

                    Instruction::CmpLe { dest, left, right } => {
                        let left = resolve(left, &alias_map);
                        let right = resolve(right, &alias_map);

                        if left == right {
                            new_instructions.push(Instruction::ConstBool { dest, value: true });
                            known_bools.insert(dest, true);
                            changed = true;
                            continue;
                        }

                        new_instructions.push(Instruction::CmpLe { dest, left, right });
                    }

                    Instruction::CmpGt { dest, left, right } => {
                        let left = resolve(left, &alias_map);
                        let right = resolve(right, &alias_map);

                        if left == right {
                            new_instructions.push(Instruction::ConstBool { dest, value: false });
                            known_bools.insert(dest, false);
                            changed = true;
                            continue;
                        }

                        new_instructions.push(Instruction::CmpGt { dest, left, right });
                    }

                    Instruction::CmpGe { dest, left, right } => {
                        let left = resolve(left, &alias_map);
                        let right = resolve(right, &alias_map);

                        if left == right {
                            new_instructions.push(Instruction::ConstBool { dest, value: true });
                            known_bools.insert(dest, true);
                            changed = true;
                            continue;
                        }

                        new_instructions.push(Instruction::CmpGe { dest, left, right });
                    }

                    Instruction::Cast { dest, value, target_ty } => {
                        let value = resolve(value, &alias_map);
                        new_instructions.push(Instruction::Cast { dest, value, target_ty });
                    }

                    Instruction::CondBr { cond, if_true, if_false } => {
                        let cond = resolve(cond, &alias_map);
                        new_instructions.push(Instruction::CondBr { cond, if_true, if_false });
                    }

                    Instruction::Load { dest, ty, src_ptr } => {
                        let src_ptr = resolve(src_ptr, &alias_map);
                        new_instructions.push(Instruction::Load { dest, ty, src_ptr });
                    }

                    Instruction::Store { ty, value, ptr } => {
                        let value = resolve(value, &alias_map);
                        let ptr = resolve(ptr, &alias_map);
                        new_instructions.push(Instruction::Store { ty, value, ptr });
                    }

                    Instruction::GetElementPtr { dest, base_ty, base_ptr, indices } => {
                        let base_ptr = resolve(base_ptr, &alias_map);
                        let indices = indices
                            .into_iter()
                            .map(|idx| resolve(idx, &alias_map))
                            .collect();

                        new_instructions.push(Instruction::GetElementPtr {
                            dest,
                            base_ty,
                            base_ptr,
                            indices,
                        });
                    }

                    Instruction::MakeFatPtr { dest, data_ptr, vtable_name } => {
                        let data_ptr = resolve(data_ptr, &alias_map);
                        new_instructions.push(Instruction::MakeFatPtr {
                            dest,
                            data_ptr,
                            vtable_name,
                        });
                    }

                    Instruction::DynamicCall {
                        dest,
                        vtable_index,
                        fat_ptr,
                        args,
                        arg_types,
                        ret_type,
                    } => {
                        let fat_ptr = resolve(fat_ptr, &alias_map);
                        let args = args
                            .into_iter()
                            .map(|arg| resolve(arg, &alias_map))
                            .collect();

                        new_instructions.push(Instruction::DynamicCall {
                            dest,
                            vtable_index,
                            fat_ptr,
                            args,
                            arg_types,
                            ret_type,
                        });
                    }

                    Instruction::CallClosure { dest, closure_ptr, args, arg_types, ret_type } => {
                        let closure_ptr = resolve(closure_ptr, &alias_map);
                        let args = args
                            .into_iter()
                            .map(|arg| resolve(arg, &alias_map))
                            .collect();

                        new_instructions.push(Instruction::CallClosure {
                            dest,
                            closure_ptr,
                            args,
                            arg_types,
                            ret_type,
                        });
                    }

                    Instruction::IndirectCall { dest, fn_ptr, args, arg_types, ret_type } => {
                        let fn_ptr = resolve(fn_ptr, &alias_map);
                        let args = args
                            .into_iter()
                            .map(|arg| resolve(arg, &alias_map))
                            .collect();

                        new_instructions.push(Instruction::IndirectCall {
                            dest,
                            fn_ptr,
                            args,
                            arg_types,
                            ret_type,
                        });
                    }

                    Instruction::Call { dest, func_name, args } => {
                        let args = args
                            .into_iter()
                            .map(|arg| resolve(arg, &alias_map))
                            .collect();
                        new_instructions.push(Instruction::Call { dest, func_name, args });
                    }

                    Instruction::MakeClosure { dest, fn_name, env_ptr } => {
                        let env_ptr = resolve(env_ptr, &alias_map);
                        new_instructions.push(Instruction::MakeClosure { dest, fn_name, env_ptr });
                    }

                    Instruction::Retain { ptr } => {
                        let ptr = resolve(ptr, &alias_map);
                        new_instructions.push(Instruction::Retain { ptr });
                    }

                    Instruction::Release { ptr } => {
                        let ptr = resolve(ptr, &alias_map);
                        new_instructions.push(Instruction::Release { ptr });
                    }

                    Instruction::Br { target } => {
                        new_instructions.push(Instruction::Br { target });
                    }

                    Instruction::Ret { value } => {
                        let value = value.map(|v| resolve(v, &alias_map));
                        new_instructions.push(Instruction::Ret { value });
                    }

                    Instruction::AllocArray { dest, size, ty } => {
                        let size = resolve(size, &alias_map);
                        new_instructions.push(Instruction::AllocArray { dest, size, ty });
                    }

                    Instruction::AllocStruct { dest, class_name, size } => {
                        new_instructions.push(Instruction::AllocStruct { dest, class_name, size });
                    }

                    Instruction::LoadFnPtr { dest, fn_name } => {
                        new_instructions.push(Instruction::LoadFnPtr { dest, fn_name });
                    }

                    Instruction::ConstString { dest, value } => {
                        new_instructions.push(Instruction::ConstString { dest, value });
                    }

                    Instruction::Unreachable => {
                        new_instructions.push(Instruction::Unreachable);
                    }
                }
            }

            block.instructions = new_instructions;
        }

        changed
    }

    fn eliminate_common_subexpressions(func: &mut FunctionIr) -> bool {
        let mut changed = false;

        for block in &mut func.blocks {
            let mut expr_map = HashMap::new();
            let mut alias_map = HashMap::new();
            let mut new_instructions = Vec::new();

            fn resolve_alias(alias_map: &HashMap<VReg, VReg>, mut reg: VReg) -> VReg {
                while let Some(&alias) = alias_map.get(&reg) {
                    reg = alias;
                }

                reg
            }

            for inst in &block.instructions {
                let inst = inst.clone();
                match inst {
                    Instruction::Add { dest, left, right } => {
                        let left = resolve_alias(&alias_map, left);
                        let right = resolve_alias(&alias_map, right);

                        let key = if left.0 <= right.0 {
                            ExprKey::Add(left, right)
                        } else {
                            ExprKey::Add(right, left)
                        };

                        if let Some(&existing) = expr_map.get(&key) {
                            alias_map.insert(dest, existing);
                            changed = true;
                            continue;
                        }

                        expr_map.insert(key, dest);
                        new_instructions.push(Instruction::Add { dest, left, right });
                    }
                    Instruction::Mul { dest, left, right } => {
                        let left = resolve_alias(&alias_map, left);
                        let right = resolve_alias(&alias_map, right);

                        let key = if left.0 <= right.0 {
                            ExprKey::Mul(left, right)
                        } else {
                            ExprKey::Mul(right, left)
                        };

                        if let Some(&existing) = expr_map.get(&key) {
                            alias_map.insert(dest, existing);
                            changed = true;
                            continue;
                        }

                        expr_map.insert(key, dest);
                        new_instructions.push(Instruction::Mul { dest, left, right });
                    }
                    Instruction::CmpEq { dest, left, right } => {
                        let left = resolve_alias(&alias_map, left);
                        let right = resolve_alias(&alias_map, right);

                        let key = if left.0 <= right.0 {
                            ExprKey::CmpEq(left, right)
                        } else {
                            ExprKey::CmpEq(right, left)
                        };

                        if let Some(&existing) = expr_map.get(&key) {
                            alias_map.insert(dest, existing);
                            changed = true;
                            continue;
                        }

                        expr_map.insert(key, dest);
                        new_instructions.push(Instruction::CmpEq { dest, left, right });
                    }

                    Instruction::CmpNeq { dest, left, right } => {
                        let left = resolve_alias(&alias_map, left);
                        let right = resolve_alias(&alias_map, right);

                        let key = if left.0 <= right.0 {
                            ExprKey::CmpNeq(left, right)
                        } else {
                            ExprKey::CmpNeq(right, left)
                        };

                        if let Some(&existing) = expr_map.get(&key) {
                            alias_map.insert(dest, existing);
                            changed = true;
                            continue;
                        }

                        expr_map.insert(key, dest);
                        new_instructions.push(Instruction::CmpNeq { dest, left, right });
                    }

                    Instruction::CmpLt { dest, left, right } => {
                        let left = resolve_alias(&alias_map, left);
                        let right = resolve_alias(&alias_map, right);
                        let key = ExprKey::CmpLt(left, right);

                        if let Some(&existing) = expr_map.get(&key) {
                            alias_map.insert(dest, existing);
                            changed = true;
                            continue;
                        }

                        expr_map.insert(key, dest);
                        new_instructions.push(Instruction::CmpLt { dest, left, right });
                    }

                    Instruction::CmpLe { dest, left, right } => {
                        let left = resolve_alias(&alias_map, left);
                        let right = resolve_alias(&alias_map, right);
                        let key = ExprKey::CmpLe(left, right);

                        if let Some(&existing) = expr_map.get(&key) {
                            alias_map.insert(dest, existing);
                            changed = true;
                            continue;
                        }

                        expr_map.insert(key, dest);
                        new_instructions.push(Instruction::CmpLe { dest, left, right });
                    }

                    Instruction::CmpGt { dest, left, right } => {
                        let left = resolve_alias(&alias_map, left);
                        let right = resolve_alias(&alias_map, right);
                        let key = ExprKey::CmpGt(left, right);

                        if let Some(&existing) = expr_map.get(&key) {
                            alias_map.insert(dest, existing);
                            changed = true;
                            continue;
                        }

                        expr_map.insert(key, dest);
                        new_instructions.push(Instruction::CmpGt { dest, left, right });
                    }

                    Instruction::CmpGe { dest, left, right } => {
                        let left = resolve_alias(&alias_map, left);
                        let right = resolve_alias(&alias_map, right);
                        let key = ExprKey::CmpGe(left, right);

                        if let Some(&existing) = expr_map.get(&key) {
                            alias_map.insert(dest, existing);
                            changed = true;
                            continue;
                        }

                        expr_map.insert(key, dest);
                        new_instructions.push(Instruction::CmpGe { dest, left, right });
                    }

                    Instruction::Sub { dest, left, right } => {
                        let left = resolve_alias(&alias_map, left);
                        let right = resolve_alias(&alias_map, right);
                        let key = ExprKey::Sub(left, right);

                        if let Some(&existing) = expr_map.get(&key) {
                            alias_map.insert(dest, existing);
                            changed = true;
                            continue;
                        }

                        expr_map.insert(key, dest);
                        new_instructions.push(Instruction::Sub { dest, left, right });
                    }

                    Instruction::Div { dest, left, right } => {
                        let left = resolve_alias(&alias_map, left);
                        let right = resolve_alias(&alias_map, right);
                        let key = ExprKey::Div(left, right);

                        if let Some(&existing) = expr_map.get(&key) {
                            alias_map.insert(dest, existing);
                            changed = true;
                            continue;
                        }

                        expr_map.insert(key, dest);
                        new_instructions.push(Instruction::Div { dest, left, right });
                    }

                    Instruction::Mod { dest, left, right } => {
                        let left = resolve_alias(&alias_map, left);
                        let right = resolve_alias(&alias_map, right);
                        let key = ExprKey::Mod(left, right);

                        if let Some(&existing) = expr_map.get(&key) {
                            alias_map.insert(dest, existing);
                            changed = true;
                            continue;
                        }

                        expr_map.insert(key, dest);
                        new_instructions.push(Instruction::Mod { dest, left, right });
                    }

                    Instruction::LoadFnPtr { dest, fn_name } => {
                        let key = ExprKey::LoadFnPtr(fn_name.clone());

                        if let Some(&existing) = expr_map.get(&key) {
                            alias_map.insert(dest, existing);
                            changed = true;
                            continue;
                        }

                        expr_map.insert(key, dest);
                        new_instructions.push(Instruction::LoadFnPtr { dest, fn_name });
                    }

                    _ => {
                        new_instructions.push(inst);
                    }
                }
            }

            if !alias_map.is_empty() {
                for inst in &mut new_instructions {
                    Self::remap_instruction_operands(inst, &alias_map);
                }
            }

            block.instructions = new_instructions;
        }

        changed
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

            | Instruction::Call { args, .. }
            | Instruction::DynamicCall { args, .. }
            | Instruction::CallClosure { args, .. } => {
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

    fn simplify_identity_casts(func: &mut FunctionIr) -> bool {
        let mut changed = false;
        let mut to_remove: Vec<(BlockId, usize)> = Vec::new();
        let mut replacements: Vec<(VReg, VReg)> = Vec::new();

        for block in &func.blocks {
            for (idx, inst) in block.instructions.iter().enumerate() {
                if let Instruction::Cast { dest, value, target_ty } = inst {
                    if let Some(src_ty) = Self::infer_register_type(func, *value) {
                        if src_ty == *target_ty {
                            replacements.push((*dest, *value));
                            to_remove.push((block.id, idx));
                        }
                    }
                }
            }
        }

        if !replacements.is_empty() {
            changed = true;
            for (dest, value) in replacements {
                Self::replace_register(func, dest, value);
            }
        }

        if changed {
            for (block_id, idx) in to_remove.into_iter().rev() {
                if let Some(block) = func.blocks.iter_mut().find(|b| b.id == block_id) {
                    if idx < block.instructions.len() {
                        block.instructions.remove(idx);
                    }
                }
            }
        }

        changed
    }

    fn merge_blocks(func: &mut FunctionIr) -> bool {
        let mut changed = false;

        loop {
            let predecessor_counts = Self::compute_predecessor_counts(func);
            let mut merged = false;

            let block_indices: HashMap<BlockId, usize> = func.blocks
                .iter()
                .enumerate()
                .map(|(idx, block)| (block.id, idx))
                .collect();

            for idx in 0..func.blocks.len() {
                let block_id = func.blocks[idx].id;
                if let Some(Instruction::Br { target }) = func.blocks[idx].instructions.last() {
                    if *target != block_id {
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

    fn replace_register(func: &mut FunctionIr, old_reg: VReg, new_reg: VReg) {
        for block in &mut func.blocks {
            for inst in &mut block.instructions {
                match inst {
                    Instruction::AllocArray { size, .. } => {
                        if *size == old_reg {
                            *size = new_reg;
                        }
                    }

                    Instruction::GetElementPtr { base_ptr, indices, .. } => {
                        if *base_ptr == old_reg {
                            *base_ptr = new_reg;
                        }

                        for idx in indices {
                            if *idx == old_reg {
                                *idx = new_reg;
                            }
                        }
                    }

                    Instruction::Load { src_ptr, .. } => {
                        if *src_ptr == old_reg {
                            *src_ptr = new_reg;
                        }
                    }

                    Instruction::Store { value, ptr, .. } => {
                        if *value == old_reg {
                            *value = new_reg;
                        }

                        if *ptr == old_reg {
                            *ptr = new_reg;
                        }
                    }

                    | Instruction::Add { left, right, .. }
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
                        if *left == old_reg {
                            *left = new_reg;
                        }

                        if *right == old_reg {
                            *right = new_reg;
                        }
                    }

                    | Instruction::Cast { value, .. }
                    | Instruction::CondBr { cond: value, .. }
                    | Instruction::MakeFatPtr { data_ptr: value, .. }
                    | Instruction::MakeClosure { env_ptr: value, .. }
                    | Instruction::Retain { ptr: value }
                    | Instruction::Release { ptr: value } => {
                        if *value == old_reg {
                            *value = new_reg;
                        }
                    }

                    | Instruction::Call { args, .. }
                    | Instruction::DynamicCall { args, .. }
                    | Instruction::CallClosure { args, .. } => {
                        for arg in args {
                            if *arg == old_reg {
                                *arg = new_reg;
                            }
                        }
                    }

                    Instruction::IndirectCall { fn_ptr, args, .. } => {
                        if *fn_ptr == old_reg {
                            *fn_ptr = new_reg;
                        }

                        for arg in args {
                            if *arg == old_reg {
                                *arg = new_reg;
                            }
                        }
                    }

                    _ => {}
                }
            }
        }
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

        // step b: count uses and track what pointers are really read
        let mut uses = HashMap::new();
        let mut ptr_reads = HashSet::new();

        for block in &func.blocks {
            for inst in &block.instructions {
                match inst {
                    Instruction::GetElementPtr { base_ptr, indices, .. } => {
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

                    | Instruction::Add { left, right, .. }
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

                    Instruction::CallClosure { closure_ptr, args, .. } => {
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
                if
                    let Instruction::Alloca { dest, .. } | Instruction::AllocStruct { dest, .. } =
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
            block.instructions.retain(|inst| {
                match inst {
                    Instruction::Alloca { dest, .. } | Instruction::AllocStruct { dest, .. } => {
                        !dead_allocas.contains(dest)
                    }

                    Instruction::Store { ptr, .. } => !dead_allocas.contains(ptr),

                    | Instruction::ConstInt { dest, .. }
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
                    | Instruction::MakeClosure { dest, .. } =>
                        uses.get(dest).copied().unwrap_or(0) > 0,

                    | Instruction::Br { .. }
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
                }
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
                        | Instruction::Alloca { dest, .. }
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
                    if let Instruction::Call { dest: call_dest, func_name, args } = inst {
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
                                    | Instruction::ConstInt { dest, .. }
                                    | Instruction::ConstBool { dest, .. }
                                    | Instruction::ConstString { dest, .. } => {
                                        *dest = get_new_reg(*dest);
                                    }

                                    Instruction::GetElementPtr { dest, base_ptr, indices, .. } => {
                                        *dest = get_new_reg(*dest);
                                        *base_ptr = remap(*base_ptr, &reg_map);

                                        for idx in indices.iter_mut() {
                                            *idx = remap(*idx, &reg_map);
                                        }
                                    }

                                    | Instruction::Alloca { dest, .. }
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

                                    | Instruction::Add { dest, left, right, .. }
                                    | Instruction::Sub { dest, left, right, .. }
                                    | Instruction::Mul { dest, left, right, .. }
                                    | Instruction::Div { dest, left, right, .. }
                                    | Instruction::Mod { dest, left, right, .. }
                                    | Instruction::CmpEq { dest, left, right, .. }
                                    | Instruction::CmpNeq { dest, left, right, .. }
                                    | Instruction::CmpLt { dest, left, right, .. }
                                    | Instruction::CmpLe { dest, left, right, .. }
                                    | Instruction::CmpGt { dest, left, right, .. }
                                    | Instruction::CmpGe { dest, left, right, .. } => {
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

                                    Instruction::Ret { value: Some(ret_val) } => {
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

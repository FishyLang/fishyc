use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::builder::Builder;
use inkwell::values::{ BasicValueEnum, BasicMetadataValueEnum };
use inkwell::types::{ BasicTypeEnum, BasicType };
use inkwell::targets::{
    InitializationConfig,
    Target,
    TargetMachine,
    RelocMode,
    CodeModel,
    FileType,
};
use std::collections::HashMap;
use std::path::Path;

use crate::ir::{ ModuleIr, Instruction, VReg, IrType };

pub struct LlvmEmitter<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    registers: HashMap<VReg, BasicValueEnum<'ctx>>,
}

impl<'ctx> LlvmEmitter<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        Self {
            context,
            module,
            builder,
            registers: HashMap::new(),
        }
    }

    fn get_llvm_type(&self, ty: &IrType) -> BasicTypeEnum<'ctx> {
        match ty {
            IrType::Void =>
                panic!("ICE: Attempted to convert IrType::Void into an LLVM BasicType!"),
            IrType::I8 => self.context.i8_type().into(),
            IrType::I16 => self.context.i16_type().into(),
            IrType::I32 => self.context.i32_type().into(),
            IrType::I64 | IrType::Any => self.context.i64_type().into(),
            IrType::F32 => self.context.f32_type().into(),
            IrType::F64 => self.context.f64_type().into(),
            IrType::Bool => self.context.bool_type().into(),

            IrType::Ptr(_) | IrType::FatPtr =>
                self.context.i64_type().ptr_type(inkwell::AddressSpace::default()).into(),

            IrType::Array(size, inner) =>
                self
                    .get_llvm_type(inner)
                    .array_type(*size as u32)
                    .into(),

            _ => self.context.i64_type().into(),
        }
    }

    pub fn compile(&mut self, ir_module: &ModuleIr) -> Result<(), String> {
        let mut llvm_funcs: HashMap<String, inkwell::values::FunctionValue<'ctx>> = HashMap::new();

        for func in &ir_module.functions {
            let mut param_types: Vec<inkwell::types::BasicMetadataTypeEnum> = Vec::new();
            for (_, arg_ty) in &func.args {
                param_types.push(self.get_llvm_type(arg_ty).into());
            }

            let is_variadic = func.is_variadic;

            let fn_type = if func.ret_type == IrType::Void {
                self.context.void_type().fn_type(param_types.as_slice(), is_variadic)
            } else {
                self.get_llvm_type(&func.ret_type).fn_type(param_types.as_slice(), is_variadic)
            };

            let llvm_func = self.module.add_function(&func.name, fn_type, None);
            llvm_funcs.insert(func.name.clone(), llvm_func);
        }

        let i64_ty = self.context.i64_type();
        for (vtable_name, method_names) in &ir_module.vtables {
            let mut func_ints = Vec::new();
            for m_name in method_names {
                let func = llvm_funcs
                    .get(m_name)
                    .ok_or_else(|| format!("VTable method '{}' not found!", m_name))?;
                let ptr_val = func.as_global_value().as_pointer_value();
                let const_int = ptr_val.const_to_int(i64_ty);
                func_ints.push(const_int);
            }
            let array_ty = i64_ty.array_type(func_ints.len() as u32);
            let const_array = i64_ty.const_array(&func_ints);
            let global_vtable = self.module.add_global(array_ty, None, vtable_name);
            global_vtable.set_initializer(&const_array);
            global_vtable.set_constant(true);
        }

        let ctx = self.context;
        let as_ptr = |
            builder: &Builder<'ctx>,
            val: BasicValueEnum<'ctx>
        | -> inkwell::values::PointerValue<'ctx> {
            if val.is_int_value() {
                let ptr_ty = ctx.i64_type().ptr_type(inkwell::AddressSpace::default());
                builder.build_int_to_ptr(val.into_int_value(), ptr_ty, "inttoptr").unwrap()
            } else {
                val.into_pointer_value()
            }
        };

        let as_int = |
            builder: &Builder<'ctx>,
            val: BasicValueEnum<'ctx>
        | -> inkwell::values::IntValue<'ctx> {
            if val.is_pointer_value() {
                builder
                    .build_ptr_to_int(val.into_pointer_value(), ctx.i64_type(), "ptr2int")
                    .unwrap()
            } else {
                val.into_int_value()
            }
        };

        for func in &ir_module.functions {
            let llvm_func = *llvm_funcs
                .get(&func.name)
                .ok_or_else(||
                    format!("LLVM Error: Function '{}' not found in the functions map!", func.name)
                )?;

            self.registers.clear();

            for (i, (reg, _)) in func.args.iter().enumerate() {
                let val = llvm_func.get_nth_param(i as u32).unwrap();
                self.registers.insert(*reg, val);
            }

            let mut llvm_blocks = HashMap::new();
            for block in &func.blocks {
                let bb = self.context.append_basic_block(llvm_func, &format!("bb{}", block.id.0));
                llvm_blocks.insert(block.id.0, bb);
            }

            for block in &func.blocks {
                let current_bb = *llvm_blocks
                    .get(&block.id.0)
                    .ok_or_else(||
                        format!("LLVM ERROR: Basic block 'bb{}' not found!", block.id.0)
                    )?;
                self.builder.position_at_end(current_bb);

                for inst in &block.instructions {
                    match inst {
                        Instruction::ConstInt { dest, value } => {
                            let val = self.context.i64_type().const_int(*value as u64, false);
                            self.registers.insert(*dest, val.into());
                        }

                        Instruction::ConstFloat { dest, value, ty } => {
                            let val = match ty {
                                IrType::F16 => self.context.f16_type().const_float(*value),
                                IrType::F32 => self.context.f32_type().const_float(*value),
                                IrType::F64 => self.context.f64_type().const_float(*value),

                                _ => panic!("ICE: ConstFloat contains an invalid type!"),
                            };

                            self.registers.insert(*dest, val.into());
                        }

                        Instruction::ConstBool { dest, value } => {
                            let val = self.context
                                .bool_type()
                                .const_int(if *value { 1 } else { 0 }, false);
                            self.registers.insert(*dest, val.into());
                        }

                        Instruction::ConstString { dest, value } => {
                            let global_str = self.builder
                                .build_global_string_ptr(value, "global_str")
                                .unwrap();
                            self.registers.insert(*dest, global_str.as_pointer_value().into());
                        }

                        Instruction::Alloca { dest, name, ty } => {
                            let llvm_ty = self.get_llvm_type(ty);
                            let ptr = self.builder.build_alloca(llvm_ty, name).unwrap();
                            self.registers.insert(*dest, ptr.into());
                        }

                        Instruction::AllocArray { dest, size, ty: _ } => {
                            let size_val = *self.registers
                                .get(size)
                                .expect("LLVM ERROR: Size register missing in AllocArray!");
                            let size_int = size_val.into_int_value();

                            // allocate in heap with header
                            // memory needed: (size * 8 bytes) + 8 bytes for header
                            let eight = self.context.i64_type().const_int(8, false);
                            let data_bytes = self.builder
                                .build_int_mul(size_int, eight, "data_bytes")
                                .unwrap();
                            let total_bytes = self.builder
                                .build_int_add(data_bytes, eight, "total_bytes")
                                .unwrap();

                            let malloc_func = self.module
                                .get_function("malloc")
                                .expect("CRITICAL ERROR: 'malloc' not found in module!");

                            let call = self.builder
                                .build_call(malloc_func, &[total_bytes.into()], "arr_alloc")
                                .unwrap();
                            let raw_ptr = call
                                .try_as_basic_value()
                                .left()
                                .unwrap()
                                .into_pointer_value();

                            // store size (size_int) in byte 0
                            let i64_ptr_ty = self.context
                                .i64_type()
                                .ptr_type(inkwell::AddressSpace::default());
                            let raw_i64_ptr = self.builder
                                .build_pointer_cast(raw_ptr, i64_ptr_ty, "cast")
                                .unwrap();
                            self.builder.build_store(raw_i64_ptr, size_int).unwrap();

                            // advance pointer 1 index (8 bytes) to hide the header from users
                            let idx1 = self.context.i64_type().const_int(1, false);
                            let data_ptr = unsafe {
                                self.builder
                                    .build_gep(
                                        self.context.i64_type(),
                                        raw_i64_ptr,
                                        &[idx1],
                                        "data_ptr"
                                    )
                                    .unwrap()
                            };

                            self.registers.insert(*dest, data_ptr.into());
                        }

                        Instruction::AllocStruct { dest, class_name: _class_name, size } => {
                            let malloc_func = self.module
                                .get_function("malloc")
                                .ok_or_else(||
                                    "CRITICAL ERROR: 'malloc' not found in module or prelude is missing!".to_string()
                                )?;

                            let size_val = self.context.i64_type().const_int(*size as u64, false);

                            let call = self.builder
                                .build_call(malloc_func, &[size_val.into()], "heap_alloc")
                                .unwrap();
                            let ptr = call
                                .try_as_basic_value()
                                .left()
                                .ok_or_else(||
                                    "ERROR: malloc did not return a value!".to_string()
                                )?;

                            self.registers.insert(*dest, ptr);
                        }

                        Instruction::GetElementPtr { dest, base_ty, base_ptr, indices } => {
                            let base_val = *self.registers
                                .get(base_ptr)
                                .ok_or_else(||
                                    format!("LLVM ERROR: Base register '{}' missing in GEP instruction!", base_ptr)
                                )?;

                            let ptr_val = as_ptr(&self.builder, base_val);

                            let mut llvm_indices = Vec::new();
                            for idx in indices {
                                let idx_val = *self.registers
                                    .get(idx)
                                    .ok_or_else(||
                                        format!("LLVM ERROR: Index register '{}' missing in GEP instruction!", idx)
                                    )?;
                                llvm_indices.push(idx_val.into_int_value());
                            }

                            let llvm_base_ty = self.get_llvm_type(base_ty);
                            let gep = unsafe {
                                self.builder
                                    .build_gep(llvm_base_ty, ptr_val, &llvm_indices, "gep")
                                    .unwrap()
                            };
                            self.registers.insert(*dest, gep.into());
                        }

                        Instruction::Store { ty, value, ptr } => {
                            let ptr_raw = *self.registers
                                .get(ptr)
                                .ok_or_else(||
                                    format!("LLVM ERROR: Pointer '{}' missing in Store instruction!", ptr)
                                )?;

                            let ptr_val = as_ptr(&self.builder, ptr_raw);

                            let mut val = *self.registers
                                .get(value)
                                .ok_or_else(||
                                    format!("LLVM ERROR: Value '{}' missing in Store instruction!", value)
                                )?;

                            let target_llvm_ty = self.get_llvm_type(ty);

                            if val.is_pointer_value() && target_llvm_ty.is_int_type() {
                                val = self.builder
                                    .build_ptr_to_int(
                                        val.into_pointer_value(),
                                        target_llvm_ty.into_int_type(),
                                        "store_cast_int"
                                    )
                                    .unwrap()
                                    .into();
                            } else if val.is_int_value() && target_llvm_ty.is_pointer_type() {
                                val = self.builder
                                    .build_int_to_ptr(
                                        val.into_int_value(),
                                        target_llvm_ty.into_pointer_type(),
                                        "store_cast_ptr"
                                    )
                                    .unwrap()
                                    .into();
                            }

                            if val.is_int_value() && target_llvm_ty.is_int_type() {
                                let val_int = val.into_int_value();
                                let target_int = target_llvm_ty.into_int_type();

                                if val_int.get_type().get_bit_width() > target_int.get_bit_width() {
                                    val = self.builder
                                        .build_int_truncate(val_int, target_int, "trunc")
                                        .unwrap()
                                        .into();
                                } else if
                                    val_int.get_type().get_bit_width() < target_int.get_bit_width()
                                {
                                    val = self.builder
                                        .build_int_z_extend(val_int, target_int, "zext")
                                        .unwrap()
                                        .into();
                                }
                            }

                            self.builder.build_store(ptr_val, val).unwrap();
                        }

                        Instruction::Load { dest, ty, src_ptr } => {
                            let ptr_raw = *self.registers
                                .get(src_ptr)
                                .ok_or_else(||
                                    format!("LLVM ERROR: Source pointer '{}' missing in Load instruction!", src_ptr)
                                )?;

                            let llvm_ty = self.get_llvm_type(ty);
                            let ptr_val = as_ptr(&self.builder, ptr_raw);
                            let val = self.builder.build_load(llvm_ty, ptr_val, "load").unwrap();

                            let final_val = if
                                val.is_int_value() &&
                                val.into_int_value().get_type().get_bit_width() < 64
                            {
                                self.builder
                                    .build_int_z_extend(
                                        val.into_int_value(),
                                        self.context.i64_type(),
                                        "zext"
                                    )
                                    .unwrap()
                                    .into()
                            } else {
                                val
                            };

                            self.registers.insert(*dest, final_val);
                        }

                        Instruction::Cast { dest, value, target_ty } => {
                            let val = *self.registers
                                .get(value)
                                .ok_or_else(||
                                    format!("LLVM ERROR: Value register '{}' missing in Cast instruction!", value)
                                )?;

                            let llvm_target_ty = self.get_llvm_type(target_ty);

                            let casted = if val.is_int_value() && llvm_target_ty.is_int_type() {
                                self.builder
                                    .build_int_cast(
                                        val.into_int_value(),
                                        llvm_target_ty.into_int_type(),
                                        "cast"
                                    )
                                    .unwrap()
                                    .into()
                            } else if val.is_int_value() && llvm_target_ty.is_pointer_type() {
                                let ptr_ty = self.context
                                    .i64_type()
                                    .ptr_type(inkwell::AddressSpace::default());
                                self.builder
                                    .build_int_to_ptr(val.into_int_value(), ptr_ty, "inttoptr")
                                    .unwrap()
                                    .into()
                            } else if val.is_pointer_value() && llvm_target_ty.is_int_type() {
                                self.builder
                                    .build_ptr_to_int(
                                        val.into_pointer_value(),
                                        llvm_target_ty.into_int_type(),
                                        "ptrtoint"
                                    )
                                    .unwrap()
                                    .into()
                            } else {
                                val
                            };
                            self.registers.insert(*dest, casted);
                        }

                        Instruction::Add { dest, left, right } => {
                            let l = *self.registers
                                .get(left)
                                .ok_or_else(||
                                    "LLVM ERROR: Left value missing in Add instruction!".to_string()
                                )?;
                            let r = *self.registers
                                .get(right)
                                .ok_or_else(||
                                    "LLVM ERROR: Right value missing in Add instruction!".to_string()
                                )?;
                            if l.is_float_value() {
                                let res = self.builder
                                    .build_float_add(
                                        l.into_float_value(),
                                        r.into_float_value(),
                                        "fadd"
                                    )
                                    .unwrap();
                                self.registers.insert(*dest, res.into());
                            } else {
                                let res = self.builder
                                    .build_int_add(
                                        as_int(&self.builder, l),
                                        as_int(&self.builder, r),
                                        "add"
                                    )
                                    .unwrap();
                                self.registers.insert(*dest, res.into());
                            }
                        }

                        Instruction::Sub { dest, left, right } => {
                            let l = *self.registers
                                .get(left)
                                .ok_or_else(||
                                    "LLVM ERROR: Left value missing in Sub instruction!".to_string()
                                )?;
                            let r = *self.registers
                                .get(right)
                                .ok_or_else(||
                                    "LLVM ERROR: Right value missing in Sub instruction!".to_string()
                                )?;
                            if l.is_float_value() {
                                let res = self.builder
                                    .build_float_sub(
                                        l.into_float_value(),
                                        r.into_float_value(),
                                        "fsub"
                                    )
                                    .unwrap();
                                self.registers.insert(*dest, res.into());
                            } else {
                                let res = self.builder
                                    .build_int_sub(
                                        as_int(&self.builder, l),
                                        as_int(&self.builder, r),
                                        "sub"
                                    )
                                    .unwrap();
                                self.registers.insert(*dest, res.into());
                            }
                        }

                        Instruction::Mul { dest, left, right } => {
                            let l = *self.registers
                                .get(left)
                                .ok_or_else(||
                                    "LLVM ERROR: Left value missing in Mul instruction!".to_string()
                                )?;
                            let r = *self.registers
                                .get(right)
                                .ok_or_else(||
                                    "LLVM ERROR: Right value missing in Mul instruction!".to_string()
                                )?;
                            if l.is_float_value() {
                                let res = self.builder
                                    .build_float_mul(
                                        l.into_float_value(),
                                        r.into_float_value(),
                                        "fmul"
                                    )
                                    .unwrap();
                                self.registers.insert(*dest, res.into());
                            } else {
                                let res = self.builder
                                    .build_int_mul(
                                        as_int(&self.builder, l),
                                        as_int(&self.builder, r),
                                        "mul"
                                    )
                                    .unwrap();
                                self.registers.insert(*dest, res.into());
                            }
                        }

                        Instruction::Div { dest, left, right } => {
                            let l = *self.registers
                                .get(left)
                                .ok_or_else(||
                                    "LLVM ERROR: Left value missing in Div instruction!".to_string()
                                )?;
                            let r = *self.registers
                                .get(right)
                                .ok_or_else(||
                                    "LLVM ERROR: Right value missing in Div instruction!".to_string()
                                )?;
                            if l.is_float_value() {
                                let res = self.builder
                                    .build_float_div(
                                        l.into_float_value(),
                                        r.into_float_value(),
                                        "fdiv"
                                    )
                                    .unwrap();
                                self.registers.insert(*dest, res.into());
                            } else {
                                let res = self.builder
                                    .build_int_signed_div(
                                        as_int(&self.builder, l),
                                        as_int(&self.builder, r),
                                        "div"
                                    )
                                    .unwrap();
                                self.registers.insert(*dest, res.into());
                            }
                        }

                        Instruction::Mod { dest, left, right } => {
                            let l = *self.registers
                                .get(left)
                                .ok_or_else(||
                                    "LLVM ERROR: Left value missing in Mod instruction!".to_string()
                                )?;
                            let r = *self.registers
                                .get(right)
                                .ok_or_else(||
                                    "LLVM ERROR: Right value missing in Mod instruction!".to_string()
                                )?;
                            if l.is_float_value() {
                                let res = self.builder
                                    .build_float_rem(
                                        l.into_float_value(),
                                        r.into_float_value(),
                                        "frem"
                                    )
                                    .unwrap();
                                self.registers.insert(*dest, res.into());
                            } else {
                                let res = self.builder
                                    .build_int_signed_rem(
                                        as_int(&self.builder, l),
                                        as_int(&self.builder, r),
                                        "rem"
                                    )
                                    .unwrap();
                                self.registers.insert(*dest, res.into());
                            }
                        }

                        Instruction::CmpEq { dest, left, right } => {
                            let l = *self.registers
                                .get(left)
                                .ok_or_else(||
                                    "LLVM ERROR: Left value missing in CmpEq instruction!".to_string()
                                )?;
                            let r = *self.registers
                                .get(right)
                                .ok_or_else(||
                                    "LLVM ERROR: Right value missing in CmpEq instruction!".to_string()
                                )?;
                            let cmp = if l.is_float_value() {
                                self.builder
                                    .build_float_compare(
                                        inkwell::FloatPredicate::OEQ,
                                        l.into_float_value(),
                                        r.into_float_value(),
                                        "fcmpeq"
                                    )
                                    .unwrap()
                            } else {
                                self.builder
                                    .build_int_compare(
                                        inkwell::IntPredicate::EQ,
                                        as_int(&self.builder, l),
                                        as_int(&self.builder, r),
                                        "cmpeq"
                                    )
                                    .unwrap()
                            };
                            let zext = self.builder
                                .build_int_z_extend(cmp, self.context.i64_type(), "zext")
                                .unwrap();
                            self.registers.insert(*dest, zext.into());
                        }

                        Instruction::CmpLt { dest, left, right } => {
                            let l = *self.registers
                                .get(left)
                                .ok_or_else(||
                                    "LLVM ERROR: Left value missing in CmpLt instruction!".to_string()
                                )?;
                            let r = *self.registers
                                .get(right)
                                .ok_or_else(||
                                    "LLVM ERROR: Right value missing in CmpLt instruction!".to_string()
                                )?;
                            let cmp = if l.is_float_value() {
                                self.builder
                                    .build_float_compare(
                                        inkwell::FloatPredicate::OLT,
                                        l.into_float_value(),
                                        r.into_float_value(),
                                        "fcmplt"
                                    )
                                    .unwrap()
                            } else {
                                self.builder
                                    .build_int_compare(
                                        inkwell::IntPredicate::SLT,
                                        as_int(&self.builder, l),
                                        as_int(&self.builder, r),
                                        "cmplt"
                                    )
                                    .unwrap()
                            };
                            let zext = self.builder
                                .build_int_z_extend(cmp, self.context.i64_type(), "zext")
                                .unwrap();
                            self.registers.insert(*dest, zext.into());
                        }

                        Instruction::CmpGt { dest, left, right } => {
                            let l = *self.registers
                                .get(left)
                                .ok_or_else(||
                                    "LLVM ERROR: Left value missing in CmpGt instruction!".to_string()
                                )?;
                            let r = *self.registers
                                .get(right)
                                .ok_or_else(||
                                    "LLVM ERROR: Right value missing in CmpGt instruction!".to_string()
                                )?;
                            let cmp = if l.is_float_value() {
                                self.builder
                                    .build_float_compare(
                                        inkwell::FloatPredicate::OGT,
                                        l.into_float_value(),
                                        r.into_float_value(),
                                        "fcmpgt"
                                    )
                                    .unwrap()
                            } else {
                                self.builder
                                    .build_int_compare(
                                        inkwell::IntPredicate::SGT,
                                        as_int(&self.builder, l),
                                        as_int(&self.builder, r),
                                        "cmpgt"
                                    )
                                    .unwrap()
                            };
                            let zext = self.builder
                                .build_int_z_extend(cmp, self.context.i64_type(), "zext")
                                .unwrap();
                            self.registers.insert(*dest, zext.into());
                        }

                        Instruction::CmpNeq { dest, left, right } => {
                            let l = *self.registers
                                .get(left)
                                .ok_or_else(||
                                    "LLVM ERROR: Left value missing in CmpNeq instruction!".to_string()
                                )?;
                            let r = *self.registers
                                .get(right)
                                .ok_or_else(||
                                    "LLVM ERROR: Right value missing in CmpNeq instruction!".to_string()
                                )?;
                            let cmp = if l.is_float_value() {
                                self.builder
                                    .build_float_compare(
                                        inkwell::FloatPredicate::ONE,
                                        l.into_float_value(),
                                        r.into_float_value(),
                                        "fcmpne"
                                    )
                                    .unwrap()
                            } else {
                                self.builder
                                    .build_int_compare(
                                        inkwell::IntPredicate::NE,
                                        as_int(&self.builder, l),
                                        as_int(&self.builder, r),
                                        "cmpne"
                                    )
                                    .unwrap()
                            };
                            let zext = self.builder
                                .build_int_z_extend(cmp, self.context.i64_type(), "zext")
                                .unwrap();
                            self.registers.insert(*dest, zext.into());
                        }

                        Instruction::CmpLe { dest, left, right } => {
                            let l = *self.registers
                                .get(left)
                                .ok_or_else(||
                                    "LLVM ERROR: Left value missing in CmpLe instruction!".to_string()
                                )?;
                            let r = *self.registers
                                .get(right)
                                .ok_or_else(||
                                    "LLVM ERROR: Right value missing in CmpLe instruction!".to_string()
                                )?;
                            let cmp = if l.is_float_value() {
                                self.builder
                                    .build_float_compare(
                                        inkwell::FloatPredicate::OLE,
                                        l.into_float_value(),
                                        r.into_float_value(),
                                        "fcmple"
                                    )
                                    .unwrap()
                            } else {
                                self.builder
                                    .build_int_compare(
                                        inkwell::IntPredicate::SLE,
                                        as_int(&self.builder, l),
                                        as_int(&self.builder, r),
                                        "cmple"
                                    )
                                    .unwrap()
                            };
                            let zext = self.builder
                                .build_int_z_extend(cmp, self.context.i64_type(), "zext")
                                .unwrap();
                            self.registers.insert(*dest, zext.into());
                        }

                        Instruction::CmpGe { dest, left, right } => {
                            let l = *self.registers
                                .get(left)
                                .ok_or_else(||
                                    "LLVM ERROR: Left value missing in CmpGe instruction!".to_string()
                                )?;
                            let r = *self.registers
                                .get(right)
                                .ok_or_else(||
                                    "LLVM ERROR: Right value missing in CmpGe instruction!".to_string()
                                )?;
                            let cmp = if l.is_float_value() {
                                self.builder
                                    .build_float_compare(
                                        inkwell::FloatPredicate::OGE,
                                        l.into_float_value(),
                                        r.into_float_value(),
                                        "fcmpge"
                                    )
                                    .unwrap()
                            } else {
                                self.builder
                                    .build_int_compare(
                                        inkwell::IntPredicate::SGE,
                                        as_int(&self.builder, l),
                                        as_int(&self.builder, r),
                                        "cmpge"
                                    )
                                    .unwrap()
                            };
                            let zext = self.builder
                                .build_int_z_extend(cmp, self.context.i64_type(), "zext")
                                .unwrap();
                            self.registers.insert(*dest, zext.into());
                        }

                        Instruction::CondBr { cond, if_true, if_false } => {
                            let cond_val = self.registers
                                .get(cond)
                                .ok_or_else(||
                                    format!("LLVM ERROR: Condition register '{}' missing in CondBr instruction!", cond)
                                )?
                                .into_int_value();
                            let trunc = self.builder
                                .build_int_truncate(cond_val, self.context.bool_type(), "trunc")
                                .unwrap();
                            let bb_true = llvm_blocks.get(&if_true.0).unwrap();
                            let bb_false = llvm_blocks.get(&if_false.0).unwrap();
                            self.builder
                                .build_conditional_branch(trunc, *bb_true, *bb_false)
                                .unwrap();
                        }

                        Instruction::Br { target } => {
                            let bb = llvm_blocks.get(&target.0).unwrap();
                            self.builder.build_unconditional_branch(*bb).unwrap();
                        }

                        Instruction::Ret { value } => {
                            let current_func = self.builder
                                .get_insert_block()
                                .unwrap()
                                .get_parent()
                                .unwrap();

                            let expected_ret_ty = current_func.get_type().get_return_type();

                            if let Some(v) = value {
                                let mut val = *self.registers
                                    .get(v)
                                    .expect(
                                        &format!("LLVM ERROR: Return register '{}' missing!", v)
                                    );

                                if let Some(expected_ty) = expected_ret_ty {
                                    if val.is_pointer_value() && expected_ty.is_int_type() {
                                        val = self.builder
                                            .build_ptr_to_int(
                                                val.into_pointer_value(),
                                                expected_ty.into_int_type(),
                                                "ret_cast_int"
                                            )
                                            .unwrap()
                                            .into();
                                    } else if val.is_int_value() && expected_ty.is_pointer_type() {
                                        val = self.builder
                                            .build_int_to_ptr(
                                                val.into_int_value(),
                                                expected_ty.into_pointer_type(),
                                                "ret_cast_ptr"
                                            )
                                            .unwrap()
                                            .into();
                                    } else if val.is_int_value() && expected_ty.is_int_type() {
                                        let val_int = val.into_int_value();
                                        let expected_int = expected_ty.into_int_type();
                                        if
                                            val_int.get_type().get_bit_width() >
                                            expected_int.get_bit_width()
                                        {
                                            val = self.builder
                                                .build_int_truncate(
                                                    val_int,
                                                    expected_int,
                                                    "ret_trunc"
                                                )
                                                .unwrap()
                                                .into();
                                        } else if
                                            val_int.get_type().get_bit_width() <
                                            expected_int.get_bit_width()
                                        {
                                            val = self.builder
                                                .build_int_z_extend(
                                                    val_int,
                                                    expected_int,
                                                    "ret_zext"
                                                )
                                                .unwrap()
                                                .into();
                                        }
                                    }
                                }

                                self.builder.build_return(Some(&val)).unwrap();
                            } else {
                                if let Some(_) = expected_ret_ty {
                                    self.builder.build_unreachable().unwrap();
                                } else {
                                    self.builder.build_return(None).unwrap();
                                }
                            }
                        }

                        Instruction::Call { dest, func_name, args } => {
                            let function = *llvm_funcs
                                .get(func_name)
                                .expect(
                                    &format!("LLVM ERROR: Calling inexistent function '{}'!", func_name)
                                );

                            let fn_type = function.get_type();
                            let param_types = fn_type.get_param_types();

                            let mut llvm_args: Vec<BasicMetadataValueEnum> = Vec::new();
                            for (i, arg) in args.iter().enumerate() {
                                let mut val = *self.registers
                                    .get(arg)
                                    .expect("LLVM ERROR: Call instruction argument missing");

                                if i < param_types.len() {
                                    let expected_ty = param_types[i];
                                    if val.is_int_value() && expected_ty.is_pointer_type() {
                                        let ptr_ty = expected_ty.into_pointer_type();
                                        val = self.builder
                                            .build_int_to_ptr(
                                                val.into_int_value(),
                                                ptr_ty,
                                                "auto_cast_ptr"
                                            )
                                            .unwrap()
                                            .into();
                                    } else if val.is_pointer_value() && expected_ty.is_int_type() {
                                        let int_ty = expected_ty.into_int_type();
                                        val = self.builder
                                            .build_ptr_to_int(
                                                val.into_pointer_value(),
                                                int_ty,
                                                "auto_cast_int"
                                            )
                                            .unwrap()
                                            .into();
                                    } else if val.is_int_value() && expected_ty.is_int_type() {
                                        let val_int = val.into_int_value();
                                        let expected_int = expected_ty.into_int_type();
                                        if
                                            val_int.get_type().get_bit_width() >
                                            expected_int.get_bit_width()
                                        {
                                            val = self.builder
                                                .build_int_truncate(
                                                    val_int,
                                                    expected_int,
                                                    "arg_trunc"
                                                )
                                                .unwrap()
                                                .into();
                                        } else if
                                            val_int.get_type().get_bit_width() <
                                            expected_int.get_bit_width()
                                        {
                                            val = self.builder
                                                .build_int_z_extend(
                                                    val_int,
                                                    expected_int,
                                                    "arg_zext"
                                                )
                                                .unwrap()
                                                .into();
                                        }
                                    }
                                }

                                llvm_args.push(val.into());
                            }

                            let call_site = self.builder
                                .build_call(function, &llvm_args, "call")
                                .unwrap();

                            if let Some(ret_val) = call_site.try_as_basic_value().left() {
                                self.registers.insert(*dest, ret_val);
                            }
                        }

                        Instruction::MakeFatPtr { dest, data_ptr, vtable_name } => {
                            let fat_ptr_ty = self.context.i64_type().array_type(2);
                            let fat_ptr_alloc = self.builder
                                .build_alloca(fat_ptr_ty, "fat_ptr")
                                .unwrap();

                            let data_raw = *self.registers
                                .get(data_ptr)
                                .ok_or_else(||
                                    "LLVM ERROR: data_ptr register missing!".to_string()
                                )?;
                            let data_int = if data_raw.is_pointer_value() {
                                self.builder
                                    .build_ptr_to_int(
                                        data_raw.into_pointer_value(),
                                        self.context.i64_type(),
                                        "cast"
                                    )
                                    .unwrap()
                                    .into()
                            } else {
                                data_raw.into_int_value()
                            };

                            let idx0 = self.context.i64_type().const_zero();
                            let data_field = unsafe {
                                self.builder
                                    .build_gep(
                                        fat_ptr_ty,
                                        fat_ptr_alloc,
                                        &[idx0, idx0],
                                        "data_field"
                                    )
                                    .unwrap()
                            };
                            self.builder.build_store(data_field, data_int).unwrap();

                            let vtable_global = self.module.get_global(vtable_name).unwrap();
                            let vtable_ptr = vtable_global.as_pointer_value();
                            let vtable_int = self.builder
                                .build_ptr_to_int(vtable_ptr, self.context.i64_type(), "vtable_int")
                                .unwrap();

                            let idx1 = self.context.i64_type().const_int(1, false);
                            let vtable_field = unsafe {
                                self.builder
                                    .build_gep(
                                        fat_ptr_ty,
                                        fat_ptr_alloc,
                                        &[idx0, idx1],
                                        "vtable_field"
                                    )
                                    .unwrap()
                            };
                            self.builder.build_store(vtable_field, vtable_int).unwrap();

                            self.registers.insert(*dest, fat_ptr_alloc.into());
                        }

                        Instruction::DynamicCall {
                            dest,
                            vtable_index,
                            fat_ptr,
                            args,
                            arg_types,
                            ret_type,
                        } => {
                            let fat_ptr_raw = *self.registers
                                .get(fat_ptr)
                                .ok_or_else(|| "LLVM ERROR: fat_ptr missing!".to_string())?;
                            let fat_ptr_alloc = as_ptr(&self.builder, fat_ptr_raw);

                            let i64_ty = self.context.i64_type();
                            let fat_ptr_ty = i64_ty.array_type(2);
                            let idx0 = i64_ty.const_zero();

                            let data_field = unsafe {
                                self.builder
                                    .build_gep(
                                        fat_ptr_ty,
                                        fat_ptr_alloc,
                                        &[idx0, idx0],
                                        "data_field"
                                    )
                                    .unwrap()
                            };
                            let data_int = self.builder
                                .build_load(i64_ty, data_field, "data_int")
                                .unwrap()
                                .into_int_value();
                            let data_ptr = self.builder
                                .build_int_to_ptr(
                                    data_int,
                                    i64_ty.ptr_type(inkwell::AddressSpace::default()),
                                    "data_ptr"
                                )
                                .unwrap();

                            let idx1 = i64_ty.const_int(1, false);
                            let vtable_field = unsafe {
                                self.builder
                                    .build_gep(
                                        fat_ptr_ty,
                                        fat_ptr_alloc,
                                        &[idx0, idx1],
                                        "vtable_field"
                                    )
                                    .unwrap()
                            };
                            let vtable_int = self.builder
                                .build_load(i64_ty, vtable_field, "vtable_int")
                                .unwrap()
                                .into_int_value();
                            let vtable_ptr = self.builder
                                .build_int_to_ptr(
                                    vtable_int,
                                    i64_ty.ptr_type(inkwell::AddressSpace::default()),
                                    "vtable_ptr"
                                )
                                .unwrap();

                            let func_idx = i64_ty.const_int(*vtable_index as u64, false);
                            let func_ptr_field = unsafe {
                                self.builder
                                    .build_gep(i64_ty, vtable_ptr, &[func_idx], "func_ptr_field")
                                    .unwrap()
                            };
                            let func_int = self.builder
                                .build_load(i64_ty, func_ptr_field, "func_int")
                                .unwrap()
                                .into_int_value();

                            let mut llvm_param_types: Vec<inkwell::types::BasicMetadataTypeEnum> =
                                vec![self.get_llvm_type(&IrType::I64).into()];
                            for arg_ty in arg_types {
                                llvm_param_types.push(self.get_llvm_type(arg_ty).into());
                            }

                            let fn_ty = if *ret_type == IrType::Void {
                                self.context.void_type().fn_type(&llvm_param_types, false)
                            } else {
                                self.get_llvm_type(ret_type).fn_type(&llvm_param_types, false)
                            };

                            let func_ptr_callable = self.builder
                                .build_int_to_ptr(
                                    func_int,
                                    fn_ty.ptr_type(inkwell::AddressSpace::default()),
                                    "func_ptr_callable"
                                )
                                .unwrap();

                            let mut llvm_args: Vec<BasicMetadataValueEnum> = vec![data_ptr.into()];
                            for (i, arg) in args.iter().enumerate() {
                                let mut val = *self.registers
                                    .get(arg)
                                    .expect("LLVM ERROR: dyn_call args missing!");

                                // the vtable starts with "self", so the real argument index is i + 1
                                if i + 1 < llvm_param_types.len() {
                                    let expected_ty = llvm_param_types[i + 1];
                                    if val.is_int_value() && expected_ty.is_pointer_type() {
                                        val = self.builder
                                            .build_int_to_ptr(
                                                val.into_int_value(),
                                                expected_ty.into_pointer_type(),
                                                "auto_cast_ptr"
                                            )
                                            .unwrap()
                                            .into();
                                    } else if val.is_pointer_value() && expected_ty.is_int_type() {
                                        val = self.builder
                                            .build_ptr_to_int(
                                                val.into_pointer_value(),
                                                expected_ty.into_int_type(),
                                                "auto_cast_int"
                                            )
                                            .unwrap()
                                            .into();
                                    } else if val.is_int_value() && expected_ty.is_int_type() {
                                        let val_int = val.into_int_value();
                                        let expected_int = expected_ty.into_int_type();
                                        if
                                            val_int.get_type().get_bit_width() >
                                            expected_int.get_bit_width()
                                        {
                                            val = self.builder
                                                .build_int_truncate(
                                                    val_int,
                                                    expected_int,
                                                    "dyn_trunc"
                                                )
                                                .unwrap()
                                                .into();
                                        } else if
                                            val_int.get_type().get_bit_width() <
                                            expected_int.get_bit_width()
                                        {
                                            val = self.builder
                                                .build_int_z_extend(
                                                    val_int,
                                                    expected_int,
                                                    "dyn_zext"
                                                )
                                                .unwrap()
                                                .into();
                                        }
                                    }
                                }
                                llvm_args.push(val.into());
                            }

                            let call_site = self.builder
                                .build_indirect_call(
                                    fn_ty,
                                    func_ptr_callable,
                                    &llvm_args,
                                    "dyn_call"
                                )
                                .unwrap();

                            if *ret_type != IrType::Void {
                                if let Some(ret_val) = call_site.try_as_basic_value().left() {
                                    self.registers.insert(*dest, ret_val);
                                }
                            }
                        }

                        Instruction::LoadFnPtr { dest, fn_name } => {
                            let func = self.module
                                .get_function(fn_name)
                                .expect(&format!("LLVM ERROR: Lambda '{}' not found!", fn_name));
                            let ptr = func.as_global_value().as_pointer_value();
                            let val = self.builder
                                .build_ptr_to_int(ptr, self.context.i64_type(), "fn_ptr_int")
                                .unwrap();

                            self.registers.insert(*dest, val.into());
                        }

                        Instruction::IndirectCall { dest, fn_ptr, args } => {
                            let ptr_raw = *self.registers.get(fn_ptr).unwrap();
                            let ptr_int = ptr_raw.into_int_value();
                            let i64_type = self.context.i64_type();

                            let mut param_types: Vec<inkwell::types::BasicMetadataTypeEnum> = vec![];
                            for _ in args {
                                param_types.push(i64_type.into());
                            }

                            let fn_ty = i64_type.fn_type(&param_types, false);
                            let callable = self.builder.build_int_to_ptr(ptr_int, fn_ty.ptr_type(inkwell::AddressSpace::default()), "callable").unwrap();

                            let mut llvm_args = vec![];
                            for arg in args {
                                let val = *self.registers.get(arg).unwrap();
                                llvm_args.push(val.into());
                            }

                            let call_site = self.builder.build_indirect_call(fn_ty, callable, &llvm_args, "icall").unwrap();

                            if let Some(ret_val) = call_site.try_as_basic_value().left() {
                                self.registers.insert(*dest, ret_val);
                            }
                        }

                        Instruction::Unreachable => {
                            self.builder.build_unreachable().unwrap();
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn build_executable(&self, filename: &str) -> Result<(), String> {
        if let Err(msg) = self.module.verify() {
            return Err(format!("LLVM module verification failed:\n{}", msg.to_string()));
        }

        Target::initialize_all(&InitializationConfig::default());
        let target_triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&target_triple).unwrap();

        let target_machine = target
            .create_target_machine(
                &target_triple,
                "generic",
                "",
                inkwell::OptimizationLevel::Aggressive,
                RelocMode::PIC,
                CodeModel::Default
            )
            .unwrap();

        target_machine
            .write_to_file(&self.module, FileType::Object, Path::new(filename))
            .map_err(|e| format!("Failed to write object file: {}", e.to_string()))?;

        println!("\nObject file '{}' successfully generated", filename);
        Ok(())
    }
}

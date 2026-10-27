use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{
    CodeModel,
    FileType,
    InitializationConfig,
    RelocMode,
    Target,
    TargetMachine,
};
use inkwell::types::{
    BasicMetadataTypeEnum,
    BasicType,
    BasicTypeEnum,
    FloatType,
    IntType,
    PointerType,
};
use inkwell::values::{
    BasicMetadataValueEnum,
    BasicValueEnum,
    FloatValue,
    FunctionValue,
    IntValue,
    PointerValue,
};
use std::collections::HashMap;
use std::path::Path;

use crate::ir::{ Instruction, IrType, ModuleIr, VReg };

pub struct LlvmEmitter<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    registers: HashMap<VReg, BasicValueEnum<'ctx>>,
    struct_layouts: HashMap<String, Vec<IrType>>,
    resources: HashMap<String, IrType>,
    string_constants: HashMap<String, PointerValue<'ctx>>,
    target_machine: TargetMachine,
}

impl<'ctx> LlvmEmitter<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        Target::initialize_native(&InitializationConfig::default()).expect("native LLVM target");
        let triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&triple).expect("host target");
        let target_machine = target
            .create_target_machine(
                &triple,
                "generic",
                "",
                inkwell::OptimizationLevel::Aggressive,
                RelocMode::PIC,
                CodeModel::Default
            )
            .expect("host target machine");
        module.set_triple(&triple);
        module.set_data_layout(&target_machine.get_target_data().get_data_layout());

        Self {
            context,
            module,
            builder,
            registers: HashMap::new(),
            struct_layouts: HashMap::new(),
            resources: HashMap::new(),
            string_constants: HashMap::new(),
            target_machine,
        }
    }

    fn collect_struct_layouts(&mut self, ir_module: &ModuleIr) {
        self.resources = ir_module.resources.clone();
        self.struct_layouts.extend(ir_module.structs.clone());
        for func in &ir_module.functions {
            self.collect_struct_types_from_ty(&func.ret_type);
            for (_, arg_ty) in &func.args {
                self.collect_struct_types_from_ty(arg_ty);
            }
            for block in &func.blocks {
                for inst in &block.instructions {
                    self.collect_struct_types_from_inst(inst);
                }
            }
        }
    }

    fn collect_struct_types_from_ty(&mut self, ty: &IrType) {
        match ty {
            IrType::Struct(name, field_types) if !field_types.is_empty() => {
                self.struct_layouts.entry(name.clone()).or_insert_with(|| field_types.clone());
                for field_ty in field_types {
                    self.collect_struct_types_from_ty(field_ty);
                }
            }
            IrType::Ptr(inner) | IrType::Array(_, inner) => {
                self.collect_struct_types_from_ty(inner);
            }
            _ => {}
        }
    }

    fn collect_struct_types_from_inst(&mut self, inst: &Instruction) {
        match inst {
            | Instruction::SizeOf { ty, .. }
            | Instruction::Alloca { ty, .. }
            | Instruction::Load { ty, .. }
            | Instruction::Store { ty, .. }
            | Instruction::Cast { target_ty: ty, .. }
            | Instruction::AllocArray { ty, .. }
            | Instruction::GetElementPtr { base_ty: ty, .. } => {
                self.collect_struct_types_from_ty(ty);
            }
            Instruction::MakeFatPtr { .. } => {}
            | Instruction::DynamicCall { arg_types, ret_type, .. }
            | Instruction::IndirectCall { arg_types, ret_type, .. }
            | Instruction::CallClosure { arg_types, ret_type, .. } => {
                for arg_ty in arg_types {
                    self.collect_struct_types_from_ty(arg_ty);
                }
                self.collect_struct_types_from_ty(ret_type);
            }
            | Instruction::MakeClosure { .. }
            | Instruction::LoadFnPtr { .. }
            | Instruction::Retain { .. }
            | Instruction::Release { .. }
            | Instruction::Call { .. }
            | Instruction::AllocStruct { .. }
            | Instruction::ConstFloat { .. }
            | Instruction::ConstString { .. }
            | Instruction::ConstBool { .. }
            | Instruction::ConstInt { .. }
            | Instruction::Add { .. }
            | Instruction::Sub { .. }
            | Instruction::Mul { .. }
            | Instruction::BitAnd { .. }
            | Instruction::BitOr { .. }
            | Instruction::Div { .. }
            | Instruction::Mod { .. }
            | Instruction::CmpEq { .. }
            | Instruction::CmpLt { .. }
            | Instruction::CmpGt { .. }
            | Instruction::CmpNeq { .. }
            | Instruction::CmpLe { .. }
            | Instruction::CmpGe { .. }
            | Instruction::Br { .. }
            | Instruction::CondBr { .. }
            | Instruction::Ret { .. }
            | Instruction::Unreachable => {
                // No type payload to collect.
            }
        }
    }

    fn get_llvm_ptr_type(&self, ty: &IrType) -> PointerType<'ctx> {
        self.get_llvm_type(ty).ptr_type(inkwell::AddressSpace::default())
    }

    fn is_signed(ty: &IrType) -> bool {
        matches!(
            ty,
            IrType::I8 |
                IrType::I16 |
                IrType::I32 |
                IrType::I64 |
                IrType::F16 |
                IrType::F32 |
                IrType::F64
        )
    }

    fn get_or_create_string_global(&mut self, value: &str) -> PointerValue<'ctx> {
        if let Some(ptr) = self.string_constants.get(value) {
            return *ptr;
        }

        let bytes = self.context.const_string(value.as_bytes(), true);
        let global = self.module.add_global(
            bytes.get_type(),
            Some(inkwell::AddressSpace::default()),
            "global_str"
        );
        global.set_linkage(inkwell::module::Linkage::Private);
        global.set_constant(true);
        global.set_initializer(&bytes);
        let ptr_value = global.as_pointer_value();
        self.string_constants.insert(value.to_owned(), ptr_value);
        ptr_value
    }

    fn value_as_int(
        builder: &Builder<'ctx>,
        val: BasicValueEnum<'ctx>,
        source_ty: &IrType,
        target_ty: IntType<'ctx>
    ) -> IntValue<'ctx> {
        if val.is_int_value() {
            let int_val = val.into_int_value();

            if int_val.get_type() == target_ty {
                return int_val;
            }

            if int_val.get_type().get_bit_width() > target_ty.get_bit_width() {
                return builder.build_int_truncate(int_val, target_ty, "int_trunc").unwrap();
            }

            // i1 represents Bool, even when an untyped comparison asks for i64.
            // Sign-extending true would produce -1 instead of canonical 1.
            if int_val.get_type().get_bit_width() == 1 {
                return builder.build_int_z_extend(int_val, target_ty, "bool_zext").unwrap();
            }
            if Self::is_signed(source_ty) {
                return builder.build_int_s_extend(int_val, target_ty, "int_sext").unwrap();
            } else {
                return builder.build_int_z_extend(int_val, target_ty, "int_zext").unwrap();
            }
        }

        if val.is_pointer_value() {
            return builder
                .build_ptr_to_int(val.into_pointer_value(), target_ty, "ptr2int")
                .unwrap();
        }

        if val.is_float_value() {
            if Self::is_signed(source_ty) {
                return builder
                    .build_float_to_signed_int(val.into_float_value(), target_ty, "float_to_int")
                    .unwrap();
            } else {
                return builder
                    .build_float_to_unsigned_int(val.into_float_value(), target_ty, "float_to_uint")
                    .unwrap();
            }
        }

        if val.is_struct_value() {
            let struct_val = val.into_struct_value();
            let first_field = builder
                .build_extract_value(struct_val, 0, "struct_first_field")
                .unwrap();
            if first_field.is_pointer_value() {
                return builder
                    .build_ptr_to_int(first_field.into_pointer_value(), target_ty, "struct_ptr2int")
                    .unwrap();
            }
            return builder
                .build_int_cast(first_field.into_int_value(), target_ty, "struct_int_cast")
                .unwrap();
        }

        panic!("LLVM ERROR: Unsupported value type for int conversion.");
    }

    fn value_as_ptr(
        builder: &Builder<'ctx>,
        val: BasicValueEnum<'ctx>,
        target_ptr_ty: PointerType<'ctx>
    ) -> PointerValue<'ctx> {
        if val.is_pointer_value() {
            let ptr_val = val.into_pointer_value();
            if ptr_val.get_type() == target_ptr_ty {
                return ptr_val;
            }
            return builder.build_pointer_cast(ptr_val, target_ptr_ty, "ptr_cast").unwrap();
        }

        if val.is_int_value() {
            return builder
                .build_int_to_ptr(val.into_int_value(), target_ptr_ty, "int_to_ptr")
                .unwrap();
        }

        if val.is_struct_value() {
            let struct_val = val.into_struct_value();
            let first_field = builder
                .build_extract_value(struct_val, 0, "struct_first_field")
                .unwrap();
            if first_field.is_pointer_value() {
                let ptr_val = first_field.into_pointer_value();
                if ptr_val.get_type() == target_ptr_ty {
                    return ptr_val;
                }
                return builder
                    .build_pointer_cast(ptr_val, target_ptr_ty, "struct_ptr_cast")
                    .unwrap();
            }
            return builder
                .build_int_to_ptr(first_field.into_int_value(), target_ptr_ty, "struct_inttoptr")
                .unwrap();
        }

        panic!("LLVM ERROR: Unsupported value type for pointer conversion.");
    }

    fn value_as_float(
        builder: &Builder<'ctx>,
        val: BasicValueEnum<'ctx>,
        target_ty: FloatType<'ctx>
    ) -> FloatValue<'ctx> {
        if val.is_float_value() {
            let float_val = val.into_float_value();
            if float_val.get_type() == target_ty {
                return float_val;
            }
            return builder.build_float_cast(float_val, target_ty, "float_cast").unwrap();
        }

        if val.is_int_value() {
            // Note: Since we don't have source_ty here easily, default to signed conversion
            return builder
                .build_signed_int_to_float(val.into_int_value(), target_ty, "sitofp")
                .unwrap();
        }

        panic!("LLVM ERROR: Expected numeric value for float conversion.");
    }

    fn promote_float_pair(
        builder: &Builder<'ctx>,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>
    ) -> (FloatValue<'ctx>, FloatValue<'ctx>) {
        let target_ty = if left.is_float_value() {
            left.into_float_value().get_type()
        } else if right.is_float_value() {
            right.into_float_value().get_type()
        } else {
            panic!("LLVM ERROR: Expected at least one float operand for float promotion.");
        };

        (
            Self::value_as_float(builder, left, target_ty),
            Self::value_as_float(builder, right, target_ty),
        )
    }

    fn get_llvm_type(&self, ty: &IrType) -> BasicTypeEnum<'ctx> {
        match ty {
            IrType::Void => {
                panic!("ICE: Attempted to convert IrType::Void into an LLVM BasicType!")
            }
            IrType::I8 | IrType::U8 => self.context.i8_type().into(),
            IrType::I16 | IrType::U16 => self.context.i16_type().into(),
            IrType::I32 | IrType::U32 => self.context.i32_type().into(),
            IrType::I64 | IrType::U64 | IrType::Any => self.context.i64_type().into(),
            IrType::F16 => self.context.f16_type().into(),
            IrType::F32 => self.context.f32_type().into(),
            IrType::F64 => self.context.f64_type().into(),
            IrType::Bool => self.context.bool_type().into(),

            IrType::Ptr(_) | IrType::Array(_, _) | IrType::Closure(_, _) | IrType::FatPtr =>
                self.context.i8_type().ptr_type(inkwell::AddressSpace::default()).into(),

            IrType::Struct(name, field_types) => {
                let effective_fields = if !field_types.is_empty() {
                    field_types.clone()
                } else {
                    self.struct_layouts.get(name).cloned().unwrap_or_default()
                };

                if effective_fields.is_empty() {
                    if let Some(existing_struct_ty) = self.module.get_struct_type(name) {
                        if existing_struct_ty.count_fields() > 0 {
                            return existing_struct_ty.as_basic_type_enum();
                        }
                    }
                    return self.context.i64_type().into();
                }

                let struct_ty = self.module
                    .get_struct_type(name)
                    .unwrap_or_else(|| self.context.opaque_struct_type(name));

                if struct_ty.count_fields() == 0 {
                    let llvm_fields: Vec<BasicTypeEnum> = effective_fields
                        .iter()
                        .map(|field_ty| self.get_llvm_type(field_ty))
                        .collect();
                    struct_ty.set_body(&llvm_fields, false);
                }

                struct_ty.as_basic_type_enum()
            }
        }
    }

    // ABI and memory cells keep their declared width. Integer SSA values use
    // the custom IR's word representation, with the source signedness intact.
    fn storage_value(&self, value: BasicValueEnum<'ctx>, ty: &IrType) -> BasicValueEnum<'ctx> {
        let t = self.get_llvm_type(ty);
        if t.is_int_type() {
            Self::value_as_int(&self.builder, value, ty, t.into_int_type()).into()
        } else if t.is_pointer_type() {
            Self::value_as_ptr(&self.builder, value, t.into_pointer_type()).into()
        } else {
            value
        }
    }
    fn register_value(&self, value: BasicValueEnum<'ctx>, ty: &IrType) -> BasicValueEnum<'ctx> {
        if value.is_int_value() && value.into_int_value().get_type().get_bit_width() < 64 {
            Self::value_as_int(&self.builder, value, ty, self.context.i64_type()).into()
        } else {
            value
        }
    }

    // A dominating definition need not precede its use in the block vector.
    // Reverse postorder visits reachable dominators before their uses; LLVM
    // blocks themselves retain their original identities and entry position.
    fn block_emission_order(func: &crate::ir::FunctionIr) -> Vec<usize> {
        let indices: HashMap<_, _> = func.blocks
            .iter()
            .enumerate()
            .map(|(i, b)| (b.id, i))
            .collect();
        let mut seen = std::collections::HashSet::new();
        let mut postorder = Vec::new();
        let mut work = if func.blocks.is_empty() { vec![] } else { vec![(0, false)] };
        while let Some((i, expanded)) = work.pop() {
            if expanded {
                postorder.push(i);
                continue;
            }
            if !seen.insert(i) {
                continue;
            }
            work.push((i, true));
            match func.blocks[i].instructions.last() {
                Some(Instruction::Br { target }) => work.push((indices[target], false)),
                Some(Instruction::CondBr { if_true, if_false, .. }) => {
                    work.push((indices[if_false], false));
                    work.push((indices[if_true], false));
                }
                _ => {}
            }
        }
        postorder.reverse();
        postorder.extend((0..func.blocks.len()).filter(|i| !seen.contains(i)));
        postorder
    }

    fn managed_type(ty: &IrType) -> bool {
        matches!(ty, IrType::Array(..) | IrType::Closure(..) | IrType::FatPtr) ||
            matches!(ty,IrType::Ptr(inner) if matches!(inner.as_ref(),IrType::Struct(name,_) if !name.starts_with("__fishy_range_")))
    }

    // Managed payloads carry [destructor pointer, refcount, length] immediately
    // before the data. Length/refcount keep their established -1/-2 offsets.
    // Single-threaded checked ARC. Only valid owned or pinned handles may be
    // retained; zero-to-live resurrection and count overflow remain fatal.
    fn create_arc_helpers(&self) {
        let ptr = self.context.i8_type().ptr_type(inkwell::AddressSpace::default());
        let word = self.context.i64_type();
        let signature = self.context.void_type().fn_type(&[ptr.into()], false);
        for (name, increase) in [
            ("__fishy_retain", true),
            ("__fishy_release", false),
        ] {
            let f = self.module.add_function(
                name,
                signature,
                Some(inkwell::module::Linkage::Internal)
            );
            let b = self.context.create_builder();
            let data = f.get_first_param().unwrap().into_pointer_value();
            let entry = self.context.append_basic_block(f, "entry");
            let active = self.context.append_basic_block(f, "active");
            let invalid = self.context.append_basic_block(f, "invalid");
            let update = self.context.append_basic_block(f, "update");
            let end = self.context.append_basic_block(f, "end");
            b.position_at_end(entry);
            let null = b.build_is_null(data, "null").unwrap();
            b.build_conditional_branch(null, end, active).unwrap();
            b.position_at_end(active);
            let rc = unsafe {
                b.build_gep(word, data, &[word.const_int(-2i64 as u64, true)], "rc").unwrap()
            };
            let count = b.build_load(word, rc, "count").unwrap().into_int_value();
            let zero = b
                .build_int_compare(
                    inkwell::IntPredicate::EQ,
                    count,
                    word.const_zero(),
                    "count.zero"
                )
                .unwrap();
            let bad = if increase {
                let full = b
                    .build_int_compare(
                        inkwell::IntPredicate::UGE,
                        count,
                        word.const_int(i64::MAX as u64, false),
                        "count.full"
                    )
                    .unwrap();
                b.build_or(zero, full, "invalid.count").unwrap()
            } else {
                zero
            };
            b.build_conditional_branch(bad, invalid, update).unwrap();
            b.position_at_end(invalid);
            let trap = inkwell::intrinsics::Intrinsic
                ::find("llvm.trap")
                .unwrap()
                .get_declaration(&self.module, &[])
                .unwrap();
            b.build_call(trap, &[], "").unwrap();
            b.build_unreachable().unwrap();
            b.position_at_end(update);
            let one = word.const_int(1, false);
            let next = (
                if increase {
                    b.build_int_add(count, one, "retained")
                } else {
                    b.build_int_sub(count, one, "released")
                }
            ).unwrap();
            b.build_store(rc, next).unwrap();
            if increase {
                b.build_unconditional_branch(end).unwrap();
            } else {
                let destroy = self.context.append_basic_block(f, "destroy");
                let invoke = self.context.append_basic_block(f, "invoke");
                let free = self.context.append_basic_block(f, "free");
                let last = b
                    .build_int_compare(inkwell::IntPredicate::EQ, count, one, "last.owner")
                    .unwrap();
                b.build_conditional_branch(last, destroy, end).unwrap();
                b.position_at_end(destroy);
                let header = unsafe {
                    b.build_gep(
                        word,
                        data,
                        &[word.const_int(-3i64 as u64, true)],
                        "header"
                    ).unwrap()
                };
                let destructor = b
                    .build_load(ptr, header, "destructor")
                    .unwrap()
                    .into_pointer_value();
                let exists = b.build_is_not_null(destructor, "has.destructor").unwrap();
                b.build_conditional_branch(exists, invoke, free).unwrap();
                b.position_at_end(invoke);
                b.build_indirect_call(signature, destructor, &[data.into()], "").unwrap();
                b.build_unconditional_branch(free).unwrap();
                b.position_at_end(free);
                b.build_call(
                    self.module.get_function("free").expect("stdlib free"),
                    &[header.into()],
                    ""
                ).unwrap();
                b.build_unconditional_branch(end).unwrap();
            }
            b.position_at_end(end);
            b.build_return(None).unwrap();
        }
    }

    fn destructor(
        &self,
        name: &str,
        fields: &[IrType],
        array: Option<&IrType>
    ) -> Option<FunctionValue<'ctx>> {
        if let Some(element) = self.resources.get(name) {
            return Some(self.vector_destructor(name, element));
        }
        if !fields.iter().any(Self::managed_type) && !array.is_some_and(Self::managed_type) {
            return None;
        }
        let name = format!("__fishy_destroy_{name}");
        if let Some(func) = self.module.get_function(&name) {
            return Some(func);
        }
        let ptr = self.context.i8_type().ptr_type(inkwell::AddressSpace::default());
        let word = self.context.i64_type();
        let func = self.module.add_function(
            &name,
            self.context.void_type().fn_type(&[ptr.into()], false),
            Some(inkwell::module::Linkage::Internal)
        );
        let b = self.context.create_builder();
        let data = func.get_first_param().unwrap().into_pointer_value();
        let entry = self.context.append_basic_block(func, "entry");
        b.position_at_end(entry);
        let release = self.module.get_function("__fishy_release").unwrap();
        if let Some(element) = array {
            let lenptr = unsafe {
                b.build_gep(word, data, &[word.const_int(-1_i64 as u64, true)], "lenptr").unwrap()
            };
            let len = b.build_load(word, lenptr, "len").unwrap().into_int_value();
            let index = b.build_alloca(word, "index").unwrap();
            b.build_store(index, len).unwrap();
            let test = self.context.append_basic_block(func, "test");
            let body = self.context.append_basic_block(func, "body");
            let end = self.context.append_basic_block(func, "end");
            b.build_unconditional_branch(test).unwrap();
            b.position_at_end(test);
            let i = b.build_load(word, index, "i").unwrap().into_int_value();
            let cond = b
                .build_int_compare(inkwell::IntPredicate::NE, i, word.const_zero(), "more")
                .unwrap();
            b.build_conditional_branch(cond, body, end).unwrap();
            b.position_at_end(body);
            let previous = b.build_int_sub(i, word.const_int(1, false), "previous").unwrap();
            let slot = unsafe {
                b.build_gep(self.get_llvm_type(element), data, &[previous], "element").unwrap()
            };
            let value = b.build_load(ptr, slot, "child").unwrap();
            b.build_call(release, &[value.into()], "").unwrap();
            b.build_store(index, previous).unwrap();
            b.build_unconditional_branch(test).unwrap();
            b.position_at_end(end);
        } else {
            let layout = self.context.struct_type(
                &fields
                    .iter()
                    .map(|t| self.get_llvm_type(t))
                    .collect::<Vec<_>>(),
                false
            );
            for (i, ty) in fields
                .iter()
                .enumerate()
                .rev()
                .filter(|(_, t)| Self::managed_type(t)) {
                let slot = b.build_struct_gep(layout, data, i as u32, "field").unwrap();
                let value = b.build_load(self.get_llvm_type(ty), slot, "child").unwrap();
                b.build_call(release, &[value.into()], "").unwrap();
            }
        }
        b.build_return(None).unwrap();
        Some(func)
    }

    // The prelude Vec contract owns exactly the initialized prefix [0,count)
    // and its raw allocation. Pointer managedness cannot express this resource.
    fn vector_destructor(&self, name: &str, element: &IrType) -> FunctionValue<'ctx> {
        let symbol = format!("__fishy_destroy_{name}");
        if let Some(function) = self.module.get_function(&symbol) {
            return function;
        }
        let pointer = self.context.i8_type().ptr_type(inkwell::AddressSpace::default());
        let word = self.context.i64_type();
        let function = self.module.add_function(
            &symbol,
            self.context.void_type().fn_type(&[pointer.into()], false),
            Some(inkwell::module::Linkage::Internal)
        );
        let b = self.context.create_builder();
        let entry = self.context.append_basic_block(function, "entry");
        b.position_at_end(entry);
        let value = function.get_first_param().unwrap().into_pointer_value();
        let layout = self.context.struct_type(&[pointer.into(), word.into(), word.into()], false);
        let data_cell = b.build_struct_gep(layout, value, 0, "buffer.cell").unwrap();
        let count_cell = b.build_struct_gep(layout, value, 1, "count.cell").unwrap();
        let capacity_cell = b.build_struct_gep(layout, value, 2, "capacity.cell").unwrap();
        let data = b.build_load(pointer, data_cell, "buffer").unwrap().into_pointer_value();
        let count = b.build_load(word, count_cell, "initialized").unwrap().into_int_value();
        b.build_store(data_cell, pointer.const_null()).unwrap();
        b.build_store(count_cell, word.const_zero()).unwrap();
        b.build_store(capacity_cell, word.const_zero()).unwrap();
        if Self::managed_type(element) {
            let counter = b.build_alloca(word, "remaining").unwrap();
            b.build_store(counter, count).unwrap();
            let test = self.context.append_basic_block(function, "test");
            let body = self.context.append_basic_block(function, "drop.element");
            let end = self.context.append_basic_block(function, "free.buffer");
            b.build_unconditional_branch(test).unwrap();
            b.position_at_end(test);
            let remaining = b.build_load(word, counter, "remaining").unwrap().into_int_value();
            let more = b
                .build_int_compare(inkwell::IntPredicate::NE, remaining, word.const_zero(), "more")
                .unwrap();
            b.build_conditional_branch(more, body, end).unwrap();
            b.position_at_end(body);
            let index = b.build_int_sub(remaining, word.const_int(1, false), "index").unwrap();
            let cell = unsafe {
                b.build_gep(self.get_llvm_type(element), data, &[index], "element.cell").unwrap()
            };
            let child = b.build_load(pointer, cell, "element").unwrap();
            b.build_store(cell, pointer.const_null()).unwrap();
            b.build_call(
                self.module.get_function("__fishy_release").unwrap(),
                &[child.into()],
                ""
            ).unwrap();
            b.build_store(counter, index).unwrap();
            b.build_unconditional_branch(test).unwrap();
            b.position_at_end(end);
        }
        b.build_call(self.module.get_function("free").unwrap(), &[data.into()], "").unwrap();
        b.build_return(None).unwrap();
        function
    }

    fn managed_alloc(
        &self,
        bytes: IntValue<'ctx>,
        length: IntValue<'ctx>,
        destructor: Option<FunctionValue<'ctx>>
    ) -> PointerValue<'ctx> {
        let word = self.context.i64_type();
        let ptr = self.context.i8_type().ptr_type(inkwell::AddressSpace::default());
        let total = self.builder
            .build_int_add(bytes, word.const_int(24, false), "allocation.bytes")
            .unwrap();
        let raw = self.builder
            .build_call(
                self.module.get_function("malloc").unwrap(),
                &[total.into()],
                "managed.alloc"
            )
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();
        // OOM is a runtime failure, never a write through null.
        let parent = self.builder.get_insert_block().unwrap().get_parent().unwrap();
        let ok = self.context.append_basic_block(parent, "allocation.ok");
        let bad = self.context.append_basic_block(parent, "allocation.failed");
        let nonnull = self.builder.build_is_not_null(raw, "allocated").unwrap();
        self.builder.build_conditional_branch(nonnull, ok, bad).unwrap();
        self.builder.position_at_end(bad);
        let message = self.builder.build_global_string_ptr("Out of memory", "oom.message").unwrap();
        self.builder
            .build_call(
                self.module.get_function("panic").unwrap(),
                &[message.as_pointer_value().into()],
                ""
            )
            .unwrap();
        self.builder.build_unreachable().unwrap();
        self.builder.position_at_end(ok);
        let dtor = destructor
            .map(|f| f.as_global_value().as_pointer_value())
            .unwrap_or_else(|| ptr.const_null());
        self.builder.build_store(raw, dtor).unwrap();
        let rc = unsafe {
            self.builder.build_gep(word, raw, &[word.const_int(1, false)], "rc").unwrap()
        };
        self.builder.build_store(rc, word.const_int(1, false)).unwrap();
        let len = unsafe {
            self.builder.build_gep(word, raw, &[word.const_int(2, false)], "length").unwrap()
        };
        self.builder.build_store(len, length).unwrap();
        let data = unsafe {
            self.builder.build_gep(word, raw, &[word.const_int(3, false)], "data").unwrap()
        };
        self.builder.build_memset(data, 1, self.context.i8_type().const_zero(), bytes).unwrap();
        data
    }

    pub fn compile(
        &mut self,
        ir_module: &ModuleIr,
        external_names: &HashMap<String, String>
    ) -> Result<(), String> {
        self.compile_expanded(ir_module, external_names)
    }

    fn compile_expanded(
        &mut self,
        ir_module: &ModuleIr,
        external_names: &HashMap<String, String>
    ) -> Result<(), String> {
        self.collect_struct_layouts(ir_module);
        for (name, field_types) in &self.struct_layouts {
            self.get_llvm_type(&IrType::Struct(name.clone(), field_types.clone()));
        }

        let mut llvm_funcs: HashMap<String, inkwell::values::FunctionValue<'ctx>> = HashMap::new();

        for func in &ir_module.functions {
            let mut param_types: Vec<BasicMetadataTypeEnum> = Vec::new();

            for (_, arg_ty) in &func.args {
                let llvm_arg_ty = match arg_ty {
                    IrType::Struct(name, _) if !name.starts_with("__fishy_range_") =>
                        self.context.i64_type().ptr_type(inkwell::AddressSpace::default()).into(),
                    _ => self.get_llvm_type(arg_ty).into(),
                };
                param_types.push(llvm_arg_ty);
            }

            let is_variadic = func.is_variadic;

            let fn_type = if func.ret_type == IrType::Void {
                self.context.void_type().fn_type(param_types.as_slice(), is_variadic)
            } else {
                let llvm_ret_ty = match &func.ret_type {
                    IrType::Struct(name, _) if !name.starts_with("__fishy_range_") =>
                        self.context
                            .i64_type()
                            .ptr_type(inkwell::AddressSpace::default())
                            .as_basic_type_enum(),
                    _ => self.get_llvm_type(&func.ret_type),
                };

                llvm_ret_ty.fn_type(param_types.as_slice(), is_variadic)
            };

            let link_name = external_names.get(&func.name).unwrap_or(&func.name);
            let llvm_func = if let Some(existing) = self.module.get_function(link_name) {
                if existing.get_type() != fn_type {
                    return Err(format!("Conflicting external signatures for '{link_name}'."));
                }
                existing
            } else {
                self.module.add_function(link_name, fn_type, None)
            };
            llvm_funcs.insert(func.name.clone(), llvm_func);
        }

        // Effectful intrinsic instructions reference private ABI operations during
        // backend expansion. Keep the central registry available after custom
        // reachability has removed unreferenced source declarations.
        self.create_arc_helpers();
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
        let as_ptr = |builder: &Builder<'ctx>, val: BasicValueEnum<'ctx>| -> PointerValue<'ctx> {
            LlvmEmitter::value_as_ptr(
                builder,
                val,
                ctx.i64_type().ptr_type(inkwell::AddressSpace::default())
            )
        };

        let as_int = |
            builder: &Builder<'ctx>,
            val: BasicValueEnum<'ctx>,
            source_ty: &IrType
        | -> IntValue<'ctx> {
            LlvmEmitter::value_as_int(builder, val, source_ty, ctx.i64_type())
        };

        let as_float = |
            builder: &Builder<'ctx>,
            val: BasicValueEnum<'ctx>,
            target_ty: inkwell::types::FloatType<'ctx>
        | -> FloatValue<'ctx> {
            LlvmEmitter::value_as_float(builder, val, target_ty)
        };

        let promote_float_pair = |
            builder: &Builder<'ctx>,
            left: BasicValueEnum<'ctx>,
            right: BasicValueEnum<'ctx>
        | -> (FloatValue<'ctx>, FloatValue<'ctx>) {
            let left_is_float = left.is_float_value();
            let right_is_float = right.is_float_value();
            let target_ty = if left_is_float {
                left.into_float_value().get_type()
            } else if right_is_float {
                right.into_float_value().get_type()
            } else {
                panic!("LLVM ERROR: Expected at least one float operand for float promotion.");
            };

            (as_float(builder, left, target_ty), as_float(builder, right, target_ty))
        };

        let mut global_registers = HashMap::new();
        for (reg, ty) in &ir_module.globals {
            let ty = self.get_llvm_type(ty);
            let g = self.module.add_global(ty, None, &format!("global_{}", reg.0));
            g.set_initializer(&ty.const_zero());
            global_registers.insert(*reg, g.as_pointer_value().into());
        }
        for func in &ir_module.functions {
            let llvm_func = *llvm_funcs
                .get(&func.name)
                .ok_or_else(|| {
                    format!("LLVM Error: Function '{}' not found in the functions map!", func.name)
                })?;

            self.registers = global_registers.clone();

            let mut llvm_blocks = HashMap::new();
            for block in &func.blocks {
                let bb = self.context.append_basic_block(llvm_func, &format!("bb{}", block.id.0));
                llvm_blocks.insert(block.id.0, bb);
            }

            if let Some(entry) = func.blocks.first() {
                self.builder.position_at_end(llvm_blocks[&entry.id.0]);
                for (i, (reg, ty)) in func.args.iter().enumerate() {
                    let val = self.register_value(llvm_func.get_nth_param(i as u32).unwrap(), ty);
                    self.registers.insert(*reg, val);
                }
            }

            for index in Self::block_emission_order(func) {
                let block = &func.blocks[index];
                let current_bb = *llvm_blocks
                    .get(&block.id.0)
                    .ok_or_else(|| {
                        format!("LLVM ERROR: Basic block 'bb{}' not found!", block.id.0)
                    })?;
                self.builder.position_at_end(current_bb);

                for inst in &block.instructions {
                    match inst {
                        Instruction::SizeOf { dest, ty } => {
                            let size = self.target_machine
                                .get_target_data()
                                .get_abi_size(&self.get_llvm_type(ty));
                            self.registers.insert(
                                *dest,
                                self.context.i64_type().const_int(size, false).into()
                            );
                        }
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
                            let global_str_ptr = self.get_or_create_string_global(value);
                            self.registers.insert(*dest, global_str_ptr.into());
                        }

                        Instruction::Alloca { dest, name, ty } => {
                            if global_registers.contains_key(dest) {
                                continue;
                            }
                            let llvm_ty = self.get_llvm_type(ty);
                            let entry = self.builder
                                .get_insert_block()
                                .unwrap()
                                .get_parent()
                                .unwrap()
                                .get_first_basic_block()
                                .unwrap();
                            let allocations = self.context.create_builder();
                            if let Some(first) = entry.get_first_instruction() {
                                allocations.position_before(&first);
                            } else {
                                allocations.position_at_end(entry);
                            }
                            let ptr = allocations.build_alloca(llvm_ty, name).unwrap();
                            self.registers.insert(*dest, ptr.into());
                        }

                        Instruction::AllocArray { dest, size, ty } => {
                            let size = as_int(&self.builder, self.registers[size], &IrType::U64);
                            let stride = self.target_machine
                                .get_target_data()
                                .get_abi_size(&self.get_llvm_type(ty));
                            let bytes = self.builder
                                .build_int_mul(
                                    size,
                                    self.context.i64_type().const_int(stride, false),
                                    "array.bytes"
                                )
                                .unwrap();
                            let destructor = self.destructor(
                                &format!("array_{ty:?}"),
                                &[],
                                Some(ty)
                            );
                            let value = self.managed_alloc(bytes, size, destructor);
                            self.registers.insert(*dest, value.into());
                        }
                        Instruction::AllocStruct { dest, class_name } => {
                            let fields = self.struct_layouts
                                .get(class_name)
                                .ok_or_else(|| format!("Missing layout {class_name}"))?
                                .clone();
                            let layout = self.get_llvm_type(
                                &IrType::Struct(class_name.clone(), fields.clone())
                            );
                            let mut bytes = self.target_machine
                                .get_target_data()
                                .get_abi_size(&layout)
                                .max(1);
                            if let Some((owner, _)) = class_name.split_once('#') {
                                // A managed enum reference keeps allocation identity when
                                // replacing one variant by another. Reserve every variant's
                                // maximum payload capacity, with typed per-variant access.
                                let prefix = format!("{owner}#");
                                for (name, fields) in &self.struct_layouts {
                                    if name.starts_with(&prefix) {
                                        let ty = self.get_llvm_type(
                                            &IrType::Struct(name.clone(), fields.clone())
                                        );
                                        bytes = bytes.max(
                                            self.target_machine.get_target_data().get_abi_size(&ty)
                                        );
                                    }
                                }
                            }
                            let destructor = self.destructor(class_name, &fields, None);
                            let value = self.managed_alloc(
                                self.context.i64_type().const_int(bytes, false),
                                self.context.i64_type().const_zero(),
                                destructor
                            );
                            self.registers.insert(*dest, value.into());
                        }

                        Instruction::GetElementPtr { dest, base_ty, base_ptr, indices } => {
                            let base_val = *self.registers
                                .get(base_ptr)
                                .ok_or_else(|| {
                                    format!("LLVM ERROR: Base register '{}' missing in GEP instruction!", base_ptr)
                                })?;

                            let llvm_base_ty = match base_ty {
                                IrType::Struct(name, field_types) => {
                                    self.get_llvm_type(
                                        &IrType::Struct(name.clone(), if !field_types.is_empty() {
                                            field_types.clone()
                                        } else {
                                            self.struct_layouts
                                                .get(name)
                                                .cloned()
                                                .unwrap_or_default()
                                        })
                                    )
                                }
                                _ => self.get_llvm_type(base_ty),
                            };
                            let ptr_ty = llvm_base_ty.ptr_type(inkwell::AddressSpace::default());

                            let ptr_val = if base_val.is_pointer_value() {
                                let raw_ptr_val = base_val.into_pointer_value();
                                if raw_ptr_val.get_type() == ptr_ty {
                                    raw_ptr_val
                                } else {
                                    self.builder
                                        .build_pointer_cast(raw_ptr_val, ptr_ty, "gep_ptr_cast")
                                        .unwrap()
                                }
                            } else {
                                self.builder
                                    .build_int_to_ptr(
                                        as_int(&self.builder, base_val, &IrType::I64),
                                        ptr_ty,
                                        "inttoptr"
                                    )
                                    .unwrap()
                            };

                            let mut llvm_indices = Vec::new();

                            if matches!(base_ty, IrType::Struct(_, _)) && indices.len() == 2 {
                                let idx0_val = *self.registers.get(&indices[0]).unwrap();
                                let idx0_int = as_int(&self.builder, idx0_val, &IrType::I64);
                                let idx0_i64 = self.builder
                                    .build_int_cast(
                                        idx0_int,
                                        self.context.i64_type(),
                                        "field_idx0_i64"
                                    )
                                    .unwrap();
                                llvm_indices.push(idx0_i64);

                                let idx1_val = *self.registers.get(&indices[1]).unwrap();
                                let field_idx_int = as_int(&self.builder, idx1_val, &IrType::I64);
                                let field_idx_i32 = self.builder
                                    .build_int_cast(
                                        field_idx_int,
                                        self.context.i32_type(),
                                        "field_idx_i32"
                                    )
                                    .unwrap();
                                llvm_indices.push(field_idx_i32);
                            } else {
                                for idx in indices {
                                    let idx_val = *self.registers
                                        .get(idx)
                                        .ok_or_else(||
                                            format!("LLVM ERROR: Index register '{}' missing in GEP instruction!", idx)
                                        )?;
                                    let idx_int = as_int(&self.builder, idx_val, &IrType::I64);
                                    let idx_i64 = if idx_int.get_type() == self.context.i64_type() {
                                        idx_int
                                    } else {
                                        self.builder
                                            .build_int_cast(
                                                idx_int,
                                                self.context.i64_type(),
                                                "gep_idx_i64"
                                            )
                                            .unwrap()
                                    };
                                    llvm_indices.push(idx_i64);
                                }
                            }

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
                                .ok_or_else(|| {
                                    format!("LLVM ERROR: Pointer '{}' missing in Store instruction!", ptr)
                                })?;

                            let ptr_val: inkwell::values::PointerValue<'ctx>;
                            let target_llvm_ty = self.get_llvm_type(ty);
                            let target_ptr_ty = target_llvm_ty.ptr_type(
                                inkwell::AddressSpace::default()
                            );

                            if ptr_raw.is_pointer_value() {
                                let raw_ptr_val = ptr_raw.into_pointer_value();
                                if raw_ptr_val.get_type() == target_ptr_ty {
                                    ptr_val = raw_ptr_val;
                                } else {
                                    ptr_val = self.builder
                                        .build_pointer_cast(
                                            raw_ptr_val,
                                            target_ptr_ty,
                                            "store_ptr_cast"
                                        )
                                        .unwrap();
                                }
                            } else {
                                ptr_val = self.builder
                                    .build_int_to_ptr(
                                        ptr_raw.into_int_value(),
                                        target_ptr_ty,
                                        "inttoptr_store"
                                    )
                                    .unwrap();
                            }

                            let mut val = *self.registers
                                .get(value)
                                .ok_or_else(|| {
                                    format!("LLVM ERROR: Value '{}' missing in Store instruction!", value)
                                })?;

                            if val.is_int_value() && target_llvm_ty.is_struct_type() {
                                if val.into_int_value().get_zero_extended_constant() == Some(0) {
                                    val = target_llvm_ty.into_struct_type().const_zero().into();
                                } else {
                                    return Err(
                                        "Nonzero scalar cannot initialize aggregate storage".into()
                                    );
                                }
                            } else if val.is_pointer_value() && target_llvm_ty.is_int_type() {
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
                            } else if val.is_pointer_value() && target_llvm_ty.is_pointer_type() {
                                let val_ptr = val.into_pointer_value();
                                let expected_ptr_ty = target_llvm_ty.into_pointer_type();
                                if val_ptr.get_type() != expected_ptr_ty {
                                    val = self.builder
                                        .build_pointer_cast(
                                            val_ptr,
                                            expected_ptr_ty,
                                            "val_ptr_cast"
                                        )
                                        .unwrap()
                                        .into();
                                }
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
                                    val = if Self::is_signed(ty) {
                                        self.builder
                                            .build_int_s_extend(val_int, target_int, "store_sext")
                                            .unwrap()
                                            .into()
                                    } else {
                                        self.builder
                                            .build_int_z_extend(val_int, target_int, "store_zext")
                                            .unwrap()
                                            .into()
                                    };
                                }
                            } else if val.is_int_value() && target_llvm_ty.is_float_type() {
                                val = if Self::is_signed(ty) {
                                    self.builder
                                        .build_signed_int_to_float(
                                            val.into_int_value(),
                                            target_llvm_ty.into_float_type(),
                                            "store_sitofp"
                                        )
                                        .unwrap()
                                        .into()
                                } else {
                                    self.builder
                                        .build_unsigned_int_to_float(
                                            val.into_int_value(),
                                            target_llvm_ty.into_float_type(),
                                            "store_uitofp"
                                        )
                                        .unwrap()
                                        .into()
                                };
                            } else if val.is_float_value() && target_llvm_ty.is_int_type() {
                                val = if Self::is_signed(ty) {
                                    self.builder
                                        .build_float_to_signed_int(
                                            val.into_float_value(),
                                            target_llvm_ty.into_int_type(),
                                            "store_fptosi"
                                        )
                                        .unwrap()
                                        .into()
                                } else {
                                    self.builder
                                        .build_float_to_unsigned_int(
                                            val.into_float_value(),
                                            target_llvm_ty.into_int_type(),
                                            "store_fptoui"
                                        )
                                        .unwrap()
                                        .into()
                                };
                            } else if val.is_float_value() && target_llvm_ty.is_float_type() {
                                let val_float = val.into_float_value();
                                let target_float = target_llvm_ty.into_float_type();
                                if val_float.get_type() != target_float {
                                    val = self.builder
                                        .build_float_cast(val_float, target_float, "store_fcast")
                                        .unwrap()
                                        .into();
                                }
                            }

                            self.builder.build_store(ptr_val, val).unwrap();
                        }

                        Instruction::Load { dest, ty, src_ptr } => {
                            let ptr_raw = *self.registers
                                .get(src_ptr)
                                .ok_or_else(|| {
                                    format!("LLVM ERROR: Source pointer '{}' missing in Load instruction!", src_ptr)
                                })?;

                            let llvm_ty = self.get_llvm_type(ty);
                            let ptr_ty = llvm_ty.ptr_type(inkwell::AddressSpace::default());
                            let ptr_val = if ptr_raw.is_pointer_value() {
                                let raw_ptr_val = ptr_raw.into_pointer_value();
                                if raw_ptr_val.get_type() == ptr_ty {
                                    raw_ptr_val
                                } else {
                                    self.builder
                                        .build_pointer_cast(raw_ptr_val, ptr_ty, "load_ptr_cast")
                                        .unwrap()
                                }
                            } else {
                                self.builder
                                    .build_int_to_ptr(ptr_raw.into_int_value(), ptr_ty, "inttoptr")
                                    .unwrap()
                            };
                            let val = self.builder.build_load(llvm_ty, ptr_val, "load").unwrap();

                            let final_val = if
                                val.is_int_value() &&
                                val.into_int_value().get_type().get_bit_width() < 64
                            {
                                let int_val = val.into_int_value();
                                if Self::is_signed(ty) {
                                    self.builder
                                        .build_int_s_extend(
                                            int_val,
                                            self.context.i64_type(),
                                            "load_sext"
                                        )
                                        .unwrap()
                                        .into()
                                } else {
                                    self.builder
                                        .build_int_z_extend(
                                            int_val,
                                            self.context.i64_type(),
                                            "load_zext"
                                        )
                                        .unwrap()
                                        .into()
                                }
                            } else {
                                val
                            };

                            self.registers.insert(*dest, final_val);
                        }

                        Instruction::Cast { dest, value, source_ty, target_ty } => {
                            let val = *self.registers
                                .get(value)
                                .ok_or_else(|| {
                                    format!("LLVM ERROR: Value register '{}' missing in Cast instruction!", value)
                                })?;

                            let llvm_target_ty = self.get_llvm_type(target_ty);
                            if
                                val.get_type() == llvm_target_ty &&
                                matches!(target_ty, IrType::Struct(..))
                            {
                                self.registers.insert(*dest, val);
                                continue;
                            }

                            let casted = if val.is_int_value() && llvm_target_ty.is_int_type() {
                                let val_int = val.into_int_value();
                                let target_int = llvm_target_ty.into_int_type();

                                if val_int.get_type().get_bit_width() < target_int.get_bit_width() {
                                    if Self::is_signed(source_ty) {
                                        self.builder
                                            .build_int_s_extend(val_int, target_int, "cast_sext")
                                            .unwrap()
                                            .into()
                                    } else {
                                        self.builder
                                            .build_int_z_extend(val_int, target_int, "cast_zext")
                                            .unwrap()
                                            .into()
                                    }
                                } else if
                                    val_int.get_type().get_bit_width() > target_int.get_bit_width()
                                {
                                    self.builder
                                        .build_int_truncate(val_int, target_int, "cast_trunc")
                                        .unwrap()
                                        .into()
                                } else {
                                    val_int.into()
                                }
                            } else if val.is_int_value() && llvm_target_ty.is_pointer_type() {
                                let ptr_ty = llvm_target_ty.into_pointer_type();
                                self.builder
                                    .build_int_to_ptr(val.into_int_value(), ptr_ty, "inttoptr")
                                    .unwrap()
                                    .into()
                            } else if val.is_int_value() && llvm_target_ty.is_float_type() {
                                if Self::is_signed(source_ty) {
                                    self.builder
                                        .build_signed_int_to_float(
                                            val.into_int_value(),
                                            llvm_target_ty.into_float_type(),
                                            "cast_sitofp"
                                        )
                                        .unwrap()
                                        .into()
                                } else {
                                    self.builder
                                        .build_unsigned_int_to_float(
                                            val.into_int_value(),
                                            llvm_target_ty.into_float_type(),
                                            "cast_uitofp"
                                        )
                                        .unwrap()
                                        .into()
                                }
                            } else if val.is_float_value() && llvm_target_ty.is_float_type() {
                                self.builder
                                    .build_float_cast(
                                        val.into_float_value(),
                                        llvm_target_ty.into_float_type(),
                                        "cast_fcast"
                                    )
                                    .unwrap()
                                    .into()
                            } else if val.is_float_value() && llvm_target_ty.is_int_type() {
                                if Self::is_signed(target_ty) {
                                    self.builder
                                        .build_float_to_signed_int(
                                            val.into_float_value(),
                                            llvm_target_ty.into_int_type(),
                                            "cast_fptosi"
                                        )
                                        .unwrap()
                                        .into()
                                } else {
                                    self.builder
                                        .build_float_to_unsigned_int(
                                            val.into_float_value(),
                                            llvm_target_ty.into_int_type(),
                                            "cast_fptoui"
                                        )
                                        .unwrap()
                                        .into()
                                }
                            } else if val.is_pointer_value() && llvm_target_ty.is_pointer_type() {
                                let target_ptr_ty = llvm_target_ty.into_pointer_type();
                                self.builder
                                    .build_pointer_cast(
                                        val.into_pointer_value(),
                                        target_ptr_ty,
                                        "ptrcast"
                                    )
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
                                return Err(
                                    format!("Unsupported LLVM cast: {source_ty} to {target_ty}")
                                );
                            };
                            self.registers.insert(*dest, self.register_value(casted, target_ty));
                        }

                        Instruction::Add { dest, left, right, .. } => {
                            let l = *self.registers
                                .get(left)
                                .ok_or_else(|| {
                                    "LLVM ERROR: Left value missing in Add instruction!".to_string()
                                })?;
                            let r = *self.registers
                                .get(right)
                                .ok_or_else(|| {
                                    "LLVM ERROR: Right value missing in Add instruction!".to_string()
                                })?;

                            if l.is_float_value() || r.is_float_value() {
                                let _target_ty = if l.is_float_value() {
                                    l.into_float_value().get_type()
                                } else {
                                    r.into_float_value().get_type()
                                };
                                let (l_float, r_float) = promote_float_pair(&self.builder, l, r);
                                let res = self.builder
                                    .build_float_add(l_float, r_float, "fadd")
                                    .unwrap();
                                self.registers.insert(*dest, res.into());
                            } else {
                                let res = self.builder
                                    .build_int_add(
                                        as_int(&self.builder, l, &IrType::I64),
                                        as_int(&self.builder, r, &IrType::I64),
                                        "add"
                                    )
                                    .unwrap();
                                self.registers.insert(*dest, res.into());
                            }
                        }

                        Instruction::Sub { dest, left, right, .. } => {
                            let l = *self.registers
                                .get(left)
                                .ok_or_else(|| {
                                    "LLVM ERROR: Left value missing in Sub instruction!".to_string()
                                })?;
                            let r = *self.registers
                                .get(right)
                                .ok_or_else(|| {
                                    "LLVM ERROR: Right value missing in Sub instruction!".to_string()
                                })?;
                            if l.is_float_value() || r.is_float_value() {
                                let _target_ty = if l.is_float_value() {
                                    l.into_float_value().get_type()
                                } else {
                                    r.into_float_value().get_type()
                                };
                                let (l_float, r_float) = promote_float_pair(&self.builder, l, r);
                                let res = self.builder
                                    .build_float_sub(l_float, r_float, "fsub")
                                    .unwrap();
                                self.registers.insert(*dest, res.into());
                            } else {
                                let res = self.builder
                                    .build_int_sub(
                                        as_int(&self.builder, l, &IrType::I64),
                                        as_int(&self.builder, r, &IrType::I64),
                                        "sub"
                                    )
                                    .unwrap();
                                self.registers.insert(*dest, res.into());
                            }
                        }

                        Instruction::Mul { dest, left, right, .. } => {
                            let l = *self.registers
                                .get(left)
                                .ok_or_else(|| {
                                    "LLVM ERROR: Left value missing in Mul instruction!".to_string()
                                })?;
                            let r = *self.registers
                                .get(right)
                                .ok_or_else(|| {
                                    "LLVM ERROR: Right value missing in Mul instruction!".to_string()
                                })?;
                            if l.is_float_value() || r.is_float_value() {
                                let _target_ty = if l.is_float_value() {
                                    l.into_float_value().get_type()
                                } else {
                                    r.into_float_value().get_type()
                                };
                                let (l_float, r_float) = promote_float_pair(&self.builder, l, r);
                                let res = self.builder
                                    .build_float_mul(l_float, r_float, "fmul")
                                    .unwrap();
                                self.registers.insert(*dest, res.into());
                            } else {
                                let res = self.builder
                                    .build_int_mul(
                                        as_int(&self.builder, l, &IrType::I64),
                                        as_int(&self.builder, r, &IrType::I64),
                                        "mul"
                                    )
                                    .unwrap();
                                self.registers.insert(*dest, res.into());
                            }
                        }

                        | Instruction::BitAnd { dest, left, right }
                        | Instruction::BitOr { dest, left, right } => {
                            let l = as_int(&self.builder, self.registers[left], &IrType::I64);
                            let r = as_int(&self.builder, self.registers[right], &IrType::I64);
                            let v = (
                                if matches!(inst, Instruction::BitAnd { .. }) {
                                    self.builder.build_and(l, r, "and")
                                } else {
                                    self.builder.build_or(l, r, "or")
                                }
                            ).unwrap();
                            self.registers.insert(*dest, v.into());
                        }

                        Instruction::Div { dest, left, right, ty } => {
                            let l = *self.registers
                                .get(left)
                                .ok_or_else(|| {
                                    "LLVM ERROR: Left value missing in Div instruction!".to_string()
                                })?;
                            let r = *self.registers
                                .get(right)
                                .ok_or_else(|| {
                                    "LLVM ERROR: Right value missing in Div instruction!".to_string()
                                })?;
                            if l.is_float_value() || r.is_float_value() {
                                let _target_ty = if l.is_float_value() {
                                    l.into_float_value().get_type()
                                } else {
                                    r.into_float_value().get_type()
                                };
                                let (l_float, r_float) = promote_float_pair(&self.builder, l, r);
                                let res = self.builder
                                    .build_float_div(l_float, r_float, "fdiv")
                                    .unwrap();
                                self.registers.insert(*dest, res.into());
                            } else {
                                let res = if Self::is_signed(ty) {
                                    self.builder
                                        .build_int_signed_div(
                                            as_int(&self.builder, l, ty),
                                            as_int(&self.builder, r, ty),
                                            "sdiv"
                                        )
                                        .unwrap()
                                } else {
                                    self.builder
                                        .build_int_unsigned_div(
                                            as_int(&self.builder, l, ty),
                                            as_int(&self.builder, r, ty),
                                            "udiv"
                                        )
                                        .unwrap()
                                };
                                self.registers.insert(*dest, res.into());
                            }
                        }

                        Instruction::Mod { dest, left, right, ty } => {
                            let l = *self.registers
                                .get(left)
                                .ok_or_else(|| {
                                    "LLVM ERROR: Left value missing in Mod instruction!".to_string()
                                })?;
                            let r = *self.registers
                                .get(right)
                                .ok_or_else(|| {
                                    "LLVM ERROR: Right value missing in Mod instruction!".to_string()
                                })?;
                            if l.is_float_value() || r.is_float_value() {
                                let _target_ty = if l.is_float_value() {
                                    l.into_float_value().get_type()
                                } else {
                                    r.into_float_value().get_type()
                                };
                                let (l_float, r_float) = promote_float_pair(&self.builder, l, r);
                                let res = self.builder
                                    .build_float_rem(l_float, r_float, "frem")
                                    .unwrap();
                                self.registers.insert(*dest, res.into());
                            } else {
                                let res = if Self::is_signed(ty) {
                                    self.builder
                                        .build_int_signed_rem(
                                            as_int(&self.builder, l, ty),
                                            as_int(&self.builder, r, ty),
                                            "srem"
                                        )
                                        .unwrap()
                                } else {
                                    self.builder
                                        .build_int_unsigned_rem(
                                            as_int(&self.builder, l, ty),
                                            as_int(&self.builder, r, ty),
                                            "urem"
                                        )
                                        .unwrap()
                                };
                                self.registers.insert(*dest, res.into());
                            }
                        }

                        Instruction::CmpEq { dest, left, right, .. } => {
                            let l = *self.registers
                                .get(left)
                                .ok_or_else(|| {
                                    "LLVM ERROR: Left value missing in CmpEq instruction!".to_string()
                                })?;

                            let r = *self.registers
                                .get(right)
                                .ok_or_else(|| {
                                    "LLVM ERROR: Right value missing in CmpEq instruction!".to_string()
                                })?;

                            let cmp = if l.is_float_value() || r.is_float_value() {
                                let target_ty = if l.is_float_value() {
                                    l.into_float_value().get_type()
                                } else {
                                    r.into_float_value().get_type()
                                };

                                let l_float = as_float(&self.builder, l, target_ty);
                                let r_float = as_float(&self.builder, r, target_ty);

                                self.builder
                                    .build_float_compare(
                                        inkwell::FloatPredicate::OEQ,
                                        l_float,
                                        r_float,
                                        "fcmpeq"
                                    )
                                    .unwrap()
                            } else {
                                self.builder
                                    .build_int_compare(
                                        inkwell::IntPredicate::EQ,
                                        as_int(&self.builder, l, &IrType::I64),
                                        as_int(&self.builder, r, &IrType::I64),
                                        "cmpeq"
                                    )
                                    .unwrap()
                            };
                            let zext = self.builder
                                .build_int_z_extend(cmp, self.context.i64_type(), "zext")
                                .unwrap();

                            self.registers.insert(*dest, zext.into());
                        }

                        Instruction::CmpLt { dest, left, right, ty } => {
                            let l = *self.registers
                                .get(left)
                                .ok_or_else(|| {
                                    "LLVM ERROR: Left value missing in CmpLt instruction!".to_string()
                                })?;

                            let r = *self.registers
                                .get(right)
                                .ok_or_else(|| {
                                    "LLVM ERROR: Right value missing in CmpLt instruction!".to_string()
                                })?;

                            let cmp = if l.is_float_value() || r.is_float_value() {
                                let target_ty = if l.is_float_value() {
                                    l.into_float_value().get_type()
                                } else {
                                    r.into_float_value().get_type()
                                };

                                let l_float = as_float(&self.builder, l, target_ty);
                                let r_float = as_float(&self.builder, r, target_ty);

                                self.builder
                                    .build_float_compare(
                                        inkwell::FloatPredicate::OLT,
                                        l_float,
                                        r_float,
                                        "fcmplt"
                                    )
                                    .unwrap()
                            } else {
                                let predicate = if Self::is_signed(ty) {
                                    inkwell::IntPredicate::SLT
                                } else {
                                    inkwell::IntPredicate::ULT
                                };
                                self.builder
                                    .build_int_compare(
                                        predicate,
                                        as_int(&self.builder, l, ty),
                                        as_int(&self.builder, r, ty),
                                        "cmplt"
                                    )
                                    .unwrap()
                            };
                            let zext = self.builder
                                .build_int_z_extend(cmp, self.context.i64_type(), "zext")
                                .unwrap();

                            self.registers.insert(*dest, zext.into());
                        }

                        Instruction::CmpGt { dest, left, right, ty } => {
                            let l = *self.registers
                                .get(left)
                                .ok_or_else(|| {
                                    "LLVM ERROR: Left value missing in CmpGt instruction!".to_string()
                                })?;
                            let r = *self.registers
                                .get(right)
                                .ok_or_else(|| {
                                    "LLVM ERROR: Right value missing in CmpGt instruction!".to_string()
                                })?;
                            let cmp = if l.is_float_value() || r.is_float_value() {
                                let target_ty = if l.is_float_value() {
                                    l.into_float_value().get_type()
                                } else {
                                    r.into_float_value().get_type()
                                };

                                let l_float = as_float(&self.builder, l, target_ty);
                                let r_float = as_float(&self.builder, r, target_ty);

                                self.builder
                                    .build_float_compare(
                                        inkwell::FloatPredicate::OGT,
                                        l_float,
                                        r_float,
                                        "fcmpgt"
                                    )
                                    .unwrap()
                            } else {
                                let predicate = if Self::is_signed(ty) {
                                    inkwell::IntPredicate::SGT
                                } else {
                                    inkwell::IntPredicate::UGT
                                };
                                self.builder
                                    .build_int_compare(
                                        predicate,
                                        as_int(&self.builder, l, ty),
                                        as_int(&self.builder, r, ty),
                                        "cmpgt"
                                    )
                                    .unwrap()
                            };

                            let zext = self.builder
                                .build_int_z_extend(cmp, self.context.i64_type(), "zext")
                                .unwrap();

                            self.registers.insert(*dest, zext.into());
                        }

                        Instruction::CmpNeq { dest, left, right, .. } => {
                            let l = *self.registers
                                .get(left)
                                .ok_or_else(|| {
                                    "LLVM ERROR: Left value missing in CmpNeq instruction!".to_string()
                                })?;
                            let r = *self.registers
                                .get(right)
                                .ok_or_else(|| {
                                    "LLVM ERROR: Right value missing in CmpNeq instruction!".to_string()
                                })?;
                            let cmp = if l.is_float_value() || r.is_float_value() {
                                let target_ty = if l.is_float_value() {
                                    l.into_float_value().get_type()
                                } else {
                                    r.into_float_value().get_type()
                                };

                                let l_float = as_float(&self.builder, l, target_ty);
                                let r_float = as_float(&self.builder, r, target_ty);

                                self.builder
                                    .build_float_compare(
                                        inkwell::FloatPredicate::UNE,
                                        l_float,
                                        r_float,
                                        "fcmpne"
                                    )
                                    .unwrap()
                            } else {
                                self.builder
                                    .build_int_compare(
                                        inkwell::IntPredicate::NE,
                                        as_int(&self.builder, l, &IrType::I64),
                                        as_int(&self.builder, r, &IrType::I64),
                                        "cmpne"
                                    )
                                    .unwrap()
                            };
                            let zext = self.builder
                                .build_int_z_extend(cmp, self.context.i64_type(), "zext")
                                .unwrap();

                            self.registers.insert(*dest, zext.into());
                        }

                        Instruction::CmpLe { dest, left, right, ty } => {
                            let l = *self.registers
                                .get(left)
                                .ok_or_else(|| {
                                    "LLVM ERROR: Left value missing in CmpLe instruction!".to_string()
                                })?;

                            let r = *self.registers
                                .get(right)
                                .ok_or_else(|| {
                                    "LLVM ERROR: Right value missing in CmpLe instruction!".to_string()
                                })?;

                            let cmp = if l.is_float_value() || r.is_float_value() {
                                let target_ty = if l.is_float_value() {
                                    l.into_float_value().get_type()
                                } else {
                                    r.into_float_value().get_type()
                                };

                                let l_float = as_float(&self.builder, l, target_ty);
                                let r_float = as_float(&self.builder, r, target_ty);

                                self.builder
                                    .build_float_compare(
                                        inkwell::FloatPredicate::OLE,
                                        l_float,
                                        r_float,
                                        "fcmple"
                                    )
                                    .unwrap()
                            } else {
                                let predicate = if Self::is_signed(ty) {
                                    inkwell::IntPredicate::SLE
                                } else {
                                    inkwell::IntPredicate::ULE
                                };
                                self.builder
                                    .build_int_compare(
                                        predicate,
                                        as_int(&self.builder, l, ty),
                                        as_int(&self.builder, r, ty),
                                        "cmple"
                                    )
                                    .unwrap()
                            };
                            let zext = self.builder
                                .build_int_z_extend(cmp, self.context.i64_type(), "zext")
                                .unwrap();

                            self.registers.insert(*dest, zext.into());
                        }

                        Instruction::CmpGe { dest, left, right, ty } => {
                            let l = *self.registers
                                .get(left)
                                .ok_or_else(|| {
                                    "LLVM ERROR: Left value missing in CmpGe instruction!".to_string()
                                })?;

                            let r = *self.registers
                                .get(right)
                                .ok_or_else(|| {
                                    "LLVM ERROR: Right value missing in CmpGe instruction!".to_string()
                                })?;

                            let cmp = if l.is_float_value() || r.is_float_value() {
                                let target_ty = if l.is_float_value() {
                                    l.into_float_value().get_type()
                                } else {
                                    r.into_float_value().get_type()
                                };

                                let l_float = as_float(&self.builder, l, target_ty);
                                let r_float = as_float(&self.builder, r, target_ty);

                                self.builder
                                    .build_float_compare(
                                        inkwell::FloatPredicate::OGE,
                                        l_float,
                                        r_float,
                                        "fcmpge"
                                    )
                                    .unwrap()
                            } else {
                                let predicate = if Self::is_signed(ty) {
                                    inkwell::IntPredicate::SGE
                                } else {
                                    inkwell::IntPredicate::UGE
                                };
                                self.builder
                                    .build_int_compare(
                                        predicate,
                                        as_int(&self.builder, l, ty),
                                        as_int(&self.builder, r, ty),
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
                            let cond_val = *self.registers
                                .get(cond)
                                .ok_or_else(||
                                    format!("LLVM ERROR: Condition register '{}' missing in CondBr instruction!", cond)
                                )?;

                            let cond_int = if cond_val.is_pointer_value() {
                                as_int(&self.builder, cond_val, &IrType::Bool)
                            } else {
                                cond_val.into_int_value()
                            };

                            let zero = cond_int.get_type().const_zero();
                            let is_true = self.builder
                                .build_int_compare(
                                    inkwell::IntPredicate::NE,
                                    cond_int,
                                    zero,
                                    "cond_true"
                                )
                                .unwrap();

                            let bb_true = llvm_blocks.get(&if_true.0).unwrap();
                            let bb_false = llvm_blocks.get(&if_false.0).unwrap();

                            self.builder
                                .build_conditional_branch(is_true, *bb_true, *bb_false)
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
                            let expected_ret_ir_ty = &func.ret_type;

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
                                    } else if
                                        val.is_pointer_value() &&
                                        expected_ty.is_pointer_type()
                                    {
                                        let val_ptr = val.into_pointer_value();
                                        let expected_ptr_ty = expected_ty.into_pointer_type();
                                        if val_ptr.get_type() != expected_ptr_ty {
                                            val = self.builder
                                                .build_pointer_cast(
                                                    val_ptr,
                                                    expected_ptr_ty,
                                                    "ret_cast_ptr"
                                                )
                                                .unwrap()
                                                .into();
                                        }
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
                                            val = if Self::is_signed(expected_ret_ir_ty) {
                                                self.builder
                                                    .build_int_s_extend(
                                                        val_int,
                                                        expected_int,
                                                        "ret_sext"
                                                    )
                                                    .unwrap()
                                                    .into()
                                            } else {
                                                self.builder
                                                    .build_int_z_extend(
                                                        val_int,
                                                        expected_int,
                                                        "ret_zext"
                                                    )
                                                    .unwrap()
                                                    .into()
                                            };
                                        }
                                    } else if val.is_int_value() && expected_ty.is_float_type() {
                                        val = if Self::is_signed(expected_ret_ir_ty) {
                                            self.builder
                                                .build_signed_int_to_float(
                                                    val.into_int_value(),
                                                    expected_ty.into_float_type(),
                                                    "ret_sitofp"
                                                )
                                                .unwrap()
                                                .into()
                                        } else {
                                            self.builder
                                                .build_unsigned_int_to_float(
                                                    val.into_int_value(),
                                                    expected_ty.into_float_type(),
                                                    "ret_uitofp"
                                                )
                                                .unwrap()
                                                .into()
                                        };
                                    } else if val.is_float_value() && expected_ty.is_float_type() {
                                        let val_float = val.into_float_value();
                                        let expected_float = expected_ty.into_float_type();
                                        if val_float.get_type() != expected_float {
                                            val = self.builder
                                                .build_float_cast(
                                                    val_float,
                                                    expected_float,
                                                    "ret_fcast"
                                                )
                                                .unwrap()
                                                .into();
                                        }
                                    } else if val.is_float_value() && expected_ty.is_int_type() {
                                        val = if Self::is_signed(expected_ret_ir_ty) {
                                            self.builder
                                                .build_float_to_signed_int(
                                                    val.into_float_value(),
                                                    expected_ty.into_int_type(),
                                                    "ret_fptosi"
                                                )
                                                .unwrap()
                                                .into()
                                        } else {
                                            self.builder
                                                .build_float_to_unsigned_int(
                                                    val.into_float_value(),
                                                    expected_ty.into_int_type(),
                                                    "ret_fptoui"
                                                )
                                                .unwrap()
                                                .into()
                                        };
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

                            let ir_func = ir_module.functions
                                .iter()
                                .find(|f| f.name == *func_name)
                                .unwrap();

                            let mut llvm_args: Vec<BasicMetadataValueEnum> = Vec::new();
                            let is_vararg = fn_type.is_var_arg();
                            for (i, arg) in args.iter().enumerate() {
                                let mut val = *self.registers
                                    .get(arg)
                                    .expect("LLVM ERROR: Call instruction argument missing");

                                let arg_ir_ty = if i < ir_func.args.len() {
                                    &ir_func.args[i].1
                                } else {
                                    &IrType::I64
                                };

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
                                            val = if Self::is_signed(arg_ir_ty) {
                                                self.builder
                                                    .build_int_s_extend(
                                                        val_int,
                                                        expected_int,
                                                        "arg_sext"
                                                    )
                                                    .unwrap()
                                                    .into()
                                            } else {
                                                self.builder
                                                    .build_int_z_extend(
                                                        val_int,
                                                        expected_int,
                                                        "arg_zext"
                                                    )
                                                    .unwrap()
                                                    .into()
                                            };
                                        }
                                    } else if val.is_int_value() && expected_ty.is_float_type() {
                                        val = if Self::is_signed(arg_ir_ty) {
                                            self.builder
                                                .build_signed_int_to_float(
                                                    val.into_int_value(),
                                                    expected_ty.into_float_type(),
                                                    "arg_sitofp"
                                                )
                                                .unwrap()
                                                .into()
                                        } else {
                                            self.builder
                                                .build_unsigned_int_to_float(
                                                    val.into_int_value(),
                                                    expected_ty.into_float_type(),
                                                    "arg_uitofp"
                                                )
                                                .unwrap()
                                                .into()
                                        };
                                    } else if val.is_float_value() && expected_ty.is_float_type() {
                                        let val_float = val.into_float_value();
                                        let expected_float = expected_ty.into_float_type();
                                        if val_float.get_type() != expected_float {
                                            val = self.builder
                                                .build_float_cast(
                                                    val_float,
                                                    expected_float,
                                                    "arg_fcast"
                                                )
                                                .unwrap()
                                                .into();
                                        }
                                    } else if val.is_float_value() && expected_ty.is_int_type() {
                                        val = if Self::is_signed(arg_ir_ty) {
                                            self.builder
                                                .build_float_to_signed_int(
                                                    val.into_float_value(),
                                                    expected_ty.into_int_type(),
                                                    "arg_fptosi"
                                                )
                                                .unwrap()
                                                .into()
                                        } else {
                                            self.builder
                                                .build_float_to_unsigned_int(
                                                    val.into_float_value(),
                                                    expected_ty.into_int_type(),
                                                    "arg_fptoui"
                                                )
                                                .unwrap()
                                                .into()
                                        };
                                    }
                                } else if is_vararg {
                                    if
                                        val.is_float_value() &&
                                        val.into_float_value().get_type() != self.context.f64_type()
                                    {
                                        val = self.builder
                                            .build_float_ext(
                                                val.into_float_value(),
                                                self.context.f64_type(),
                                                "vararg_fpext"
                                            )
                                            .unwrap()
                                            .into();
                                    }
                                }

                                llvm_args.push(val.into());
                            }

                            let call_site = self.builder
                                .build_call(function, &llvm_args, "call")
                                .unwrap();

                            if let Some(ret_val) = call_site.try_as_basic_value().left() {
                                self.registers.insert(
                                    *dest,
                                    self.register_value(ret_val, &ir_func.ret_type)
                                );
                            }
                        }

                        Instruction::MakeFatPtr { dest, data_ptr, vtable_name, adopt } => {
                            let fat_ptr_ty = self.context.i64_type().array_type(2);
                            let word = self.context.i64_type();
                            let fields = vec![
                                IrType::Array(0, Box::new(IrType::I64)),
                                IrType::Ptr(Box::new(IrType::U8))
                            ];
                            let destructor = self.destructor("trait_object", &fields, None);
                            let raw_ptr = self.managed_alloc(
                                word.const_int(16, false),
                                word.const_zero(),
                                destructor
                            );
                            let fat_ptr_alloc = self.builder
                                .build_pointer_cast(
                                    raw_ptr,
                                    fat_ptr_ty.ptr_type(inkwell::AddressSpace::default()),
                                    "fat_ptr_cast"
                                )
                                .unwrap();

                            let data_raw = *self.registers
                                .get(data_ptr)
                                .ok_or_else(|| {
                                    "LLVM ERROR: data_ptr register missing!".to_string()
                                })?;
                            let data_pointer = as_ptr(&self.builder, data_raw);
                            if !adopt {
                                self.builder
                                    .build_call(
                                        self.module.get_function("__fishy_retain").unwrap(),
                                        &[data_pointer.into()],
                                        ""
                                    )
                                    .unwrap();
                            }
                            let data_int = if data_raw.is_pointer_value() {
                                self.builder
                                    .build_ptr_to_int(
                                        data_raw.into_pointer_value(),
                                        self.context.i64_type(),
                                        "cast"
                                    )
                                    .unwrap()
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
                            receiver_mode,
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
                            let mut fat_ptr_alloc = as_ptr(&self.builder, fat_ptr_raw);
                            let borrowed = matches!(
                                receiver_mode,
                                crate::ir::ReceiverMode::Shared | crate::ir::ReceiverMode::Exclusive
                            );
                            if borrowed {
                                fat_ptr_alloc = self.builder
                                    .build_load(
                                        self.context
                                            .i64_type()
                                            .ptr_type(inkwell::AddressSpace::default()),
                                        fat_ptr_alloc,
                                        "trait.handle"
                                    )
                                    .unwrap()
                                    .into_pointer_value();
                            }

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
                                vec![
                                    self.context
                                        .i64_type()
                                        .ptr_type(inkwell::AddressSpace::default())
                                        .into()
                                ];
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

                            let receiver = if borrowed { data_field } else { data_ptr };
                            if *receiver_mode == crate::ir::ReceiverMode::Consume {
                                self.builder.build_store(data_field, i64_ty.const_zero()).unwrap();
                            }
                            let mut llvm_args: Vec<BasicMetadataValueEnum> = vec![receiver.into()];
                            for (i, arg) in args.iter().enumerate() {
                                let mut val = *self.registers
                                    .get(arg)
                                    .expect("LLVM ERROR: dyn_call args missing!");

                                if i + 1 < llvm_param_types.len() {
                                    let expected_ty = llvm_param_types[i + 1];
                                    let arg_ir_ty = &arg_types[i];

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
                                            val = if Self::is_signed(arg_ir_ty) {
                                                self.builder
                                                    .build_int_s_extend(
                                                        val_int,
                                                        expected_int,
                                                        "dyn_sext"
                                                    )
                                                    .unwrap()
                                                    .into()
                                            } else {
                                                self.builder
                                                    .build_int_z_extend(
                                                        val_int,
                                                        expected_int,
                                                        "dyn_zext"
                                                    )
                                                    .unwrap()
                                                    .into()
                                            };
                                        }
                                    } else if val.is_int_value() && expected_ty.is_float_type() {
                                        val = if Self::is_signed(arg_ir_ty) {
                                            self.builder
                                                .build_signed_int_to_float(
                                                    val.into_int_value(),
                                                    expected_ty.into_float_type(),
                                                    "dyn_sitofp"
                                                )
                                                .unwrap()
                                                .into()
                                        } else {
                                            self.builder
                                                .build_unsigned_int_to_float(
                                                    val.into_int_value(),
                                                    expected_ty.into_float_type(),
                                                    "dyn_uitofp"
                                                )
                                                .unwrap()
                                                .into()
                                        };
                                    } else if val.is_float_value() && expected_ty.is_float_type() {
                                        let val_float = val.into_float_value();
                                        let expected_float = expected_ty.into_float_type();
                                        if val_float.get_type() != expected_float {
                                            val = self.builder
                                                .build_float_cast(
                                                    val_float,
                                                    expected_float,
                                                    "dyn_fcast"
                                                )
                                                .unwrap()
                                                .into();
                                        }
                                    } else if val.is_float_value() && expected_ty.is_int_type() {
                                        val = if Self::is_signed(arg_ir_ty) {
                                            self.builder
                                                .build_float_to_signed_int(
                                                    val.into_float_value(),
                                                    expected_ty.into_int_type(),
                                                    "dyn_fptosi"
                                                )
                                                .unwrap()
                                                .into()
                                        } else {
                                            self.builder
                                                .build_float_to_unsigned_int(
                                                    val.into_float_value(),
                                                    expected_ty.into_int_type(),
                                                    "dyn_fptoui"
                                                )
                                                .unwrap()
                                                .into()
                                        };
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
                                    self.registers.insert(
                                        *dest,
                                        self.register_value(ret_val, ret_type)
                                    );
                                }
                            }
                        }

                        Instruction::LoadFnPtr { dest, fn_name } => {
                            let func = llvm_funcs
                                .get(fn_name)
                                .ok_or_else(||
                                    format!("LLVM ERROR: Function '{}' not found!", fn_name)
                                )?;
                            let ptr = func.as_global_value().as_pointer_value();
                            let val = self.builder
                                .build_ptr_to_int(ptr, self.context.i64_type(), "fn_ptr_int")
                                .unwrap();

                            self.registers.insert(*dest, val.into());
                        }

                        Instruction::IndirectCall { dest, fn_ptr, args, arg_types, ret_type } => {
                            let ptr_raw = *self.registers.get(fn_ptr).unwrap();
                            let mut param_types: Vec<inkwell::types::BasicMetadataTypeEnum> =
                                vec![];
                            for arg_ty in arg_types {
                                param_types.push(self.get_llvm_type(arg_ty).into());
                            }

                            let fn_ty = if *ret_type == IrType::Void {
                                self.context.void_type().fn_type(&param_types, false)
                            } else {
                                self.get_llvm_type(ret_type).fn_type(&param_types, false)
                            };
                            let callable = Self::value_as_ptr(
                                &self.builder,
                                ptr_raw,
                                fn_ty.ptr_type(inkwell::AddressSpace::default())
                            );

                            let mut llvm_args = vec![];
                            for (i, arg) in args.iter().enumerate() {
                                let mut val = *self.registers.get(arg).unwrap();
                                let expected_ty = param_types[i];
                                let arg_ir_ty = &arg_types[i];

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
                                                "icall_trunc"
                                            )
                                            .unwrap()
                                            .into();
                                    } else if
                                        val_int.get_type().get_bit_width() <
                                        expected_int.get_bit_width()
                                    {
                                        val = if Self::is_signed(arg_ir_ty) {
                                            self.builder
                                                .build_int_s_extend(
                                                    val_int,
                                                    expected_int,
                                                    "icall_sext"
                                                )
                                                .unwrap()
                                                .into()
                                        } else {
                                            self.builder
                                                .build_int_z_extend(
                                                    val_int,
                                                    expected_int,
                                                    "icall_zext"
                                                )
                                                .unwrap()
                                                .into()
                                        };
                                    }
                                } else if val.is_int_value() && expected_ty.is_float_type() {
                                    val = if Self::is_signed(arg_ir_ty) {
                                        self.builder
                                            .build_signed_int_to_float(
                                                val.into_int_value(),
                                                expected_ty.into_float_type(),
                                                "icall_sitofp"
                                            )
                                            .unwrap()
                                            .into()
                                    } else {
                                        self.builder
                                            .build_unsigned_int_to_float(
                                                val.into_int_value(),
                                                expected_ty.into_float_type(),
                                                "icall_uitofp"
                                            )
                                            .unwrap()
                                            .into()
                                    };
                                } else if val.is_float_value() && expected_ty.is_float_type() {
                                    let val_float = val.into_float_value();
                                    let expected_float = expected_ty.into_float_type();
                                    if val_float.get_type() != expected_float {
                                        val = self.builder
                                            .build_float_cast(
                                                val_float,
                                                expected_float,
                                                "icall_fcast"
                                            )
                                            .unwrap()
                                            .into();
                                    }
                                } else if val.is_float_value() && expected_ty.is_int_type() {
                                    val = if Self::is_signed(arg_ir_ty) {
                                        self.builder
                                            .build_float_to_signed_int(
                                                val.into_float_value(),
                                                expected_ty.into_int_type(),
                                                "icall_fptosi"
                                            )
                                            .unwrap()
                                            .into()
                                    } else {
                                        self.builder
                                            .build_float_to_unsigned_int(
                                                val.into_float_value(),
                                                expected_ty.into_int_type(),
                                                "icall_fptoui"
                                            )
                                            .unwrap()
                                            .into()
                                    };
                                }

                                llvm_args.push(val.into());
                            }

                            let call_site = self.builder
                                .build_indirect_call(fn_ty, callable, &llvm_args, "icall")
                                .unwrap();

                            if *ret_type != IrType::Void {
                                if let Some(ret_val) = call_site.try_as_basic_value().left() {
                                    self.registers.insert(
                                        *dest,
                                        self.register_value(ret_val, ret_type)
                                    );
                                }
                            }
                        }

                        Instruction::MakeClosure { dest, fn_name, env_ptr } => {
                            let word = self.context.i64_type();
                            let fields = vec![
                                IrType::Ptr(Box::new(IrType::U8)),
                                IrType::Array(0, Box::new(IrType::I64))
                            ];
                            let destructor = self.destructor("closure", &fields, None);
                            let data = self.managed_alloc(
                                word.const_int(16, false),
                                word.const_zero(),
                                destructor
                            );
                            let function = llvm_funcs
                                .get(fn_name)
                                .ok_or_else(|| format!("Missing closure function {fn_name}"))?;
                            self.builder
                                .build_store(data, function.as_global_value().as_pointer_value())
                                .unwrap();
                            let env = as_ptr(&self.builder, self.registers[env_ptr]);
                            let slot = unsafe {
                                self.builder
                                    .build_gep(
                                        word,
                                        data,
                                        &[word.const_int(1, false)],
                                        "closure.env"
                                    )
                                    .unwrap()
                            };
                            self.builder.build_store(slot, env).unwrap();
                            self.registers.insert(*dest, data.into());
                        }

                        Instruction::CallClosure {
                            dest,
                            closure_ptr,
                            args,
                            arg_types,
                            ret_type,
                        } => {
                            let closure_raw = *self.registers.get(closure_ptr).unwrap();
                            let closure_ptr_val = as_ptr(&self.builder, closure_raw);

                            let func_int = self.builder
                                .build_load(self.context.i64_type(), closure_ptr_val, "func_int")
                                .unwrap()
                                .into_int_value();

                            let minus_one = self.context.i64_type().const_int(1, false);

                            let env_field = unsafe {
                                self.builder
                                    .build_gep(
                                        self.context.i64_type(),
                                        closure_ptr_val,
                                        &[minus_one],
                                        "env_field"
                                    )
                                    .unwrap()
                            };

                            let env_int = self.builder
                                .build_load(self.context.i64_type(), env_field, "env_int")
                                .unwrap()
                                .into_int_value();

                            let mut llvm_param_types: Vec<inkwell::types::BasicMetadataTypeEnum> =
                                vec![
                                    self.context
                                        .i64_type()
                                        .ptr_type(inkwell::AddressSpace::default())
                                        .into()
                                ];

                            for arg_ty in arg_types {
                                llvm_param_types.push(self.get_llvm_type(arg_ty).into());
                            }

                            let fn_ty = if *ret_type == IrType::Void {
                                self.context.void_type().fn_type(&llvm_param_types, false)
                            } else {
                                self.get_llvm_type(ret_type).fn_type(&llvm_param_types, false)
                            };

                            let func_callable = self.builder
                                .build_int_to_ptr(
                                    func_int,
                                    fn_ty.ptr_type(inkwell::AddressSpace::default()),
                                    "callable"
                                )
                                .unwrap();

                            let mut llvm_args: Vec<BasicMetadataValueEnum> = vec![];
                            let env_ty = llvm_param_types[0];
                            let env_val: BasicValueEnum = if env_ty.is_pointer_type() {
                                self.builder
                                    .build_int_to_ptr(
                                        env_int,
                                        env_ty.into_pointer_type(),
                                        "env_ptr_cast"
                                    )
                                    .unwrap()
                                    .into()
                            } else {
                                env_int.into()
                            };
                            llvm_args.push(env_val.into());

                            for (i, arg) in args.iter().enumerate() {
                                let mut val = *self.registers.get(arg).unwrap();
                                let expected_ty = llvm_param_types[i + 1];
                                let arg_ir_ty = &arg_types[i];

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
                                            .build_int_truncate(val_int, expected_int, "arg_trunc")
                                            .unwrap()
                                            .into();
                                    } else if
                                        val_int.get_type().get_bit_width() <
                                        expected_int.get_bit_width()
                                    {
                                        val = if Self::is_signed(arg_ir_ty) {
                                            self.builder
                                                .build_int_s_extend(
                                                    val_int,
                                                    expected_int,
                                                    "arg_sext"
                                                )
                                                .unwrap()
                                                .into()
                                        } else {
                                            self.builder
                                                .build_int_z_extend(
                                                    val_int,
                                                    expected_int,
                                                    "arg_zext"
                                                )
                                                .unwrap()
                                                .into()
                                        };
                                    }
                                } else if val.is_int_value() && expected_ty.is_float_type() {
                                    val = if Self::is_signed(arg_ir_ty) {
                                        self.builder
                                            .build_signed_int_to_float(
                                                val.into_int_value(),
                                                expected_ty.into_float_type(),
                                                "arg_sitofp"
                                            )
                                            .unwrap()
                                            .into()
                                    } else {
                                        self.builder
                                            .build_unsigned_int_to_float(
                                                val.into_int_value(),
                                                expected_ty.into_float_type(),
                                                "arg_uitofp"
                                            )
                                            .unwrap()
                                            .into()
                                    };
                                } else if val.is_float_value() && expected_ty.is_float_type() {
                                    let val_float = val.into_float_value();
                                    let expected_float = expected_ty.into_float_type();
                                    if val_float.get_type() != expected_float {
                                        val = self.builder
                                            .build_float_cast(
                                                val_float,
                                                expected_float,
                                                "arg_fcast"
                                            )
                                            .unwrap()
                                            .into();
                                    }
                                } else if val.is_float_value() && expected_ty.is_int_type() {
                                    val = if Self::is_signed(arg_ir_ty) {
                                        self.builder
                                            .build_float_to_signed_int(
                                                val.into_float_value(),
                                                expected_ty.into_int_type(),
                                                "arg_fptosi"
                                            )
                                            .unwrap()
                                            .into()
                                    } else {
                                        self.builder
                                            .build_float_to_unsigned_int(
                                                val.into_float_value(),
                                                expected_ty.into_int_type(),
                                                "arg_fptoui"
                                            )
                                            .unwrap()
                                            .into()
                                    };
                                }

                                llvm_args.push(val.into());
                            }

                            let call_site = self.builder
                                .build_indirect_call(
                                    fn_ty,
                                    func_callable,
                                    &llvm_args,
                                    "closure_call"
                                )
                                .unwrap();
                            if *ret_type != IrType::Void {
                                if let Some(ret_val) = call_site.try_as_basic_value().left() {
                                    self.registers.insert(
                                        *dest,
                                        self.register_value(ret_val, ret_type)
                                    );
                                }
                            }
                        }

                        Instruction::Retain { ptr } | Instruction::Release { ptr } => {
                            let value = as_ptr(&self.builder, self.registers[ptr]);
                            let name = if matches!(inst, Instruction::Retain { .. }) {
                                "__fishy_retain"
                            } else {
                                "__fishy_release"
                            };
                            self.builder
                                .build_call(
                                    self.module.get_function(name).unwrap(),
                                    &[value.into()],
                                    ""
                                )
                                .unwrap();
                        }

                        Instruction::Unreachable => {
                            self.builder.build_unreachable().unwrap();
                        }
                    }
                }
            }
        }

        self.module.verify().map_err(|e| format!("LLVM module verification failed: {}", e))?;
        Ok(())
    }

    pub fn build_executable(&self, filename: &str) -> Result<(), String> {
        self.module.verify().map_err(|e| format!("LLVM module verification failed: {e}"))?;
        self.target_machine
            .write_to_file(&self.module, FileType::Object, Path::new(filename))
            .map_err(|e| format!("Object emission failed: {e}"))
    }
}

#[cfg(test)]
#[path = "../tests/support/llvm_abi_tests.rs"]
mod abi_tests;

use crate::ast::{ Expr, Stmt, Type };
use crate::physical_ownership::{Operation as Physical, ReadMode, SlotKind, EdgeKind};
use crate::ir::{ BasicBlock, BlockId, FunctionIr, Instruction, IrType, ModuleIr, VReg };
use crate::token::{ Literal, Token, TokenType };
use crate::type_checker::{ CallType, StructInfo, DefaultCall };
use std::collections::{ HashMap, HashSet };

pub struct IrBuilder {
    capture_plans: crate::captures::CapturePlans,

    binding_slots: HashMap<usize, (VReg, IrType, bool)>,
    next_reg: usize,
    next_block: usize,
    current_block: Option<BlockId>,
    blocks: HashMap<BlockId, BasicBlock>,
    functions: Vec<FunctionIr>,
    scopes: Vec<HashMap<String, (VReg, IrType, bool)>>,
    pub property_indices: HashMap<Token, usize>,
    pub resolved_calls: HashMap<Token, CallType>,
    specialized_methods: HashMap<Token, (String, bool)>,
    default_calls: HashMap<Token, DefaultCall>,
    pub variant_tags: HashMap<String, usize>,
    pub traits: HashSet<String>,
    pub vtables: HashMap<String, Vec<String>>,
    pub trait_vtable_layout: HashMap<String, HashMap<String, usize>>,
    pub user_types: HashMap<String, StructInfo>,
    pub lambda_count: usize,
    globals: HashMap<String, (VReg, IrType, bool)>,
    synthetic_layouts: HashMap<String, Vec<IrType>>,
    function_signatures: HashMap<String, (Vec<IrType>, IrType)>,
    current_function: Option<String>,
    unit_functions: HashSet<String>,
    string_temporaries: HashMap<VReg, VReg>,
    cleanup_order: HashMap<VReg, usize>,
    physical_slots: HashSet<VReg>,
    stack_slots: HashSet<VReg>,
    temporary_values: HashMap<VReg,VReg>,
    ownership: Option<crate::ownership::Plan>,
}

impl IrBuilder {
    fn impl_type_name(target_type: &Type) -> Option<String> {
        Some(match target_type {
            Type::Custom(name) => name.clone(),
            Type::Bool => "bool".into(),
            Type::String => "string".into(),
            Type::U8 => "u8".into(),
            Type::U16 => "u16".into(),
            Type::U32 => "u32".into(),
            Type::U64 => "u64".into(),
            Type::I8 => "i8".into(),
            Type::I16 => "i16".into(),
            Type::I32 => "i32".into(),
            Type::I64 => "i64".into(),
            Type::F16 => "f16".into(),
            Type::F32 => "f32".into(),
            Type::F64 => "f64".into(),
            _ => {
                return None;
            }
        })
    }

    pub fn new(
        capture_plans: crate::captures::CapturePlans,

        property_indices: HashMap<Token, usize>,
        resolved_calls: HashMap<Token, CallType>,
        specialized_methods: HashMap<Token, (String, bool)>,
        default_calls: HashMap<Token, DefaultCall>,
        traits: HashSet<String>,
        trait_vtable_layout: HashMap<String, HashMap<String, usize>>,
        user_types: HashMap<String, StructInfo>
    ) -> Self {
        Self {
            capture_plans,
            binding_slots: HashMap::new(),
            next_reg: 0,
            next_block: 0,
            current_block: None,
            blocks: HashMap::new(),
            functions: Vec::new(),
            scopes: vec![HashMap::new()],
            property_indices,
            resolved_calls,
            specialized_methods,
            default_calls,
            variant_tags: HashMap::new(),
            traits,
            vtables: HashMap::new(),
            trait_vtable_layout,
            user_types,
            lambda_count: 0,
            globals: HashMap::new(),
            synthetic_layouts: HashMap::new(),
            function_signatures: HashMap::new(),
            current_function: None,
            unit_functions: HashSet::new(),
            string_temporaries: HashMap::new(),
            cleanup_order: HashMap::new(),
            physical_slots: HashSet::new(),
            stack_slots: HashSet::new(),
            temporary_values: HashMap::new(),
            ownership: None,
        }
    }

    pub fn with_ownership(mut self, plan: crate::ownership::Plan) -> Self {
        assert!(plan.errors.is_empty(), "ownership must be checked before physical lowering");
        self.ownership = Some(plan);
        self
    }

    fn transfers_expr(&self, expr: &Expr) -> bool {
        expr.identity().is_some_and(|id|
            self.ownership.as_ref().is_some_and(|p| p.consumed.contains(&id))
        )
    }

    pub fn build(mut self, stmts: &[Stmt]) -> ModuleIr {
        assert!(
            self.ownership.is_some(),
            "source IR construction requires a verified ownership plan"
        );
        for stmt in stmts {
            if let Stmt::Function { name, params, return_type, type_params, .. } = stmt {
                if type_params.is_empty() {
                    self.function_signatures.insert(name.lexeme.clone(), (
                        params
                            .iter()
                            .map(|p|
                                self.map_type(
                                    p.type_annotation.as_ref().expect("checked parameter")
                                )
                            )
                            .collect(),
                        self.map_type(return_type.as_ref().unwrap_or(&Type::Void)),
                    ));
                }
            }
            if let Stmt::ExternFunction { name, params, return_type, .. } = stmt {
                self.function_signatures.insert(name.lexeme.clone(), (
                    params
                        .iter()
                        .map(|p|
                            self.map_type(p.type_annotation.as_ref().expect("checked parameter"))
                        )
                        .collect(),
                    self.map_type(return_type.as_ref().unwrap_or(&Type::Void)),
                ));
            }
        }
        for (name, info) in &self.user_types {
            if !info.case_names.is_empty() {
                for case in &info.case_names {
                    let (ps, ret, _) = &info.static_methods[case];
                    self.function_signatures.insert(crate::symbols::member(name, case), (
                        ps
                            .iter()
                            .map(|t| self.map_type(t))
                            .collect(),
                        self.map_type(ret),
                    ));
                }
            }
        }
        for stmt in stmts {
            if let Stmt::Var { name, type_annotation, initializer, .. } = stmt {
                let ty = type_annotation
                    .as_ref()
                    .map(|t| self.map_type(t))
                    .unwrap_or_else(||
                        self.infer_ir_type(
                            initializer.as_ref().expect("checked global initializer")
                        )
                    );
                let arc = type_annotation
                    .as_ref()
                    .map(|t| self.is_arc_ty(t))
                    .unwrap_or_else(|| self.infer_is_arc(initializer.as_ref().unwrap()));
                let slot = self.new_reg();
                self.globals.insert(name.lexeme.clone(), (slot, ty, arc));
            }
        }
        let entry_block = self.new_block("entry");
        self.set_insert_point(entry_block);
        self.begin_scope();

        for stmt in stmts {
            match stmt {
                | Stmt::Function { .. }
                | Stmt::Struct { .. }
                | Stmt::Impl { .. }
                | Stmt::ExternFunction { .. } => {
                    let old_block = self.current_block.take();
                    self.lower_stmt(stmt);
                    self.current_block = old_block;
                }
                _ => {
                    self.lower_stmt(stmt);
                    if let Stmt::Var { name, .. } = stmt {
                        if let Some(v) = self.resolve_variable(&name.lexeme) {
                            self.globals.insert(name.lexeme.clone(), v);
                        }
                    }
                }
            }
        }

        self.scopes.pop();

        if !self.is_current_block_terminated() {
            self.emit(Instruction::Ret { value: None });
        }

        let mut main_blocks: Vec<_> = self.blocks
            .drain()
            .map(|(_, b)| b)
            .collect();
        main_blocks.sort_by_key(|b| b.id.0);

        self.functions.push(FunctionIr {
            name: "__global_init".to_string(),
            ret_type: IrType::Void,
            args: vec![],
            blocks: main_blocks,
            is_variadic: false,
        });

        let mut cleanup = Vec::new();
        // On normal completion every global initializer has succeeded. Drop in
        // reverse source/module initialization order, never HashMap iteration
        // order. Clear owning slots before any recursive destruction.
        let globals = stmts
            .iter()
            .rev()
            .filter_map(|stmt| {
                match stmt {
                    Stmt::Var { name, .. } => self.globals.get(&name.lexeme).cloned(),
                    _ => None,
                }
            })
            .collect::<Vec<_>>();
        for (slot, ty, arc) in globals {
            if arc {
                cleanup.push(Instruction::Ownership(Physical::DropSlot{slot,ty,guarded:false}));
            }
        }
        cleanup.push(Instruction::Ret { value: None });
        let block = BlockId(self.next_block);
        self.next_block += 1;
        self.functions.push(FunctionIr {
            name: "__fishy_global_cleanup".into(),
            ret_type: IrType::Void,
            args: vec![],
            is_variadic: false,
            blocks: vec![BasicBlock { id: block, name: "entry".into(), instructions: cleanup }],
        });
        // Normal main exits release globals in reverse initialization order.
        let mut exit = Vec::new();
        for name in ["__fishy_global_cleanup"] {
            let dest = self.new_reg();
            exit.push(Instruction::Call { dest, func_name: name.into(), args: vec![] });
        }
        exit.push(Instruction::Ret { value: None });
        let block = BlockId(self.next_block);
        self.next_block += 1;
        self.functions.push(FunctionIr {
            name: "__fishy_program_exit".into(),
            ret_type: IrType::Void,
            args: vec![],
            is_variadic: false,
            blocks: vec![BasicBlock { id: block, name: "entry".into(), instructions: exit }],
        });
        let mut functions: Vec<FunctionIr> = Vec::new();
        for function in std::mem::take(&mut self.functions) {
            if let Some(existing) = functions.iter_mut().find(|f| f.name == function.name) {
                if !function.blocks.is_empty() {
                    *existing = function;
                }
            } else {
                functions.push(function);
            }
        }
        self.functions = functions;
        let mut structs: HashMap<_, _> = self.user_types
            .keys()
            .map(|n| (n.clone(), self.get_struct_field_types(n)))
            .collect();
        let resources = self.user_types
            .iter()
            .filter_map(|(name, info)|
                info.resource_element.as_ref().map(|element| (name.clone(), self.map_type(element)))
            )
            .collect();
        structs.extend(self.synthetic_layouts);
        let mut module=ModuleIr {
            ownership_phase: crate::ir::OwnershipPhase::Explicit,
            shared_types: self.user_types.iter().filter(|(_,info)|info.immutable_payload).map(|(name,_)|name.clone()).collect(),
            resources,

            globals: self.globals
                .values()
                .map(|(r, t, _)| (*r, t.clone()))
                .collect(),
            structs,
            name: "Fishy_Module".to_string(),
            functions: self.functions,
            vtables: self.vtables,
        };
        crate::physical_ownership::formalize_storage(&mut module);
        module
    }

    fn inject_panic(&mut self, message: &str) {
        let msg_reg = self.new_reg();
        self.emit(Instruction::ConstString {
            dest: msg_reg,
            value: message.to_string(),
        });
        let dump_reg = self.new_reg();
        self.emit(Instruction::Call {
            dest: dump_reg,
            func_name: "panic".to_string(),
            args: vec![msg_reg],
        });
        self.emit(Instruction::Unreachable);
    }

    fn inject_null_check(&mut self, ptr: VReg, msg: &str) {
        let null_val = self.new_reg();
        self.emit(Instruction::ConstInt {
            dest: null_val,
            value: 0,
        });

        let is_not_null = self.new_reg();
        self.emit(Instruction::CmpNeq {
            dest: is_not_null,
            left: ptr,
            right: null_val,
        });

        let ok_block = self.new_block("nullcheck.ok");
        let fail_block = self.new_block("nullcheck.fail");

        self.emit(Instruction::CondBr {
            cond: is_not_null,
            if_true: ok_block,
            if_false: fail_block,
        });

        self.set_insert_point(fail_block);
        self.inject_panic(msg);

        self.set_insert_point(ok_block);
    }

    fn inject_bounds_check(&mut self, ptr: VReg, idx: VReg) {
        // read invisible header. size is in index -1
        let minus_one = self.new_reg();
        self.emit(Instruction::ConstInt {
            dest: minus_one,
            value: -1,
        });

        let size_ptr = self.new_reg();
        self.emit(Instruction::GetElementPtr {
            dest: size_ptr,
            base_ty: IrType::I64,
            base_ptr: ptr,
            indices: vec![minus_one],
        });

        let limit_reg = self.new_reg();
        self.emit(Instruction::Load {
            dest: limit_reg,
            ty: IrType::I64,
            src_ptr: size_ptr,
        });

        let zero_reg = self.new_reg();
        self.emit(Instruction::ConstInt {
            dest: zero_reg,
            value: 0,
        });

        let is_valid_upper = self.new_reg();
        self.emit(Instruction::CmpLt {
            dest: is_valid_upper,
            left: idx,
            right: limit_reg,
            ty: IrType::I64,
        });

        let lower_check_block = self.new_block("bounds.lower_check");
        let fail_block = self.new_block("bounds.fail");
        let ok_block = self.new_block("bounds.ok");

        self.emit(Instruction::CondBr {
            cond: is_valid_upper,
            if_true: lower_check_block,
            if_false: fail_block,
        });

        self.set_insert_point(lower_check_block);
        let is_valid_lower = self.new_reg();
        self.emit(Instruction::CmpGe {
            dest: is_valid_lower,
            left: idx,
            right: zero_reg,
            ty: IrType::I64,
        });
        self.emit(Instruction::CondBr {
            cond: is_valid_lower,
            if_true: ok_block,
            if_false: fail_block,
        });

        self.set_insert_point(fail_block);
        self.inject_panic("Array index out of bounds! Invalid memory access prevented.");

        self.set_insert_point(ok_block);
    }

    fn poison_expr(&mut self, expr: &Expr) {
        let mut target = expr;
        while let Expr::Cast { value, .. } | Expr::Grouping(value) = target.untyped() {
            target = value;
        }
        if
            matches!(
                target.untyped(),
                Expr::Variable(_) |
                    Expr::Get { .. } |
                    Expr::SubscriptGet { .. } |
                    Expr::Dereference { .. }
            )
        {
            let slot = self.lower_lvalue(target);
            let value = self.new_reg();
            self.emit(Instruction::ConstInt {
                dest: value,
                value: 0,
            });
            let ty = self.infer_ir_type(target);
            self.emit(Instruction::Store {
                ty,
                value,
                ptr: slot,
            });
        }
    }

    fn get_struct_field_types(&self, struct_name: &str) -> Vec<IrType> {
        if let Some(info) = self.user_types.get(struct_name) {
            let mut fields: Vec<(usize, IrType)> = info.fields
                .iter()
                .map(|(_, (index, ty, _))| (*index, self.map_type(ty)))
                .collect();
            fields.sort_by_key(|(index, _)| *index);
            return fields
                .into_iter()
                .map(|(_, ty)| ty)
                .collect();
        }
        vec![]
    }

    fn get_struct_field_info(&self, struct_name: &str) -> Vec<(String, IrType)> {
        if let Some(info) = self.user_types.get(struct_name) {
            let mut fields: Vec<(usize, String, IrType)> = info.fields
                .iter()
                .map(|(name, (index, ty, _))| (*index, name.clone(), self.map_type(ty)))
                .collect();
            fields.sort_by_key(|(index, _, _)| *index);
            return fields
                .into_iter()
                .map(|(_, name, ty)| (name, ty))
                .collect();
        }
        vec![]
    }

    pub fn map_type(&self, ast_type: &Type) -> IrType {
        match ast_type {
            Type::Void => IrType::Void,

            Type::U8 => IrType::U8,
            Type::U16 => IrType::U16,
            Type::U32 => IrType::U32,
            Type::U64 => IrType::U64,

            Type::I8 => IrType::I8,
            Type::I16 => IrType::I16,
            Type::I32 => IrType::I32,
            Type::I64 => IrType::I64,

            Type::F16 => IrType::F16,
            Type::F32 => IrType::F32,
            Type::F64 => IrType::F64,

            Type::Bool => IrType::Bool,

            Type::String => IrType::Ptr(Box::new(IrType::I8)),


            Type::Pointer(inner) | Type::Reference(inner) | Type::MutReference(inner) => {
                IrType::Ptr(Box::new(self.map_type(inner)))
            }

            Type::Slice(inner) => IrType::Array(0, Box::new(self.map_type(inner))),

            Type::Array(size, inner) => IrType::Array(*size, Box::new(self.map_type(inner))),

            Type::Custom(name) => {
                if self.traits.contains(name) {
                    IrType::FatPtr
                } else {
                    IrType::Ptr(Box::new(IrType::Struct(name.clone(), vec![])))
                }
            }

            Type::Range(inner) => {
                let element = self.map_type(inner);
                IrType::Struct(format!("__fishy_range_{}", inner), vec![element.clone(), element])
            }
            Type::Tuple(types) if types.is_empty() => IrType::I8,
            Type::Tuple(types) =>
                IrType::Ptr(
                    Box::new(
                        IrType::Struct(
                            format!("__tuple_{}", ast_type),
                            types
                                .iter()
                                .map(|t| self.map_type(t))
                                .collect()
                        )
                    )
                ),

            Type::Function(ps, r, _) =>
                IrType::Closure(
                    ps
                        .iter()
                        .map(|t| self.map_type(t))
                        .collect(),
                    Box::new(self.map_type(r))
                ),
            Type::TypeVar(_) => IrType::I64,
            _ => IrType::Any,
        }
    }

    fn infer_ir_type(&self, expr: &Expr) -> IrType {
        if let Some(ty) = expr.checked_type() {
            return self.map_type(ty);
        }
        match expr.untyped() {
            Expr::Literal(Literal::Number(_)) => IrType::F64,
            Expr::Literal(Literal::Integer(_)) => IrType::I64,
            Expr::Literal(Literal::String(_)) => IrType::Ptr(Box::new(IrType::I8)),
            Expr::Literal(Literal::Bool(_)) => IrType::Bool,

            Expr::StructInit { class_name, .. } => {
                let real_name = match self.resolved_calls.get(class_name) {
                    Some(CallType::Static(c)) | Some(CallType::Instance(c)) => c.clone(),
                    None => class_name.lexeme.clone(),
                };
                let field_info = self.get_struct_field_info(&real_name);
                if !field_info.is_empty() {
                    IrType::Ptr(
                        Box::new(
                            IrType::Struct(
                                real_name.clone(),
                                field_info
                                    .into_iter()
                                    .map(|(_, ty)| ty)
                                    .collect()
                            )
                        )
                    )
                } else {
                    IrType::Any
                }
            }

            Expr::Variable(name) | Expr::This(name) =>
                self
                    .resolve_variable(&name.lexeme)
                    .map(|(_, ty, _)| ty)
                    .unwrap_or(IrType::Any),

            Expr::Call { callee, .. } => {
                if let Expr::Variable(name) = callee.untyped() {
                    if let Some(func) = self.functions.iter().find(|f| f.name == name.lexeme) {
                        return func.ret_type.clone();
                    }
                }
                IrType::Any
            }

            Expr::Grouping(e) | Expr::Unary { right: e, .. } => self.infer_ir_type(e),
            Expr::Binary { left, .. } => self.infer_ir_type(left),
            Expr::Array { elements, .. } =>
                IrType::Array(elements.len(), Box::new(self.infer_array_element_type(elements))),
            Expr::Get { object, name } => self.get_property_ir_type(object, &name.lexeme),
            Expr::SubscriptGet { indexee, .. } => self.deref_type(&self.infer_ir_type(indexee)),
            Expr::Cast { target_type, .. } => self.map_type(target_type),
            Expr::Dereference { operand, .. } => self.deref_type(&self.infer_ir_type(operand)),
            Expr::AddressOf { operand, .. } => IrType::Ptr(Box::new(self.infer_ir_type(operand))),

            _ => IrType::Any,
        }
    }

    fn deref_type(&self, ty: &IrType) -> IrType {
        match ty {
            IrType::Ptr(inner) => (**inner).clone(),
            IrType::Array(_, inner) => (**inner).clone(),
            other => other.clone(),
        }
    }

    fn object_type(&self, object: &Expr) -> IrType {
        let mut ty = self.infer_ir_type(object);
        while let IrType::Ptr(inner) = ty {
            ty = *inner;
        }
        ty
    }

    fn object_value_type(&self, object: &Expr) -> IrType {
        if let Some(mut ty) = object.checked_type() {
            while
                let Type::Pointer(inner) | Type::Reference(inner) | Type::MutReference(inner) = ty
            {
                ty = inner;
            }
            return self.map_type(ty);
        }
        self.infer_ir_type(object)
    }

    fn lower_object(&mut self, object: &Expr) -> VReg {
        let mut value = self.lower_expr(object);
        let mut source = object.checked_type();
        while
            let Some(Type::Pointer(inner) | Type::Reference(inner) | Type::MutReference(inner)) =
                source
        {
            self.inject_null_check(value, "Null object indirection");
            let dest=self.new_reg();
            self.emit(Instruction::Load{dest,ty:self.map_type(inner),src_ptr:value});
            value=dest;
            source = Some(inner);
        }
        value
    }

    fn lower_receiver(&mut self, object: &Expr) -> VReg {
        if

            !object
                .identity()
                .is_some_and(|id| self.ownership.as_ref().unwrap().receiver_values.contains(&id))
        {
            self.lower_expr(object)
        } else {
            self.lower_object(object)
        }
    }
    fn receiver_mode(&self, owner: &str, method: &str) -> crate::ir::ReceiverMode {
        use crate::ir::ReceiverMode;

        match
            self.user_types
                .get(owner)
                .and_then(|i| i.methods.get(method))
                .and_then(|(ps, _, _)| ps.first())
        {
            Some(Type::Reference(_)) => ReceiverMode::Shared,
            Some(Type::MutReference(_)) => ReceiverMode::Exclusive,
            _ => ReceiverMode::Consume,
        }
    }
    fn is_borrowed_expr(&self, expr: &Expr) -> bool {
        expr.identity().is_some_and(|id|
            self.ownership
                .as_ref()
                .is_some_and(
                    |p| (p.borrowed.contains_key(&id) || p.borrowed_temporaries.contains(&id))
                )
        )
    }

    fn get_property_ir_type(&self, object: &Expr, property_name: &str) -> IrType {
        let obj_ty = self.object_type(object);

        if let IrType::Struct(ref name, ref types) = obj_ty {
            if name.starts_with("__tuple_") {
                return types[property_name.parse::<usize>().expect("checked tuple field")].clone();
            }
        }
        if let IrType::Struct(s_name, _) = obj_ty {
            if let Some(info) = self.user_types.get(&s_name) {
                if let Some((_, ast_ty, _)) = info.fields.get(property_name) {
                    return self.map_type(ast_ty);
                }
            }
        }

        IrType::I64
    }

    fn infer_array_element_type(&self, elements: &[Expr]) -> IrType {
        if let Some(first) = elements.first() {
            let first_ty = self.infer_ir_type(first);
            if elements.iter().all(|el| self.infer_ir_type(el) == first_ty) {
                first_ty
            } else {
                IrType::Any
            }
        } else {
            IrType::Any
        }
    }

    fn new_reg(&mut self) -> VReg {
        let reg = VReg(self.next_reg);
        self.next_reg += 1;
        reg
    }

    fn new_block(&mut self, name: &str) -> BlockId {
        let id = BlockId(self.next_block);
        self.next_block += 1;

        self.blocks.insert(id, BasicBlock {
            id,
            name: name.to_string(),
            instructions: Vec::new(),
        });

        id
    }

    fn set_insert_point(&mut self, block: BlockId) {
        self.current_block = Some(block);
    }

    fn emit(&mut self, inst: Instruction) {
        if let Instruction::Alloca{dest,..}=&inst {self.stack_slots.insert(*dest);}
        assert!(!matches!(inst,Instruction::Retain{..}|Instruction::Release{..}),"ARC insertion belongs to physical lifetime lowering");
        if let Some(block_id) = self.current_block {
            if let Some(block) = self.blocks.get_mut(&block_id) {
                block.instructions.push(inst);
            }
        } else {
            eprintln!("WARNING: Instruction emitted with no current block: {:?}", inst);
        }
    }

    fn transfer_call_arguments(&mut self) {
        let pins=self.scopes.last().into_iter().flat_map(|s|s.values())
            .filter_map(|(slot,ty,_)| self.temporary_values.get(slot).map(|value|(*slot,*value,ty.clone())))
            .collect::<Vec<_>>();
        for (slot,value,ty) in pins {
            self.emit(Instruction::Ownership(Physical::TransferSlot{slot,value,ty}));
        }
    }

    fn emit_source_call(&mut self, instruction: Instruction) {
        self.transfer_call_arguments();
        self.emit(instruction);
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        if let Some(scope) = self.scopes.pop() {
            if !self.is_current_block_terminated() {
                self.emit_releases_for_scope(&scope);
            }
        }
    }

    fn emit_releases_for_scope(&mut self, scope: &HashMap<String, (VReg, IrType, bool)>) {
        let mut entries = scope.values().collect::<Vec<_>>();
        entries.sort_by_key(|(slot,_,_)|std::cmp::Reverse(self.cleanup_order.get(slot).copied().unwrap_or(0)));
        for (slot,ty,owning) in entries {
            if *owning && !self.globals.values().any(|(r,_,_)|r==slot) {
                self.emit(Instruction::Ownership(Physical::DropSlot{slot:*slot,ty:ty.clone(),guarded:true}));
            }
        }
    }

    fn register_owner_slot(&mut self, slot:VReg, ty:IrType, kind:SlotKind) {
        if self.physical_slots.insert(slot) {self.emit(Instruction::Ownership(Physical::OwnerSlot{slot,ty,kind}));}
    }

    fn declare_binding(&mut self, name: &Token, ptr_reg: VReg, ty: IrType, is_arc: bool) {
        self.binding_slots.insert(name.id, (ptr_reg, ty.clone(), is_arc));
        self.declare_variable(name.lexeme.clone(), ptr_reg, ty, is_arc);
    }

    fn declare_variable(&mut self, name: String, ptr_reg: VReg, ty: IrType, is_arc: bool) {
        if is_arc {self.register_owner_slot(ptr_reg,ty.clone(),SlotKind::Local);}
        else if self.is_arc_ir_type(&ty)&&self.stack_slots.contains(&ptr_reg)&&self.physical_slots.insert(ptr_reg) {
            self.emit(Instruction::Ownership(Physical::ViewSlot{slot:ptr_reg,ty:ty.clone()}));
        }
        let next = self.cleanup_order.len();
        self.cleanup_order.entry(ptr_reg).or_insert(next);
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, (ptr_reg, ty, is_arc));
        }
    }

    fn resolve_variable(&self, name: &str) -> Option<(VReg, IrType, bool)> {
        for scope in self.scopes.iter().rev() {
            if let Some((reg, ty, is_arc)) = scope.get(name) {
                return Some((*reg, ty.clone(), *is_arc));
            }
        }
        None
    }

    fn is_current_block_terminated(&self) -> bool {
        if let Some(block_id) = self.current_block {
            if let Some(block) = self.blocks.get(&block_id) {
                if let Some(last_inst) = block.instructions.last() {
                    return matches!(
                        last_inst,
                        Instruction::Br { .. } |
                            Instruction::CondBr { .. } |
                            Instruction::Ret { .. } |
                            Instruction::Unreachable
                    );
                }
            }
        }
        false
    }

    fn is_arc_ty(&self, ty: &Type) -> bool {
        self.ownership.as_ref().expect("checked source ownership").types.get(ty).needs_drop
    }

    fn is_arc_ir_type(&self, ty: &IrType) -> bool { crate::physical_ownership::managed(ty) }

    fn infer_is_arc(&self, expr: &Expr) -> bool {
        if let Some(ty) = expr.checked_type() {
            return self.is_arc_ty(ty);
        }
        match expr.untyped() {
            Expr::Array { .. } | Expr::Lambda { .. } => true,
            Expr::StructInit { .. } => true,
            Expr::Variable(name) | Expr::This(name) =>
                self
                    .resolve_variable(&name.lexeme)
                    .map(|(_, _, is_arc)| is_arc)
                    .unwrap_or(false),

            _ => false,
        }
    }

    fn has_result_obligation(&self, expr: &Expr) -> bool {
        use crate::ownership::ValueKind;
        expr.identity().and_then(|id| self.ownership.as_ref().unwrap().values.get(&id))
            .is_some_and(|k| matches!(k,ValueKind::Move|ValueKind::Produced))
    }

    // A compiler-owned evaluation pin is a normal scoped owner. Keeping it
    // in the cleanup scope also covers returns nested in later expressions.
    fn hold_owned_value(&mut self, value: VReg, ty: IrType) -> VReg {
        self.hold_temporary(value,ty,SlotKind::Temporary)
    }
    fn hold_temporary(&mut self, value: VReg, ty: IrType, kind: SlotKind) -> VReg {
        let slot = self.new_reg();
        self.emit(Instruction::Alloca { dest: slot, name: "owner.pin".into(), ty: ty.clone() });
        self.emit(Instruction::Store { ty: ty.clone(), value, ptr: slot });
        self.physical_slots.insert(slot);
        self.emit(Instruction::Ownership(Physical::OwnerSlot{slot,ty:ty.clone(),kind}));
        self.temporary_values.insert(slot,value);
        self.declare_variable(format!("#owner.{}", slot.0), slot, ty, true);
        slot
    }

    // The zero-initialized shell is an owner before evaluating children. A
    // nested return can therefore destroy exactly the fields initialized so far.
    fn begin_construction(&mut self, value: VReg, ty: IrType) -> VReg {
        self.begin_scope();
        self.hold_temporary(value, ty, SlotKind::Construction)
    }

    fn finish_construction(&mut self, pin: VReg, ty: IrType) {
        let value=self.temporary_values[&pin];
        self.emit(Instruction::Ownership(Physical::TransferSlot{slot:pin,value,ty}));
        self.end_scope();
    }

    fn hold_expr_value(&mut self, expr: &Expr, value: VReg) -> bool {
        if !self.infer_is_arc(expr) || !self.has_result_obligation(expr) {return false;}
        self.hold_owned_value(value,self.infer_ir_type(expr));
        true
    }

    fn discard_expr(&mut self, expr: &Expr) {
        let value = self.lower_expr(expr);
        if
            !self.is_current_block_terminated() &&
            self.infer_is_arc(expr) &&
            self.has_result_obligation(expr)
        {
            self.emit(Instruction::Ownership(Physical::DropValue{value:value,recipe:crate::physical_ownership::DropRecipe::Temporary}));
        } else {
        }
    }

    fn replace_owned_slot(&mut self, slot: VReg, value: VReg, expr: &Expr, ty: &IrType) {
        if self.is_arc_ir_type(ty) {
            if !self.has_result_obligation(expr) {self.emit(Instruction::Ownership(Physical::AcquireSharedEdge{value:value,kind:EdgeKind::Evaluation}));}
            self.emit(Instruction::Ownership(Physical::DropSlot{slot,ty:ty.clone(),guarded:false}));
        }
    }

    fn check_index(&mut self, indexee: &Expr, ptr: VReg, idx: VReg) {
        let mut ty = indexee.checked_type();
        while let Some(Type::Reference(t) | Type::MutReference(t)) = ty {
            ty = Some(t);
        }
        if matches!(ty, Some(Type::Array(_, _) | Type::Slice(_))) {
            self.inject_bounds_check(ptr, idx);
        }
        // Raw pointers, including libc buffers, do not carry a managed header.
        // Their bounds are the caller's responsibility.
    }

    fn index_element_type(&self, indexee: &Expr) -> IrType {
        let mut ty = indexee.checked_type();
        while let Some(Type::Reference(t) | Type::MutReference(t)) = ty {
            ty = Some(t);
        }
        match ty {
            Some(Type::Array(_, t) | Type::Slice(t) | Type::Pointer(t)) => self.map_type(t),
            Some(Type::String) => IrType::U8,
            _ => self.deref_type(&self.infer_ir_type(indexee)),
        }
    }

    fn referenced_type(mut ty: &Type) -> &Type {
        while let Type::Reference(inner) | Type::MutReference(inner) = ty {
            ty = inner;
        }
        ty
    }

    fn lower_index_base(&mut self, indexee: &Expr) -> VReg {
        let mut value = self.lower_expr(indexee);

            let mut ty = indexee.checked_type();
            while let Some(Type::Reference(t) | Type::MutReference(t)) = ty {
                let loaded = self.new_reg();
                self.emit(Instruction::Load { dest: loaded, ty: self.map_type(t), src_ptr: value });
                value = loaded;
                ty = Some(t);
            }

        value
    }

    fn lower_array(&mut self, elements: &[Expr], element: IrType) -> VReg {
        let length = elements
            .iter()
            .map(|e| {
                if let Expr::Spread { right, .. } = e.untyped() {
                    match Self::referenced_type(right.checked_type().expect("checked spread")) {
                        Type::Array(n, _) => *n,
                        Type::Tuple(ts) => ts.len(),
                        _ => unreachable!("checked spread type"),
                    }
                } else {
                    1
                }
            })
            .sum::<usize>();
        let size = self.new_reg();
        self.emit(Instruction::ConstInt {
            dest: size,
            value: length as i64,
        });
        let dest = self.new_reg();
        self.emit(Instruction::AllocArray {
            dest,
            size,
            ty: element.clone(),
        });
        let array_ty = IrType::Array(length, Box::new(element.clone()));
        let construction = self.begin_construction(dest, array_ty.clone());

        let mut index = 0;
        for expr in elements {
            let mut values = Vec::new();
            let mut temporary = None;
            if let Expr::Spread { right, .. } = expr.untyped() {
                let data = self.lower_index_base(right);
                if self.is_current_block_terminated() {
                    self.end_scope();
                    return dest;
                }
                let source_type = Self::referenced_type(right.checked_type().unwrap());
                let ty = self.map_type(source_type);
                if !matches!(source_type,Type::Tuple(ts) if ts.is_empty()) {
                    self.inject_null_check(data, "Null array spread");
                }
                let (count, tuple) = match source_type {
                    Type::Array(n, _) => (*n, false),
                    Type::Tuple(ts) => (ts.len(), true),
                    _ => unreachable!(),
                };
                for i in 0..count {
                    let ptr = if tuple {
                        self.field_address(data, self.deref_type(&ty), i)
                    } else {
                        let idx = self.new_reg();
                        self.emit(Instruction::ConstInt {
                            dest: idx,
                            value: i as i64,
                        });
                        let ptr = self.new_reg();
                        self.emit(Instruction::GetElementPtr {
                            dest: ptr,
                            base_ty: element.clone(),
                            base_ptr: data,
                            indices: vec![idx],
                        });
                        ptr
                    };
                    let val = self.new_reg();
                    self.emit(Instruction::Load {
                        dest: val,
                        ty: element.clone(),
                        src_ptr: ptr,
                    });
                    if self.is_arc_ir_type(&element) {
                        self.emit(Instruction::Ownership(Physical::AcquireSharedEdge{value:val,kind:EdgeKind::Evaluation}));
                    }
                    values.push(val);
                }
                if self.infer_is_arc(right) && self.has_result_obligation(right) {
                    temporary = Some(data);
                }
            } else {
                let val = self.lower_expr(expr);
                if self.is_current_block_terminated() {
                    self.end_scope();
                    return dest;
                }
                if self.infer_is_arc(expr) && !self.has_result_obligation(expr) {
                    self.emit(Instruction::Ownership(Physical::AcquireSharedEdge{value:val,kind:EdgeKind::Evaluation}));
                }
                values.push(val);
            }
            for value in values {
                let idx = self.new_reg();
                self.emit(Instruction::ConstInt {
                    dest: idx,
                    value: index,
                });
                index += 1;
                let ptr = self.new_reg();
                self.emit(Instruction::GetElementPtr {
                    dest: ptr,
                    base_ty: element.clone(),
                    base_ptr: dest,
                    indices: vec![idx],
                });

                self.emit(Instruction::Store {
                    ty: element.clone(),
                    value,
                    ptr,
                });
            }
            if let Some(ptr) = temporary {
                self.emit(Instruction::Ownership(Physical::DropValue {
                    value: ptr, recipe: crate::physical_ownership::DropRecipe::Temporary,
                }));
            }
        }

        self.finish_construction(construction, array_ty);
        dest
    }

    fn method_value(&mut self, object: &Expr, member: &Token) -> VReg {
        let call = self.resolved_calls.get(member).expect("resolved method value").clone();
        let (name, instance) = match call {
            CallType::Instance(n) => (n, true),
            CallType::Static(n) => (n, false),
        };
        let info = &self.user_types[&name];
        let (ps, ret, _) = if instance {
            &info.methods[&member.lexeme]
        } else {
            &info.static_methods[&member.lexeme]
        };
        let params = ps
            .iter()
            .skip(if instance { 1 } else { 0 })
            .map(|t| self.map_type(t))
            .collect::<Vec<_>>();
        let ret = self.map_type(ret);
        let target = crate::symbols::member(&name, &member.lexeme);
        if !instance {
            return self.named_function_value(&target, params, ret);
        }
        let receiver_type = self.map_type(&ps[0]);
        let receiver = self.lower_receiver(object);
        if self.is_arc_ir_type(&receiver_type) && !self.has_result_obligation(object) {
            self.emit(Instruction::Ownership(Physical::AcquireSharedEdge{value:receiver,kind:EdgeKind::Evaluation}));
        }
        let adapter = format!("__fishy_bound_{target}");
        let env_name = format!("{adapter}.env");
        let env_type = IrType::Struct(env_name.clone(), vec![receiver_type.clone()]);
        self.synthetic_layouts.insert(env_name.clone(), vec![receiver_type.clone()]);
        let env = self.new_reg();
        self.emit(Instruction::AllocStruct {
            dest: env,
            class_name: env_name,
        });
        let slot = self.field_address(env, env_type.clone(), 0);
        self.emit(Instruction::Store {
            ty: receiver_type.clone(),
            value: receiver,
            ptr: slot,
        });
        if !self.functions.iter().any(|f| f.name == adapter) {
            let old_blocks = std::mem::take(&mut self.blocks);
            let old_current = self.current_block.take();
            let mut args = vec![(self.new_reg(), IrType::Ptr(Box::new(IrType::U8)))];
            args.extend(params.iter().map(|t| (self.new_reg(), t.clone())));
            let entry = self.new_block("entry");
            self.set_insert_point(entry);
            let ptr = self.field_address(args[0].0, env_type, 0);
            let receiver = self.new_reg();
            self.emit(Instruction::Load {
                dest: receiver,
                ty: receiver_type,
                src_ptr: ptr,
            });
            let dest = self.new_reg();
            let actual = args
                .iter()
                .skip(1)
                .map(|(r, _)| *r)
                .collect::<Vec<_>>();
            if self.traits.contains(&name) {
                let index = self.trait_vtable_layout[&name][&member.lexeme];
                self.emit(Instruction::DynamicCall {
                    receiver_mode: self.receiver_mode(&name, &member.lexeme),
                    dest,
                    vtable_index: index,
                    fat_ptr: receiver,
                    args: actual,
                    arg_types: params,
                    ret_type: ret.clone(),
                });
            } else {
                let mut actual_args = vec![receiver];
                actual_args.extend(actual);
                self.emit(Instruction::Call {
                    dest,
                    func_name: target,
                    args: actual_args,
                });
            }
            self.emit(Instruction::Ret {
                value: if ret == IrType::Void {
                    None
                } else {
                    Some(dest)
                },
            });
            let mut blocks = self.blocks
                .drain()
                .map(|(_, b)| b)
                .collect::<Vec<_>>();
            blocks.sort_by_key(|b| b.id.0);
            self.functions.push(FunctionIr {
                name: adapter.clone(),
                args,
                ret_type: ret,
                blocks,
                is_variadic: false,
            });
            self.blocks = old_blocks;
            self.current_block = old_current;
        }
        let dest = self.new_reg();
        self.emit(Instruction::MakeClosure {
            dest,
            fn_name: adapter,
            env_ptr: env,
        });
        dest
    }

    fn named_function_value(
        &mut self,
        target: &str,
        params: Vec<IrType>,
        ret_type: IrType
    ) -> VReg {
        let name = format!("__fishy_function_value_{target}");
        if !self.functions.iter().any(|f| f.name == name) {
            let mut args = vec![(self.new_reg(), IrType::Ptr(Box::new(IrType::U8)))];
            args.extend(params.into_iter().map(|t| (self.new_reg(), t)));
            let result = self.new_reg();
            let block = BlockId(self.next_block);
            self.next_block += 1;
            self.functions.push(FunctionIr {
                name: name.clone(),
                args: args.clone(),
                ret_type: ret_type.clone(),
                is_variadic: false,
                blocks: vec![BasicBlock {
                    id: block,
                    name: "entry".into(),
                    instructions: vec![
                        Instruction::Call {
                            dest: result,
                            func_name: target.into(),
                            args: args
                                .iter()
                                .skip(1)
                                .map(|(r, _)| *r)
                                .collect(),
                        },
                        Instruction::Ret {
                            value: if ret_type == IrType::Void {
                                None
                            } else {
                                Some(result)
                            },
                        }
                    ],
                }],
            });
        }
        let env = self.new_reg();
        self.emit(Instruction::ConstInt {
            dest: env,
            value: 0,
        });
        let dest = self.new_reg();
        self.emit(Instruction::MakeClosure {
            dest,
            fn_name: name,
            env_ptr: env,
        });
        dest
    }

    fn lower_power(&mut self, base: VReg, exponent: VReg, ty: IrType) -> VReg {
        if matches!(ty, IrType::F16 | IrType::F32 | IrType::F64) {
            let l = self.new_reg();
            self.emit(Instruction::Cast {
                dest: l,
                value: base,
                source_ty: ty.clone(),
                target_ty: IrType::F64,
            });
            let r = self.new_reg();
            self.emit(Instruction::Cast {
                dest: r,
                value: exponent,
                source_ty: ty,
                target_ty: IrType::F64,
            });
            let dest = self.new_reg();
            self.emit(Instruction::Call {
                dest,
                func_name: "pow".into(),
                args: vec![l, r],
            });
            return dest;
        }
        let zero = self.new_reg();
        self.emit(Instruction::ConstInt {
            dest: zero,
            value: 0,
        });
        let one = self.new_reg();
        self.emit(Instruction::ConstInt {
            dest: one,
            value: 1,
        });
        if matches!(ty, IrType::I8 | IrType::I16 | IrType::I32 | IrType::I64) {
            let positive = self.new_reg();
            self.emit(Instruction::CmpGe {
                dest: positive,
                left: exponent,
                right: zero,
                ty: ty.clone(),
            });
            let ok = self.new_block("power.valid");
            let bad = self.new_block("power.negative");
            self.emit(Instruction::CondBr {
                cond: positive,
                if_true: ok,
                if_false: bad,
            });
            self.set_insert_point(bad);
            self.inject_panic("Integer exponent must be nonnegative");
            self.set_insert_point(ok);
        }
        let bslot = self.new_reg();
        let eslot = self.new_reg();
        let result = self.new_reg();
        for (slot, name, value) in [
            (bslot, "power.base", base),
            (eslot, "power.exp", exponent),
            (result, "power.result", one),
        ] {
            self.emit(Instruction::Alloca {
                dest: slot,
                name: name.into(),
                ty: IrType::I64,
            });
            self.emit(Instruction::Store {
                ty: IrType::I64,
                value,
                ptr: slot,
            });
        }
        let test = self.new_block("power.test");
        let body = self.new_block("power.body");
        let end = self.new_block("power.end");
        self.emit(Instruction::Br { target: test });
        self.set_insert_point(test);
        let exp = self.new_reg();
        self.emit(Instruction::Load {
            dest: exp,
            ty: IrType::I64,
            src_ptr: eslot,
        });
        let cond = self.new_reg();
        self.emit(Instruction::CmpNeq {
            dest: cond,
            left: exp,
            right: zero,
        });
        self.emit(Instruction::CondBr {
            cond,
            if_true: body,
            if_false: end,
        });
        self.set_insert_point(body);
        let b = self.new_reg();
        self.emit(Instruction::Load {
            dest: b,
            ty: IrType::I64,
            src_ptr: bslot,
        });
        let odd = self.new_reg();
        self.emit(Instruction::BitAnd {
            dest: odd,
            left: exp,
            right: one,
        });
        let mul = self.new_block("power.multiply");
        let square = self.new_block("power.square");
        self.emit(Instruction::CondBr {
            cond: odd,
            if_true: mul,
            if_false: square,
        });
        self.set_insert_point(mul);
        let old = self.new_reg();
        self.emit(Instruction::Load {
            dest: old,
            ty: IrType::I64,
            src_ptr: result,
        });
        let product = self.new_reg();
        self.emit(Instruction::Mul {
            dest: product,
            left: old,
            right: b,
        });
        self.emit(Instruction::Store {
            ty: IrType::I64,
            value: product,
            ptr: result,
        });
        self.emit(Instruction::Br { target: square });
        self.set_insert_point(square);
        let squared = self.new_reg();
        self.emit(Instruction::Mul {
            dest: squared,
            left: b,
            right: b,
        });
        self.emit(Instruction::Store {
            ty: IrType::I64,
            value: squared,
            ptr: bslot,
        });
        let two = self.new_reg();
        self.emit(Instruction::ConstInt {
            dest: two,
            value: 2,
        });
        let next = self.new_reg();
        self.emit(Instruction::Div {
            dest: next,
            left: exp,
            right: two,
            ty: IrType::U64,
        });
        self.emit(Instruction::Store {
            ty: IrType::I64,
            value: next,
            ptr: eslot,
        });
        self.emit(Instruction::Br { target: test });
        self.set_insert_point(end);
        let dest = self.new_reg();
        self.emit(Instruction::Load {
            dest,
            ty: IrType::I64,
            src_ptr: result,
        });
        dest
    }

    fn enum_pattern(&self, pattern: &Expr, ty: &IrType) -> Option<(String, usize, Vec<IrType>)> {
        let IrType::Ptr(inner) = ty else {
            return None;
        };
        let IrType::Struct(n, _) = inner.as_ref() else {
            return None;
        };
        let info = self.user_types.get(n)?;
        let name = match pattern.untyped() {
            Expr::UnionPattern { case_name, .. } | Expr::Variable(case_name) => &case_name.lexeme,
            _ => {
                return None;
            }
        };
        let tag = info.case_names.iter().position(|c| c == name)?;
        let ps = info.static_methods[name].0
            .iter()
            .map(|t| self.map_type(t))
            .collect();
        Some((format!("{}#{}", n, name), tag, ps))
    }

    fn pattern_branch(&mut self, cond: VReg, fail: BlockId) {
        let yes = self.new_block("pattern.pass");
        self.emit(Instruction::CondBr {
            cond,
            if_true: yes,
            if_false: fail,
        });
        self.set_insert_point(yes);
    }

    fn pattern_fields(
        &mut self,
        pattern: &Expr,
        value: VReg,
        ty: &IrType,
        consuming: bool
    ) -> Vec<(Expr, VReg, IrType)> {
        match pattern.untyped() {
            Expr::ListPattern { elements, .. } =>
                elements
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        let element = self.deref_type(ty);
                        let idx = self.new_reg();
                        self.emit(Instruction::ConstInt {
                            dest: idx,
                            value: i as i64,
                        });
                        let ptr = self.new_reg();
                        self.emit(Instruction::GetElementPtr {
                            dest: ptr,
                            base_ty: element.clone(),
                            base_ptr: value,
                            indices: vec![idx],
                        });
                        if consuming  {
                            return self.take_pattern_child(p, ptr, element);
                        }
                        if
                            self.ownership
                                .as_ref()
                                .is_some_and(
                                    |plan|
                                        matches!(p.untyped(),Expr::Variable(n) if plan.pattern_views.contains(&n.id))
                                )
                        {
                            return (
                                p.clone(),
                                ptr,
                                self.map_type(p.checked_type().expect("typed borrowed pattern")),
                            );
                        }
                        let val = self.new_reg();
                        self.emit(Instruction::Load {
                            dest: val,
                            ty: element.clone(),
                            src_ptr: ptr,
                        });
                        (p.clone(), val, element)
                    })
                    .collect(),
            Expr::ObjectPattern { properties, .. } =>
                properties
                    .iter()
                    .map(|(n, p)| {
                        let base = self.deref_type(ty);
                        let IrType::Struct(ref name, _) = base else {
                            unreachable!("checked object pattern")
                        };
                        let (index, field, _) = self.user_types[name].fields[&n.lexeme].clone();
                        let field = self.map_type(&field);
                        let ptr = self.field_address(value, base, index);
                        if consuming  {
                            return self.take_pattern_child(p, ptr, field);
                        }
                        if
                            self.ownership
                                .as_ref()
                                .is_some_and(
                                    |plan|
                                        matches!(p.untyped(),Expr::Variable(n) if plan.pattern_views.contains(&n.id))
                                )
                        {
                            return (
                                p.clone(),
                                ptr,
                                self.map_type(p.checked_type().expect("typed borrowed pattern")),
                            );
                        }
                        let val = self.new_reg();
                        self.emit(Instruction::Load {
                            dest: val,
                            ty: field.clone(),
                            src_ptr: ptr,
                        });
                        (p.clone(), val, field)
                    })
                    .collect(),
            _ => vec![],
        }
    }

    fn binding_moves(&self, id:usize)->bool {self.ownership.as_ref().unwrap().noncopy_bindings.contains(&id)}

    fn take_pattern_child(
        &mut self,
        pattern: &Expr,
        ptr: VReg,
        ty: IrType
    ) -> (Expr, VReg, IrType) {
        let value = self.new_reg();
        let take=(self.is_arc_ir_type(&ty)||matches!(pattern.untyped(),Expr::Variable(n) if self.binding_moves(n.id))) && !matches!(pattern.untyped(),Expr::WildcardPattern(_));
        self.emit(Instruction::Ownership(Physical::ReadValue{dest:value,source:ptr,ty:ty.clone(),mode:if take{ReadMode::Move}else{ReadMode::Observe}}));
        if take && !matches!(pattern.untyped(), Expr::Variable(_)) {self.hold_owned_value(value,ty.clone());}
        (pattern.clone(), value, ty)
    }

    fn test_pattern(&mut self, pattern: &Expr, value: VReg, ty: &IrType, fail: BlockId) {
        if let Some((_, tag, _)) = self.enum_pattern(pattern, ty) {
            self.inject_null_check(value, "Null enum match");
            let actual = self.new_reg();
            self.emit(Instruction::Load {
                dest: actual,
                ty: IrType::I64,
                src_ptr: value,
            });
            let expected = self.new_reg();
            self.emit(Instruction::ConstInt {
                dest: expected,
                value: tag as i64,
            });
            let cond = self.new_reg();
            self.emit(Instruction::CmpEq {
                dest: cond,
                left: actual,
                right: expected,
            });
            self.pattern_branch(cond, fail);
            return;
        }
        match pattern.untyped() {
            Expr::Literal(_) => {
                let expected = self.lower_expr(pattern);
                let cond = self.new_reg();
                self.emit(Instruction::CmpEq {
                    dest: cond,
                    left: value,
                    right: expected,
                });
                self.pattern_branch(cond, fail);
            }
            Expr::ListPattern { elements, rest } => {
                self.inject_null_check(value, "Null list pattern");
                let idx = self.new_reg();
                self.emit(Instruction::ConstInt {
                    dest: idx,
                    value: -1,
                });
                let ptr = self.new_reg();
                self.emit(Instruction::GetElementPtr {
                    dest: ptr,
                    base_ty: IrType::I64,
                    base_ptr: value,
                    indices: vec![idx],
                });
                let len = self.new_reg();
                self.emit(Instruction::Load {
                    dest: len,
                    ty: IrType::I64,
                    src_ptr: ptr,
                });
                let count = self.new_reg();
                self.emit(Instruction::ConstInt {
                    dest: count,
                    value: elements.len() as i64,
                });
                let cond = self.new_reg();
                self.emit(
                    if rest.is_some() {
                        Instruction::CmpGe {
                            dest: cond,
                            left: len,
                            right: count,
                            ty: IrType::I64,
                        }
                    } else {
                        Instruction::CmpEq {
                            dest: cond,
                            left: len,
                            right: count,
                        }
                    }
                );
                self.pattern_branch(cond, fail);
                for (p, v, t) in self.pattern_fields(pattern, value, ty, false) {
                    self.test_pattern(&p, v, &t, fail);
                }
            }
            Expr::ObjectPattern { .. } => {
                self.inject_null_check(value, "Null object pattern");
                for (p, v, t) in self.pattern_fields(pattern, value, ty, false) {
                    self.test_pattern(&p, v, &t, fail);
                }
            }
            Expr::Variable(_) | Expr::WildcardPattern(_) => {}
            _ => unreachable!("checked match pattern"),
        }
    }

    fn bind_pattern(&mut self, pattern: &Expr, value: VReg, ty: &IrType, consuming: bool) {
        if let Some((name, _, ps)) = self.enum_pattern(pattern, ty) {
            if let Expr::UnionPattern { bindings, .. } = pattern.untyped() {
                let mut fields = vec![IrType::I64];
                fields.extend(ps.clone());
                let base = IrType::Struct(name, fields);
                for (i, (binding, ty)) in bindings.iter().zip(ps).enumerate() {
                    let ptr = self.field_address(value, base.clone(), i + 1);
                    if
                        self.ownership
                            .as_ref()
                            .is_some_and(|plan| plan.pattern_views.contains(&binding.id))
                    {
                        let binding_type = self.ownership
                            .as_ref()
                            .unwrap()
                            .pattern_bindings[&binding.id].clone();
                        self.bind_value(binding, ptr, self.map_type(&binding_type));
                        continue;
                    }
                    let val = self.new_reg();
                    self.emit(Instruction::Ownership(Physical::ReadValue{dest:val,source:ptr,ty:ty.clone(),mode:if consuming&&(self.is_arc_ir_type(&ty)||self.binding_moves(binding.id)){ReadMode::Move}else{ReadMode::Observe}}));
                    self.bind_value(binding, val, ty);
                }
            }
            return;
        }
        if let Expr::Variable(n) = pattern.untyped() {
            self.bind_value(n, value, ty.clone());
        }
        for (p, v, t) in self.pattern_fields(pattern, value, ty, consuming) {
            self.bind_pattern(&p, v, &t, consuming);
        }
        if let Expr::ListPattern { elements, rest: Some(rest) } = pattern.untyped() {
            let Expr::Variable(name) = rest.untyped() else {
                unreachable!("checked remainder binding")
            };
            let remainder_type = self.map_type(
                rest.checked_type().expect("checked remainder type")
            );
            let element = self.deref_type(ty);
            let minus_one = self.new_reg();
            self.emit(Instruction::ConstInt { dest: minus_one, value: -1 });
            let len_ptr = self.new_reg();
            self.emit(Instruction::GetElementPtr {
                dest: len_ptr,
                base_ty: IrType::I64,
                base_ptr: value,
                indices: vec![minus_one],
            });
            let length = self.new_reg();
            self.emit(Instruction::Load { dest: length, ty: IrType::I64, src_ptr: len_ptr });
            let prefix = self.new_reg();
            self.emit(Instruction::ConstInt { dest: prefix, value: elements.len() as i64 });
            let size = self.new_reg();
            self.emit(Instruction::Sub { dest: size, left: length, right: prefix });
            let remainder = self.new_reg();
            self.emit(Instruction::AllocArray { dest: remainder, size, ty: element.clone() });
            let counter = self.new_reg();
            self.emit(Instruction::Alloca {
                dest: counter,
                name: "remainder.index".into(),
                ty: IrType::I64,
            });
            let zero = self.new_reg();
            self.emit(Instruction::ConstInt { dest: zero, value: 0 });
            self.emit(Instruction::Store { ty: IrType::I64, value: zero, ptr: counter });
            let test = self.new_block("remainder.test");
            let copy = self.new_block("remainder.copy");
            let done = self.new_block("remainder.end");
            self.emit(Instruction::Br { target: test });
            self.set_insert_point(test);
            let index = self.new_reg();
            self.emit(Instruction::Load { dest: index, ty: IrType::I64, src_ptr: counter });
            let more = self.new_reg();
            self.emit(Instruction::CmpLt { dest: more, left: index, right: size, ty: IrType::I64 });
            self.emit(Instruction::CondBr { cond: more, if_true: copy, if_false: done });
            self.set_insert_point(copy);
            let source_index = self.new_reg();
            self.emit(Instruction::Add { dest: source_index, left: index, right: prefix });
            let source_ptr = self.new_reg();
            self.emit(Instruction::GetElementPtr {
                dest: source_ptr,
                base_ty: element.clone(),
                base_ptr: value,
                indices: vec![source_index],
            });
            let child = self.new_reg();
            // The remainder belongs to a whole-consuming pattern. Transfer its
            // cells explicitly instead of reconstructing a take from load/zero.
            self.emit(Instruction::Ownership(Physical::ReadValue {
                dest: child, source: source_ptr, ty: element.clone(),
                mode: if consuming || self.is_arc_ir_type(&element) {ReadMode::Move} else {ReadMode::Copy},
            }));
            let dest = self.new_reg();
            self.emit(Instruction::GetElementPtr {
                dest,
                base_ty: element.clone(),
                base_ptr: remainder,
                indices: vec![index],
            });
            self.emit(Instruction::Store { ty: element, value: child, ptr: dest });
            let one = self.new_reg();
            self.emit(Instruction::ConstInt { dest: one, value: 1 });
            let next = self.new_reg();
            self.emit(Instruction::Add { dest: next, left: index, right: one });
            self.emit(Instruction::Store { ty: IrType::I64, value: next, ptr: counter });
            self.emit(Instruction::Br { target: test });
            self.set_insert_point(done);
            self.bind_value(name, remainder, remainder_type);

        }
    }

    fn bind_value(&mut self, name: &Token, value: VReg, ty: IrType) {
        let arc = self.is_arc_ir_type(&ty);

        let slot = self.new_reg();
        self.emit(Instruction::Alloca {
            dest: slot,
            name: name.lexeme.clone(),
            ty: ty.clone(),
        });
        self.emit(Instruction::Store {
            ty: ty.clone(),
            value,
            ptr: slot,
        });
        self.declare_binding(name, slot, ty, arc);
    }

    fn field_address(&mut self, base_ptr: VReg, base_ty: IrType, index: usize) -> VReg {
        let zero = self.new_reg();
        self.emit(Instruction::ConstInt {
            dest: zero,
            value: 0,
        });
        let idx = self.new_reg();
        self.emit(Instruction::ConstInt {
            dest: idx,
            value: index as i64,
        });
        let dest = self.new_reg();
        self.emit(Instruction::GetElementPtr {
            dest,
            base_ty,
            base_ptr,
            indices: vec![zero, idx],
        });
        dest
    }

    fn lower_lvalue(&mut self, expr: &Expr) -> VReg {
        self.lower_place(expr, false)
    }

    // A complete read/modify/write operation owns its base for the operation.
    // Address creation keeps the existing caller-lifetime reference contract.
    fn lower_place(&mut self, expr: &Expr, pin_owner: bool) -> VReg {
        match expr.untyped() {
            Expr::Variable(name) | Expr::This(name) => {
                self.resolve_variable(&name.lexeme).expect("Variable not found!").0
            }
            Expr::Dereference { operand, .. } => {
                let value = self.lower_expr(operand);
                if pin_owner && !self.is_current_block_terminated() {
                    self.hold_expr_value(operand, value);
                }
                value
            }
            Expr::SubscriptGet { indexee, index, .. } => {
                let base_ptr = self.lower_index_base(indexee);
                if self.is_current_block_terminated() {
                    return base_ptr;
                }
                if pin_owner {
                    self.hold_expr_value(indexee, base_ptr);
                }
                let idx_val = self.lower_expr(index);
                if self.is_current_block_terminated() {
                    return idx_val;
                }
                self.inject_null_check(base_ptr, "Null indexed address");
                self.check_index(indexee, base_ptr, idx_val);
                let dest = self.new_reg();

                self.emit(Instruction::GetElementPtr {
                    dest,
                    base_ty: self.index_element_type(indexee),
                    base_ptr,
                    indices: vec![idx_val],
                });

                if !pin_owner && self.infer_is_arc(indexee) && self.has_result_obligation(indexee) {
                    self.emit(Instruction::Ownership(Physical::DropValue{value:base_ptr,recipe:crate::physical_ownership::DropRecipe::Temporary}));
                }
                dest
            }
            Expr::Grouping(e) => self.lower_place(e, pin_owner),
            Expr::Get { object, name } => {
                let base_ptr = self.lower_object(object);
                if self.is_current_block_terminated() {
                    return base_ptr;
                }
                if pin_owner {
                    self.hold_expr_value(object, base_ptr);
                }
                self.inject_null_check(base_ptr, "Null field address");
                let base_ty = self.object_type(object);
                let field = *self.property_indices.get(name).expect("checked field index");
                let zero = self.new_reg();
                self.emit(Instruction::ConstInt { dest: zero, value: 0 });
                let index = self.new_reg();
                self.emit(Instruction::ConstInt { dest: index, value: field as i64 });
                let dest = self.new_reg();
                self.emit(Instruction::GetElementPtr {
                    dest,
                    base_ty,
                    base_ptr,
                    indices: vec![zero, index],
                });
                if !pin_owner && self.infer_is_arc(object) && self.has_result_obligation(object) {
                    self.emit(Instruction::Ownership(Physical::DropValue{value:base_ptr,recipe:crate::physical_ownership::DropRecipe::Temporary}));
                }
                dest
            }
            _ => panic!("Expression is not a valid L-Value!"),
        }
    }

    fn external_call(&mut self, name: &str, args: Vec<VReg>) -> VReg {
        let dest = self.new_reg();
        self.emit(Instruction::Call {
            dest,
            func_name: name.into(),
            args,
        });
        dest
    }

    fn string_constant(&mut self, text: &str) -> VReg {
        let dest = self.new_reg();
        self.emit(Instruction::ConstString {
            dest,
            value: text.into(),
        });
        dest
    }

    fn mark_temporary_string(&mut self, value: VReg) {
        let flag = self.new_reg();
        self.emit(Instruction::ConstBool {
            dest: flag,
            value: true,
        });
        self.string_temporaries.insert(value, flag);
    }

    fn free_temporary_string(&mut self, value: VReg) {
        if let Some(flag) = self.string_temporaries.get(&value).copied() {
            let free = self.new_block("string.temporary.free");
            let done = self.new_block("string.temporary.end");
            self.emit(Instruction::CondBr {
                cond: flag,
                if_true: free,
                if_false: done,
            });
            self.set_insert_point(free);

            self.external_call("free", vec![value]);
            self.emit(Instruction::Br { target: done });
            self.set_insert_point(done);
        }
    }

    fn string_ownership_slot(&mut self, ty: &IrType) -> Option<VReg> {
        if *ty != self.map_type(&Type::String) {
            return None;
        }
        let slot = self.new_reg();
        self.emit(Instruction::Alloca {
            dest: slot,
            name: "string.temporary.owned".into(),
            ty: IrType::Bool,
        });
        Some(slot)
    }

    fn store_string_ownership(&mut self, slot: Option<VReg>, value: VReg) {
        if let Some(slot) = slot {
            let flag = if let Some(flag) = self.string_temporaries.get(&value) {
                *flag
            } else {
                let flag = self.new_reg();
                self.emit(Instruction::ConstBool {
                    dest: flag,
                    value: false,
                });
                flag
            };
            self.emit(Instruction::Store {
                ty: IrType::Bool,
                value: flag,
                ptr: slot,
            });
        }
    }

    fn load_string_ownership(&mut self, slot: Option<VReg>, value: VReg) {
        if let Some(slot) = slot {
            let flag = self.new_reg();
            self.emit(Instruction::Load {
                dest: flag,
                ty: IrType::Bool,
                src_ptr: slot,
            });
            self.string_temporaries.insert(value, flag);
        }
    }

    fn interpolation_part(&mut self, part: &Expr) -> (VReg, bool) {
        let value = self.lower_expr(part);
        let ty = part.checked_type().expect("checked interpolation part");
        if *ty == Type::String {
            return (value, self.string_temporaries.contains_key(&value));
        }
        if *ty == Type::Bool {
            let slot = self.new_reg();
            let ty = self.map_type(&Type::String);
            self.emit(Instruction::Alloca {
                dest: slot,
                name: "interpolation.bool".into(),
                ty: ty.clone(),
            });
            let yes = self.new_block("interpolation.true");
            let no = self.new_block("interpolation.false");
            let end = self.new_block("interpolation.bool.end");
            self.emit(Instruction::CondBr {
                cond: value,
                if_true: yes,
                if_false: no,
            });
            for (block, text) in [
                (yes, "true"),
                (no, "false"),
            ] {
                self.set_insert_point(block);
                let text = self.string_constant(text);
                self.emit(Instruction::Store {
                    ty: ty.clone(),
                    value: text,
                    ptr: slot,
                });
                self.emit(Instruction::Br { target: end });
            }
            self.set_insert_point(end);
            let result = self.new_reg();
            self.emit(Instruction::Load {
                dest: result,
                ty,
                src_ptr: slot,
            });
            return (result, false);
        }
        let format = match ty {
            Type::U8 | Type::U16 | Type::U32 | Type::U64 => "%lu",
            Type::F16 | Type::F32 | Type::F64 => "%f",
            _ => "%ld",
        };
        let format = self.string_constant(format);
        let zero = self.new_reg();
        self.emit(Instruction::ConstInt {
            dest: zero,
            value: 0,
        });
        let size = self.external_call("snprintf", vec![zero, zero, format, value]);
        let valid = self.new_reg();
        self.emit(Instruction::CmpGe {
            dest: valid,
            left: size,
            right: zero,
            ty: IrType::I32,
        });
        let ok = self.new_block("interpolation.format.valid");
        let bad = self.new_block("interpolation.format.failed");
        self.emit(Instruction::CondBr {
            cond: valid,
            if_true: ok,
            if_false: bad,
        });
        self.set_insert_point(bad);
        self.inject_panic("String interpolation formatting failed");
        self.set_insert_point(ok);
        let wide = self.new_reg();
        self.emit(Instruction::Cast {
            dest: wide,
            value: size,
            source_ty: IrType::I32,
            target_ty: IrType::U64,
        });
        let one = self.new_reg();
        self.emit(Instruction::ConstInt {
            dest: one,
            value: 1,
        });
        let capacity = self.new_reg();
        self.emit(Instruction::Add {
            dest: capacity,
            left: wide,
            right: one,
        });
        let buffer = self.external_call("malloc", vec![capacity]);
        self.inject_null_check(buffer, "Out of memory in string interpolation");
        self.external_call("snprintf", vec![buffer, capacity, format, value]);
        self.mark_temporary_string(buffer);
        (buffer, true)
    }

    fn lower_interpolation(&mut self, parts: &[Expr]) -> VReg {
        let mut result = self.string_constant("");
        let mut owned = false;
        for part in parts {
            let (text, temporary) = self.interpolation_part(part);
            let joined = self.external_call(
                &crate::symbols::member("string", "concat"),
                vec![result, text]
            );
            if owned {
                self.free_temporary_string(result);
            }
            if temporary {
                self.free_temporary_string(text);
            }
            result = joined;
            self.mark_temporary_string(result);
            owned = true;
        }
        self.mark_temporary_string(result);
        result
    }

    fn lower_range_loop(
        &mut self,
        key: &Token,
        value: Option<&Token>,
        iterable: &Expr,
        body: &[Stmt]
    ) {
        self.begin_scope();
        let mut bounds = self.lower_expr(iterable);
        let mut source_ty = iterable.checked_type().expect("typed range");
        while let Type::Reference(inner) | Type::MutReference(inner) = source_ty {
            let loaded = self.new_reg();
            let ty = self.map_type(inner);
            self.emit(Instruction::Load { dest: loaded, ty, src_ptr: bounds });
            bounds = loaded;
            source_ty = inner;
        }
        let range_ty = self.map_type(source_ty);
        let IrType::Struct(_, ref fields) = range_ty else { unreachable!("checked range") };
        let element = fields[0].clone();
        let slot = self.new_reg();
        self.emit(Instruction::Alloca {
            dest: slot,
            name: "range.bounds".into(),
            ty: range_ty.clone(),
        });
        self.emit(Instruction::Store {
            ty: range_ty.clone(),
            value: bounds,
            ptr: slot,
        });
        let current_ptr = self.field_address(slot, range_ty.clone(), 0);
        let end_ptr = self.field_address(slot, range_ty, 1);
        let end = self.new_reg();
        self.emit(Instruction::Load {
            dest: end,
            ty: element.clone(),
            src_ptr: end_ptr,
        });
        let counter = self.new_reg();
        self.emit(Instruction::Alloca {
            dest: counter,
            name: "range.index".into(),
            ty: IrType::I64,
        });
        let zero = self.new_reg();
        self.emit(Instruction::ConstInt {
            dest: zero,
            value: 0,
        });
        self.emit(Instruction::Store {
            ty: IrType::I64,
            value: zero,
            ptr: counter,
        });
        let test = self.new_block("range.test");
        let run = self.new_block("range.body");
        let done = self.new_block("range.end");
        self.emit(Instruction::Br { target: test });
        self.set_insert_point(test);
        let current = self.new_reg();
        self.emit(Instruction::Load {
            dest: current,
            ty: element.clone(),
            src_ptr: current_ptr,
        });
        let cond = self.new_reg();
        self.emit(Instruction::CmpLt {
            dest: cond,
            left: current,
            right: end,
            ty: element.clone(),
        });
        self.emit(Instruction::CondBr {
            cond,
            if_true: run,
            if_false: done,
        });
        self.set_insert_point(run);
        self.begin_scope();
        self.bind_value(value.unwrap_or(key), current, element.clone());
        if value.is_some() {
            let index = self.new_reg();
            self.emit(Instruction::Load {
                dest: index,
                ty: IrType::I64,
                src_ptr: counter,
            });
            self.bind_value(key, index, IrType::I64);
        }
        for stmt in body {
            self.lower_stmt(stmt);
        }
        self.end_scope();
        if !self.is_current_block_terminated() {
            // current < end guarantees current + 1 is representable even at MAX.
            let one = self.new_reg();
            self.emit(Instruction::ConstInt {
                dest: one,
                value: 1,
            });
            let next = self.new_reg();
            self.emit(Instruction::Add {
                dest: next,
                left: current,
                right: one,
            });
            self.emit(Instruction::Store {
                ty: element,
                value: next,
                ptr: current_ptr,
            });
            let index = self.new_reg();
            self.emit(Instruction::Load {
                dest: index,
                ty: IrType::I64,
                src_ptr: counter,
            });
            let next_index = self.new_reg();
            self.emit(Instruction::Add {
                dest: next_index,
                left: index,
                right: one,
            });
            self.emit(Instruction::Store {
                ty: IrType::I64,
                value: next_index,
                ptr: counter,
            });
            self.emit(Instruction::Br { target: test });
        }
        self.set_insert_point(done);
        self.end_scope();
    }

    pub fn lower_expr(&mut self, expr: &Expr) -> VReg {
        self.lower_expr_impl(expr)
    }

    fn borrow_address(&mut self, source: VReg, expr: &Expr, mutable: bool) -> VReg {
        let ty=self.infer_ir_type(expr);
        let ty=if matches!(expr.checked_type(),Some(Type::Reference(_)|Type::MutReference(_))){ty}else{IrType::Ptr(Box::new(ty))};
        let dest=self.new_reg();
        self.emit(Instruction::Ownership(Physical::BorrowValue{dest,source,ty,mutable}));
        dest
    }

    fn lower_expr_impl(&mut self, expr: &Expr) -> VReg {

            if
                let Some(id) = expr
                    .identity()
                    .filter(|id| self.ownership.as_ref().unwrap().borrowed_temporaries.contains(id))
            {
                let kind = self.ownership.as_mut().unwrap().borrowed.remove(&id);
                self.ownership.as_mut().unwrap().borrowed_temporaries.remove(&id);
                let owner = if let Expr::AddressOf { operand, .. } = expr.untyped() {
                    operand.as_ref()
                } else {
                    expr
                };
                let value = self.lower_expr(owner);
                if let Some(kind) = kind {
                    self.ownership.as_mut().unwrap().borrowed.insert(id, kind);
                }
                self.ownership.as_mut().unwrap().borrowed_temporaries.insert(id);
                let ty = self.infer_ir_type(owner);
                let slot = self.new_reg();
                self.emit(Instruction::Alloca {
                    dest: slot,
                    name: "borrow.temporary".into(),
                    ty: ty.clone(),
                });
                self.emit(Instruction::Store { ty: ty.clone(), value, ptr: slot });
                let arc = self.is_arc_ir_type(&ty);
                self.declare_variable(format!("#borrow.{}", slot.0), slot, ty, arc);
                return self.borrow_address(slot,expr,false);
            }
            if
                let Some(kind) = expr
                    .identity()
                    .and_then(|id| self.ownership.as_ref().unwrap().borrowed.get(&id))
                    .copied()
            {
                let mutable=kind==crate::ownership_ir::LoanKind::Exclusive;
                if let Expr::AddressOf { operand, .. } = expr.untyped() {
                    let cell=self.lower_lvalue(operand);
                    return self.borrow_address(cell,expr,mutable);
                }
                if let Expr::Variable(name) | Expr::This(name) = expr.untyped() {
                    let (slot, stored, _) = self
                        .resolve_variable(&name.lexeme)
                        .expect("checked borrowed place");
                    if
                        !matches!(
                            expr.checked_type(),
                            Some(Type::Reference(_) | Type::MutReference(_))
                        ) ||
                        stored != self.infer_ir_type(expr)
                    {
                        return self.borrow_address(slot,expr,mutable);
                    }
                } else if
                    !matches!(expr.checked_type(), Some(Type::Reference(_) | Type::MutReference(_)))
                {
                    let cell=self.lower_lvalue(expr);
                    return self.borrow_address(cell,expr,mutable);
                }
            }
            if self.transfers_expr(expr) {
                if let Expr::Variable(name) | Expr::This(name) = expr.untyped() {
                    if let Some((slot, ty, _)) = self.resolve_variable(&name.lexeme) {
                        let value = self.new_reg();
                        self.emit(Instruction::Ownership(Physical::ReadValue {dest:value,source:slot,ty,mode:ReadMode::Move}));
                        return value;
                    }
                    // A named function item produces a fresh callable value;
                    // it has no local owner cell to invalidate.
                }
            }

        if let Expr::Variable(name)|Expr::This(name)=expr.untyped() {
            if let Some((slot,ty,_))=self.resolve_variable(&name.lexeme) {
                let kind=expr.identity().and_then(|id|self.ownership.as_ref().unwrap().values.get(&id));
                let mode=if kind==Some(&crate::ownership::ValueKind::Copy){ReadMode::Copy}else{ReadMode::Observe};
                let dest=self.new_reg();
                self.emit(Instruction::Ownership(Physical::ReadValue{dest,source:slot,ty,mode}));
                return dest;
            }
        }
        if let Expr::Typed { expr: inner, ty, .. } = expr {
            if let (Expr::Binary { left, right, .. }, Type::Range(_)) = (inner.as_ref(), ty) {
                let start = self.lower_expr(left);
                let end = self.lower_expr(right);
                let range_ty = self.map_type(ty);
                let element = self.infer_ir_type(left);
                let slot = self.new_reg();
                self.emit(Instruction::Alloca {
                    dest: slot,
                    name: "range.value".into(),
                    ty: range_ty.clone(),
                });
                for (index, value) in [start, end].into_iter().enumerate() {
                    let ptr = self.field_address(slot, range_ty.clone(), index);
                    self.emit(Instruction::Store { ty: element.clone(), value, ptr });
                }
                let value = self.new_reg();
                self.emit(Instruction::Load { dest: value, ty: range_ty, src_ptr: slot });
                return value;
            }
            if
                let (Expr::Array { elements, .. }, Type::Array(_, element) | Type::Slice(element)) =
                    (inner.as_ref(), ty)
            {
                return self.lower_array(elements, self.map_type(element));
            }
            let value = self.lower_expr_inner(inner, Some(ty));
            if self.is_current_block_terminated() {
                return value;
            }
            let target = self.map_type(ty);
            if
                matches!(
                    target,
                    IrType::U8 |
                        IrType::U16 |
                        IrType::U32 |
                        IrType::U64 |
                        IrType::I8 |
                        IrType::I16 |
                        IrType::I32 |
                        IrType::I64 |
                        IrType::F16 |
                        IrType::F32 |
                        IrType::F64 |
                        IrType::Bool
                )
            {
                let dest = self.new_reg();
                self.emit(Instruction::Cast {
                    dest,
                    value,
                    source_ty: target.clone(),
                    target_ty: target.clone(),
                });
                if
                    matches!(
                        target,
                        IrType::U8 |
                            IrType::U16 |
                            IrType::U32 |
                            IrType::I8 |
                            IrType::I16 |
                            IrType::I32 |
                            IrType::Bool
                    )
                {
                    let extended = self.new_reg();
                    self.emit(Instruction::Cast {
                        dest: extended,
                        value: dest,
                        source_ty: target,
                        target_ty: IrType::I64,
                    });
                    return extended;
                }
                return dest;
            }
            return value;
        }
        self.lower_expr_inner(expr, None)
    }

    fn lower_expr_inner(&mut self, expr: &Expr, checked_type: Option<&Type>) -> VReg {
        // Preserve any nested materialized wrappers, including scalar normalization.
        if matches!(expr, Expr::Typed { .. }) {
            return self.lower_expr(expr);
        }
        match expr.untyped() {
            Expr::Memory { kind, element, arguments, .. } => {
                use crate::ast::MemoryOp;
                if *kind == MemoryOp::ReplaceValue {
                    let cell = self.lower_expr(&arguments[0]);
                    let incoming = self.lower_expr(&arguments[1]);
                    let ty = self.map_type(element);
                    let outgoing = self.new_reg();
                    self.emit(Instruction::Ownership(Physical::SwapValue{dest:outgoing,slot:cell,value:incoming,ty}));
                    return outgoing;
                }
                if *kind == MemoryOp::SharedClone {
                    let cell = self.lower_expr(&arguments[0]);
                    let shared = match arguments[0].checked_type().expect("typed shared receiver") {
                        Type::Reference(t) => self.map_type(t),
                        _ => panic!("shared clone requires a shared receiver"),
                    };
                    let value = self.new_reg();
                    self.emit(Instruction::Load { dest: value, ty: shared, src_ptr: cell });
                    self.emit(Instruction::Ownership(Physical::AcquireSharedEdge{value,kind:EdgeKind::ImmutableBacking}));
                    return value;
                }
                let offset = 1;
                let base = self.lower_expr(&arguments[offset]);
                let index = self.lower_expr(&arguments[offset + 1]);
                let ty = self.map_type(element);
                let slot = self.new_reg();
                self.emit(Instruction::GetElementPtr {
                    dest: slot,
                    base_ty: ty.clone(),
                    base_ptr: base,
                    indices: vec![index],
                });
                let resource=match arguments[0].checked_type().expect("resource capability") {
                    Type::Reference(t)|Type::MutReference(t)=>match t.as_ref(){Type::Custom(n)=>n.clone(),_=>unreachable!("resource owner")},
                    _=>unreachable!("resource capability"),
                };
                let owner=self.lower_expr(&arguments[0]);
                let value=self.new_reg();
                let action=match kind {
                    MemoryOp::RawBorrow=>crate::physical_ownership::ResourceAction::Borrow(value),
                    MemoryOp::RawTake=>crate::physical_ownership::ResourceAction::Take(value),
                    MemoryOp::RawDrop=>crate::physical_ownership::ResourceAction::Drop(value),
                    MemoryOp::RawInitialize=>crate::physical_ownership::ResourceAction::Initialize(self.lower_expr(&arguments[3])),
                    _=>unreachable!(),
                };
                self.emit(Instruction::Ownership(Physical::Resource{resource,owner,slot,ty,action}));
                if *kind==MemoryOp::RawInitialize {self.emit(Instruction::ConstInt{dest:value,value:0});}
                value
            }

            Expr::Interpolation { parts, .. } => self.lower_interpolation(parts),
            Expr::StorageSize(t) => {
                let dest = self.new_reg();
                self.emit(Instruction::SizeOf { dest, ty: self.map_type(t) });
                dest
            }
            Expr::Literal(lit) =>
                match lit {
                    Literal::Number(n) => {
                        let dest = self.new_reg();
                        self.emit(Instruction::ConstFloat {
                            dest,
                            value: *n,
                            ty: IrType::F64,
                        });
                        dest
                    }

                    Literal::Integer(n) => {
                        let dest = self.new_reg();
                        let value = *n as i64;
                        self.emit(Instruction::ConstInt { dest, value });
                        dest
                    }

                    Literal::String(s) => {
                        let dest = self.new_reg();
                        self.emit(Instruction::ConstString {
                            dest,
                            value: s.clone(),
                        });
                        dest
                    }

                    Literal::Bool(b) => {
                        let dest = self.new_reg();
                        self.emit(Instruction::ConstBool { dest, value: *b });
                        dest
                    }

                    Literal::None => {
                        let dest = self.new_reg();
                        self.emit(Instruction::ConstInt { dest, value: 0 });
                        dest
                    }
                }

            Expr::Variable(name) | Expr::This(name) => {
                if self.resolve_variable(&name.lexeme).is_none() {
                    let target = match self.resolved_calls.get(name) {
                        Some(CallType::Static(n)) => n.clone(),
                        _ => name.lexeme.clone(),
                    };
                    let signature = self.function_signatures
                        .get(&target)
                        .expect("checked function signature")
                        .clone();
                    return self.named_function_value(&target, signature.0, signature.1);
                }
                let (ptr_reg, var_ty, _is_arc) = self
                    .resolve_variable(&name.lexeme)
                    .expect("Variable not declared!");

                let dest = self.new_reg();
                let mode=if expr.identity().and_then(|id| self.ownership.as_ref().unwrap().values.get(&id))==Some(&crate::ownership::ValueKind::Copy) {ReadMode::Copy}else{ReadMode::Observe};
                self.emit(Instruction::Ownership(Physical::ReadValue{dest,ty:var_ty,source:ptr_reg,mode}));

                dest
            }

            Expr::Assign { name, value } => {
                let val_reg = self.lower_expr(value);
                let (ptr_reg, var_ty, is_arc) = self
                    .resolve_variable(&name.lexeme)
                    .expect("Variable not declared!");

                if is_arc || self.infer_is_arc(value) {
                    if !self.has_result_obligation(value) {self.emit(Instruction::Ownership(Physical::AcquireSharedEdge{value:val_reg,kind:EdgeKind::Evaluation}));}
                    self.emit(Instruction::Ownership(Physical::DropSlot{slot:ptr_reg,ty:var_ty.clone(),guarded:true}));
                }

                self.emit(Instruction::Store {
                    ty: var_ty,
                    ptr: ptr_reg,
                    value: val_reg,
                });
                self.string_temporaries.remove(&val_reg);
                val_reg
            }

            Expr::Cast { value, target_type, .. } => {
                let source_ty = self.infer_ir_type(&value);
                let val_reg = self.lower_expr(value);
                let dest = self.new_reg();
                let ir_ty = self.map_type(target_type);

                if ir_ty == IrType::FatPtr {
                    if source_ty == IrType::FatPtr {
                        if !self.has_result_obligation(value) {
                            self.emit(Instruction::Ownership(Physical::AcquireSharedEdge{value:val_reg,kind:EdgeKind::Evaluation}));
                        }
                        return val_reg;
                    }
                    let trait_name = match target_type {
                        Type::Custom(n) => n.clone(),
                        _ => "UnknownTrait".to_string(),
                    };

                    let source_struct = match &source_ty {
                        IrType::Ptr(inner) =>
                            match inner.as_ref() {
                                IrType::Struct(n, _) => n.clone(),
                                _ => unreachable!("checked trait cast"),
                            }
                        _ => unreachable!("checked trait cast"),
                    };

                    let vtable_name = crate::symbols::vtable(&source_struct, &trait_name);

                    self.emit(Instruction::MakeFatPtr {
                        dest,
                        data_ptr: val_reg,

                        vtable_name,
                    });

                } else {
                    if self.is_arc_ir_type(&source_ty)&&self.has_result_obligation(value) {
                        self.emit(Instruction::Ownership(Physical::OwnedCast{dest,value:val_reg,source_ty,target_ty:ir_ty.clone()}));
                    }else{
                        self.emit(Instruction::Cast{dest,value:val_reg,source_ty,target_ty:ir_ty.clone()});
                    }
                    if ir_ty == self.map_type(&Type::String) {
                        if let Some(flag) = self.string_temporaries.get(&val_reg).copied() {
                            self.string_temporaries.insert(dest, flag);
                        }
                    }
                }
                dest
            }

            Expr::Unary { operator, right, .. } =>
                match operator.token_type {
                    TokenType::PlusPlus | TokenType::MinusMinus => {
                        self.begin_scope();
                        let ptr = self.lower_place(right, true);
                        if self.is_current_block_terminated() {
                            self.end_scope();
                            return ptr;
                        }
                        let value_ty = self.infer_ir_type(right);
                        let val = self.new_reg();
                        self.emit(Instruction::Load {
                            dest: val,
                            ty: value_ty.clone(),
                            src_ptr: ptr,
                        });

                        let one = self.new_reg();
                        self.emit(Instruction::ConstInt {
                            dest: one,
                            value: 1,
                        });

                        let new_val = self.new_reg();
                        if operator.token_type == TokenType::PlusPlus {
                            self.emit(Instruction::Add {
                                dest: new_val,
                                left: val,
                                right: one,
                            });
                        } else {
                            self.emit(Instruction::Sub {
                                dest: new_val,
                                left: val,
                                right: one,
                            });
                        }

                        self.emit(Instruction::Store {
                            ty: value_ty.clone(),
                            ptr: ptr,
                            value: new_val,
                        });

                        self.end_scope();
                        new_val
                    }

                    TokenType::Minus => {
                        if let Expr::Literal(Literal::Integer(n)) = right.untyped() {
                            if let Ok(value) = i64::try_from(-*n) {
                                let dest = self.new_reg();
                                self.emit(Instruction::ConstInt { dest, value });
                                return dest;
                            }
                        }

                        let val = self.lower_expr(right);
                        let zero = self.new_reg();
                        let ty = self.infer_ir_type(right);
                        if matches!(ty, IrType::F16 | IrType::F32 | IrType::F64) {
                            self.emit(Instruction::ConstFloat { dest: zero, value: -0.0, ty });
                        } else {
                            self.emit(Instruction::ConstInt { dest: zero, value: 0 });
                        }

                        let new_val = self.new_reg();
                        self.emit(Instruction::Sub {
                            dest: new_val,
                            left: zero,
                            right: val,
                        });

                        new_val
                    }

                    TokenType::Bang => {
                        let val = self.lower_expr(right);
                        let zero = self.new_reg();
                        self.emit(Instruction::ConstInt {
                            dest: zero,
                            value: 0,
                        });

                        let new_val = self.new_reg();
                        self.emit(Instruction::CmpEq {
                            dest: new_val,
                            left: val,
                            right: zero,
                        });

                        new_val
                    }
                    _ => unimplemented!("Unary operator not supported in IR yet."),
                }

            Expr::Binary { left, operator, right, .. } => {
                if
                    matches!(operator.token_type, TokenType::EqualEqual | TokenType::BangEqual) &&
                    (self.infer_is_arc(left) || self.infer_is_arc(right))
                {
                    self.begin_scope();
                    let left_reg = self.lower_expr(left);
                    if self.is_current_block_terminated() {
                        self.end_scope();
                        return left_reg;
                    }
                    self.hold_expr_value(left, left_reg);
                    let right_reg = self.lower_expr(right);
                    if self.is_current_block_terminated() {
                        self.end_scope();
                        return right_reg;
                    }
                    self.hold_expr_value(right, right_reg);
                    let (left_reg, right_reg) = if
                        left.checked_type() == Some(&Type::Custom("ThreadId".into())) &&
                        right.checked_type() == Some(&Type::Custom("ThreadId".into()))
                    {
                        let l = self.new_reg();
                        let r = self.new_reg();
                        self.emit(Instruction::Call {
                            dest: l,
                            func_name: crate::symbols::member("ThreadId", "hash"),
                            args: vec![left_reg],
                        });
                        self.emit(Instruction::Call {
                            dest: r,
                            func_name: crate::symbols::member("ThreadId", "hash"),
                            args: vec![right_reg],
                        });
                        (l, r)
                    } else {
                        (left_reg, right_reg)
                    };
                    let dest = self.new_reg();
                    self.emit(
                        if operator.token_type == TokenType::EqualEqual {
                            Instruction::CmpEq { dest, left: left_reg, right: right_reg }
                        } else {
                            Instruction::CmpNeq { dest, left: left_reg, right: right_reg }
                        }
                    );
                    self.end_scope();
                    return dest;
                }
                let left_ty = self.infer_ir_type(left);
                let left_reg = self.lower_expr(left);
                if operator.token_type == TokenType::QuestionQuestion {
                    let slot = self.new_reg();
                    self.emit(Instruction::Alloca {
                        dest: slot,
                        name: "coalesce".into(),
                        ty: left_ty.clone(),
                    });
                    self.emit(Instruction::Store {
                        ty: left_ty.clone(),
                        value: left_reg,
                        ptr: slot,
                    });
                    let zero = self.new_reg();
                    self.emit(Instruction::ConstInt { dest: zero, value: 0 });
                    let cond = self.new_reg();
                    self.emit(Instruction::CmpNeq { dest: cond, left: left_reg, right: zero });
                    let rhs = self.new_block("coalesce.rhs");
                    let end = self.new_block("coalesce.end");
                    self.emit(Instruction::CondBr { cond, if_true: end, if_false: rhs });
                    self.set_insert_point(rhs);
                    let value = self.lower_expr(right);
                    self.emit(Instruction::Store { ty: left_ty.clone(), value, ptr: slot });
                    self.emit(Instruction::Br { target: end });
                    self.set_insert_point(end);
                    let dest = self.new_reg();
                    self.emit(Instruction::Load { dest, ty: left_ty, src_ptr: slot });
                    return dest;
                }
                let right_reg = self.lower_expr(right);
                if
                    matches!(operator.token_type, TokenType::Slash | TokenType::Percent) &&
                    !matches!(left_ty, IrType::F16 | IrType::F32 | IrType::F64)
                {
                    let zero = self.new_reg();
                    self.emit(Instruction::ConstInt { dest: zero, value: 0 });
                    let valid = self.new_reg();
                    self.emit(Instruction::CmpNeq { dest: valid, left: right_reg, right: zero });
                    let ok = self.new_block("division.valid");
                    let bad = self.new_block("division.zero");
                    self.emit(Instruction::CondBr { cond: valid, if_true: ok, if_false: bad });
                    self.set_insert_point(bad);
                    self.inject_panic("Integer division or remainder by zero");
                    self.set_insert_point(ok);
                    if left_ty == IrType::I64 {
                        let min = self.new_reg();
                        self.emit(Instruction::ConstInt { dest: min, value: i64::MIN });
                        let neg = self.new_reg();
                        self.emit(Instruction::ConstInt { dest: neg, value: -1 });
                        let is_min = self.new_reg();
                        self.emit(Instruction::CmpEq { dest: is_min, left: left_reg, right: min });
                        let is_neg = self.new_reg();
                        self.emit(Instruction::CmpEq { dest: is_neg, left: right_reg, right: neg });
                        let overflow = self.new_reg();
                        self.emit(Instruction::BitAnd {
                            dest: overflow,
                            left: is_min,
                            right: is_neg,
                        });
                        let safe = self.new_block("division.safe");
                        let fail = self.new_block("division.overflow");
                        self.emit(Instruction::CondBr {
                            cond: overflow,
                            if_true: fail,
                            if_false: safe,
                        });
                        self.set_insert_point(fail);
                        self.inject_panic("Signed division overflow");
                        self.set_insert_point(safe);
                    }
                }
                if operator.token_type == TokenType::Exponentiation {
                    return self.lower_power(left_reg, right_reg, left_ty);
                }
                if
                    operator.token_type == TokenType::Plus &&
                    matches!(left.checked_type(), Some(Type::String))
                {
                    let dest = self.external_call(
                        &crate::symbols::member("string", "concat"),
                        vec![left_reg, right_reg]
                    );
                    self.free_temporary_string(left_reg);
                    self.free_temporary_string(right_reg);
                    self.mark_temporary_string(dest);
                    return dest;
                }
                let dest = self.new_reg();

                let inst = match operator.token_type {
                    TokenType::Plus =>
                        Instruction::Add {
                            dest,
                            left: left_reg,
                            right: right_reg,
                        },
                    TokenType::Minus =>
                        Instruction::Sub {
                            dest,
                            left: left_reg,
                            right: right_reg,
                        },
                    TokenType::Star =>
                        Instruction::Mul {
                            dest,
                            left: left_reg,
                            right: right_reg,
                        },
                    TokenType::BitwiseAnd =>
                        Instruction::BitAnd { dest, left: left_reg, right: right_reg },
                    TokenType::BitwiseOr =>
                        Instruction::BitOr { dest, left: left_reg, right: right_reg },
                    TokenType::Slash =>
                        Instruction::Div {
                            dest,
                            left: left_reg,
                            right: right_reg,
                            ty: left_ty.clone(),
                        },
                    TokenType::Percent =>
                        Instruction::Mod {
                            dest,
                            left: left_reg,
                            right: right_reg,
                            ty: left_ty.clone(),
                        },
                    TokenType::EqualEqual =>
                        Instruction::CmpEq {
                            dest,
                            left: left_reg,
                            right: right_reg,
                        },
                    TokenType::BangEqual =>
                        Instruction::CmpNeq {
                            dest,
                            left: left_reg,
                            right: right_reg,
                        },
                    TokenType::Less =>
                        Instruction::CmpLt {
                            dest,
                            left: left_reg,
                            right: right_reg,
                            ty: left_ty.clone(),
                        },
                    TokenType::LessEqual =>
                        Instruction::CmpLe {
                            dest,
                            left: left_reg,
                            right: right_reg,
                            ty: left_ty.clone(),
                        },
                    TokenType::Greater =>
                        Instruction::CmpGt {
                            dest,
                            left: left_reg,
                            right: right_reg,
                            ty: left_ty.clone(),
                        },
                    TokenType::GreaterEqual =>
                        Instruction::CmpGe {
                            dest,
                            left: left_reg,
                            right: right_reg,
                            ty: left_ty.clone(),
                        },
                    _ => panic!("Binary operator not supported in IR yet."),
                };

                self.emit(inst);
                dest
            }

            Expr::Grouping(inner) => self.lower_expr(inner),

            Expr::Call { callee, arguments, paren } => {
                // Grouping preserves a direct call's resolved specialization.
                let mut callee = callee.as_ref();
                while let Expr::Grouping(inner) = callee.untyped() {
                    callee = inner;
                }
                self.begin_scope();
                let mut arg_regs = Vec::new();
                for arg in arguments {
                    let value = self.lower_expr(arg);
                    if self.is_current_block_terminated() {
                        self.end_scope();
                        return value;
                    }
                    self.hold_expr_value(arg, value);
                    arg_regs.push(value);
                }

                let mut default_receiver = None;
                let mut receiver_temporary = None;
                if let Some(defaults) = self.default_calls.get(paren).cloned() {
                    let mut values = arg_regs.clone();
                    if defaults.instance {
                        if let Expr::Get { object, .. } = callee.untyped() {
                            let receiver = self.lower_receiver(object);
                            default_receiver = Some(receiver);
                            values.insert(0, receiver);
                            if self.is_arc_ir_type(&self.map_type(&defaults.parameters[0].1)) {
                                if !(self.infer_is_arc(object) && self.has_result_obligation(object)) {
                                    self.emit(Instruction::Ownership(Physical::AcquireSharedEdge{value:receiver,kind:EdgeKind::Evaluation}));
                                }
                                self.hold_owned_value(
                                    receiver,
                                    self.map_type(&defaults.parameters[0].1)
                                );
                            }
                        }
                    }
                    let scopes = std::mem::replace(
                        &mut self.scopes,
                        vec![self.globals.clone(), HashMap::new()]
                    );
                    let binding_slots = self.binding_slots.clone();
                    for (index, (name, ty)) in defaults.parameters.iter().enumerate() {
                        let ty = self.map_type(ty);
                        let value = if index < values.len() {
                            values[index]
                        } else {
                            let e = &defaults.defaults[index - values.len()];
                            let value = self.lower_expr(e);
                            if self.is_current_block_terminated() {
                                self.scopes = scopes;
                                self.binding_slots = binding_slots;
                                self.end_scope();
                                return value;
                            }
                            self.hold_expr_value(e, value);
                            arg_regs.push(value);
                            value
                        };
                        let slot = self.new_reg();
                        self.emit(Instruction::Alloca {
                            dest: slot,
                            name: format!("default.{}", name.lexeme),
                            ty: ty.clone(),
                        });
                        self.emit(Instruction::Store { ty: ty.clone(), value, ptr: slot });
                        self.declare_binding(name, slot, ty, false);
                    }
                    let pins = self.scopes
                        .last()
                        .unwrap()
                        .iter()
                        .filter(|(_, (slot, _, _))| self.temporary_values.contains_key(slot))
                        .map(|(n, v)| (n.clone(), v.clone()))
                        .collect::<Vec<_>>();
                    self.scopes = scopes;
                    self.binding_slots = binding_slots;
                    self.scopes.last_mut().unwrap().extend(pins);
                }
                let result = (|| {
                    let dest = self.new_reg();

                    if let Expr::Get { object, name } = callee.untyped() {
                        if
                            let Some((function, instance)) = self.specialized_methods
                                .get(name)
                                .cloned()
                        {
                            let mut args = arg_regs;

                            if instance || object.checked_type().is_some() {
                                let receiver = default_receiver.unwrap_or_else(||
                                    self.lower_receiver(object)
                                );
                                if
                                    default_receiver.is_none() &&
                                    self.infer_is_arc(object) &&
                                    self.has_result_obligation(object) &&
                                    !self.transfers_expr(object) &&
                                    !self.is_borrowed_expr(object)
                                {
                                    receiver_temporary = Some((
                                        receiver,
                                        self.infer_ir_type(object),
                                    ));
                                }
                                if instance {
                                    args.insert(0, receiver);
                                }
                            }
                            self.emit_source_call(Instruction::Call {
                                dest,
                                func_name: function,
                                args,
                            });
                            return dest;
                        }
                    }
                    if
                        matches!(callee.untyped(),Expr::Get {name,..} if self.property_indices.contains_key(name))
                    {
                        let closure_ptr = self.lower_expr(callee);
                        if self.has_result_obligation(callee) {
                            receiver_temporary = Some((closure_ptr, self.infer_ir_type(callee)));
                        }
                        self.inject_null_check(closure_ptr, "Null closure field call");
                        if let IrType::Closure(arg_types, ret_type) = self.infer_ir_type(callee) {
                            self.emit_source_call(Instruction::CallClosure {
                                dest,
                                closure_ptr,
                                args: arg_regs,
                                arg_types,
                                ret_type: *ret_type,
                            });
                            return dest;
                        }
                    }
                    match callee.untyped() {
                        Expr::Variable(t) => {
                            let func_name = t.lexeme.clone();

                            if let Some((ptr_reg, var_ty, _)) = self.resolve_variable(&func_name) {
                                let closure_ptr = self.new_reg();
                                self.emit(Instruction::Load {
                                    dest: closure_ptr,
                                    ty: var_ty,
                                    src_ptr: ptr_reg,
                                });
                                self.inject_null_check(
                                    closure_ptr,
                                    "Null pointer dereference on Closure Call!"
                                );
                                let (arg_types, ret_type) = match self.infer_ir_type(callee) {
                                    IrType::Closure(ps, r) => (ps, *r),
                                    _ => panic!("checked callee is not a closure"),
                                };
                                self.emit_source_call(Instruction::CallClosure {
                                    dest,
                                    closure_ptr,
                                    args: arg_regs,
                                    arg_types,
                                    ret_type,
                                });
                            } else {
                                let final_func_name = match self.resolved_calls.get(t) {
                                    Some(CallType::Static(mangled)) => mangled.clone(),
                                    _ => func_name.clone(),
                                };

                                if
                                    let Some(info) = self.user_types
                                        .get(&final_func_name)
                                        .cloned()
                                        .filter(
                                            |i|
                                                i.case_names.is_empty() &&
                                                !self.traits.contains(&final_func_name)
                                        )
                                {
                                    self.transfer_call_arguments();
                                    self.emit(Instruction::AllocStruct {
                                        dest,
                                        class_name: final_func_name.clone(),
                                    });
                                    let base = IrType::Struct(
                                        final_func_name.clone(),
                                        self.get_struct_field_types(&final_func_name)
                                    );
                                    let mut fields = info.fields.values().collect::<Vec<_>>();
                                    fields.sort_by_key(|(i, _, _)| i);
                                    for (i, ((_, ty, _), value)) in fields
                                        .into_iter()
                                        .zip(&arg_regs)
                                        .enumerate() {
                                        let ty = self.map_type(ty);

                                        let ptr = self.field_address(dest, base.clone(), i);
                                        self.emit(Instruction::Store { ty, value: *value, ptr });
                                    }
                                } else {
                                    self.emit_source_call(Instruction::Call {
                                        dest,
                                        func_name: final_func_name.clone(),
                                        args: arg_regs,
                                    });
                                }


                            }
                        }

                        Expr::Get { object, name } => {
                            let mut is_static = false;
                            if let Expr::Variable(var_name) = object.untyped() {
                                if self.resolve_variable(&var_name.lexeme).is_none() {
                                    is_static = true;
                                }
                            }

                            if is_static {
                                if let Expr::Variable(var_name) = object.untyped() {
                                    let class_name = match self.resolved_calls.get(name) {
                                        Some(CallType::Static(c)) | Some(CallType::Instance(c)) => {
                                            c.clone()
                                        }
                                        None => var_name.lexeme.clone(),
                                    };
                                    let mangled_func_name = crate::symbols::member(
                                        &class_name,
                                        &name.lexeme
                                    );

                                    self.emit_source_call(Instruction::Call {
                                        dest,
                                        func_name: mangled_func_name,
                                        args: arg_regs,
                                    });
                                }
                            } else {
                                let obj_ir_ty = self.object_value_type(object);
                                let obj_reg = default_receiver.unwrap_or_else(||
                                    self.lower_receiver(object)
                                );
                                if
                                    default_receiver.is_none() &&
                                    self.infer_is_arc(object) &&
                                    self.has_result_obligation(object) &&
                                    !self.transfers_expr(object) &&
                                    !self.is_borrowed_expr(object)
                                {
                                    receiver_temporary = Some((
                                        obj_reg,
                                        self.infer_ir_type(object),
                                    ));
                                }

                                // Scalar receivers (including false) are values, not addresses.
                                if
                                    matches!(
                                        obj_ir_ty,
                                        IrType::Ptr(_) |
                                            IrType::Array(_, _) |
                                            IrType::FatPtr |
                                            IrType::Closure(_, _)
                                    )
                                {
                                    self.inject_null_check(
                                        obj_reg,
                                        "Null pointer dereference on Method Call!"
                                    );
                                }

                                let class_name_opt = match self.resolved_calls.get(name) {
                                    Some(CallType::Static(c)) | Some(CallType::Instance(c)) => {
                                        Some(c.clone())
                                    }
                                    None => None,
                                };

                                if let Some(class_name) = class_name_opt {
                                    let instance = matches!(
                                        self.resolved_calls.get(name),
                                        Some(CallType::Instance(_))
                                    );
                                    let mut method_args = if instance {
                                        vec![obj_reg]
                                    } else {
                                        vec![]
                                    };
                                    method_args.extend(arg_regs.clone());

                                    if self.traits.contains(&class_name) {
                                        let vtable_index = *self.trait_vtable_layout
                                            .get(&class_name)
                                            .and_then(|layout| layout.get(&name.lexeme))
                                            .expect("ERROR: Method not found in trait layout!");

                                        let (arg_types, ret_type) = if
                                            let Some(info) = self.user_types.get(&class_name)
                                        {
                                            if
                                                let Some((params, ret_t, _)) = info.methods.get(
                                                    &name.lexeme
                                                )
                                            {
                                                let arg_types: Vec<IrType> = params
                                                    .iter()
                                                    .skip(1)
                                                    .map(|t| self.map_type(t))
                                                    .collect();
                                                let ret_type = self.map_type(ret_t);
                                                (arg_types, ret_type)
                                            } else {
                                                (
                                                    arguments
                                                        .iter()
                                                        .map(|arg| self.infer_ir_type(arg))
                                                        .collect(),
                                                    IrType::I64,
                                                )
                                            }
                                        } else {
                                            (
                                                arguments
                                                    .iter()
                                                    .map(|arg| self.infer_ir_type(arg))
                                                    .collect(),
                                                IrType::I64,
                                            )
                                        };

                                        let mode = self.receiver_mode(&class_name, &name.lexeme);
                                        self.emit_source_call(Instruction::DynamicCall {
                                            receiver_mode: mode,
                                            dest,
                                            vtable_index,
                                            fat_ptr: obj_reg,
                                            args: arg_regs,
                                            arg_types,
                                            ret_type,
                                        });
                                        if mode == crate::ir::ReceiverMode::Consume {
                                            self.emit(Instruction::Ownership(Physical::DropValue{value:obj_reg,recipe:crate::physical_ownership::DropRecipe::Temporary}));
                                        }
                                    } else {
                                        let mangled_func_name = crate::symbols::member(
                                            &class_name,
                                            &name.lexeme
                                        );
                                        self.emit_source_call(Instruction::Call {
                                            dest,
                                            func_name: mangled_func_name,
                                            args: method_args,
                                        });
                                    }
                                } else {
                                    unimplemented!("Get path not supported in Call instruction.");
                                }
                            }
                        }

                        _ => {
                            let closure_ptr = self.lower_expr(callee);
                            if self.infer_is_arc(callee) && self.has_result_obligation(callee) {
                                receiver_temporary = Some((
                                    closure_ptr,
                                    self.infer_ir_type(callee),
                                ));
                            }
                            self.inject_null_check(closure_ptr, "Null closure call");
                            let (arg_types, ret_type) = match self.infer_ir_type(callee) {
                                IrType::Closure(ps, r) => (ps, *r),
                                _ => panic!("checked callee is not a closure"),
                            };
                            self.emit_source_call(Instruction::CallClosure {
                                dest,
                                closure_ptr,
                                args: arg_regs,
                                arg_types,
                                ret_type,
                            });
                        }
                    }
                    dest
                })();
                if let Some((receiver, ty)) = receiver_temporary {
                    self.hold_owned_value(receiver, ty);
                }
                self.end_scope();
                if
                    matches!(callee.untyped(),Expr::Variable(n) if matches!(n.lexeme.as_str(),"panic"|"exit")&&self.resolve_variable(&n.lexeme).is_none())
                {
                    self.emit(Instruction::Unreachable);
                }
                result
            }

            Expr::Get { object, name } => {
                if !self.property_indices.contains_key(name) {
                    return self.method_value(object, name);
                }
                let obj_ptr = self.lower_object(object);

                self.inject_null_check(obj_ptr, "Null pointer dereference on Property Read!");

                let field_index = self.property_indices
                    .get(name)
                    .copied()
                    .expect("checked property index");

                let idx_reg = self.new_reg();
                self.emit(Instruction::ConstInt {
                    dest: idx_reg,
                    value: field_index as i64,
                });

                let field_base_ty = self.object_type(object);
                let field_ptr = self.new_reg();
                let gep_indices = if matches!(field_base_ty, IrType::Struct(_, _)) {
                    let idx0_reg = self.new_reg();
                    self.emit(Instruction::ConstInt {
                        dest: idx0_reg,
                        value: 0,
                    });
                    vec![idx0_reg, idx_reg]
                } else {
                    vec![idx_reg]
                };
                self.emit(Instruction::GetElementPtr {
                    dest: field_ptr,
                    base_ty: field_base_ty,
                    base_ptr: obj_ptr,
                    indices: gep_indices,
                });

                let dest = self.new_reg();
                let field_ty = self.get_property_ir_type(object, &name.lexeme);
                self.emit(Instruction::Load {
                    dest,
                    ty: field_ty.clone(),
                    src_ptr: field_ptr,
                });
                if self.infer_is_arc(object) && self.has_result_obligation(object) {
                    if self.is_arc_ir_type(&field_ty) {
                        self.emit(Instruction::Ownership(Physical::AcquireSharedEdge{value:dest,kind:EdgeKind::Evaluation}));
                    }
                    self.emit(Instruction::Ownership(Physical::DropValue{value:obj_ptr,recipe:crate::physical_ownership::DropRecipe::Temporary}));
                }
                dest
            }

            Expr::Set { object, name, value } => {
                self.begin_scope();
                let obj_ptr = self.lower_object(object);
                if self.is_current_block_terminated() {
                    self.end_scope();
                    return obj_ptr;
                }
                let pinned = self.hold_expr_value(object, obj_ptr);
                self.inject_null_check(obj_ptr, "Null pointer dereference on Property Write!");

                let val_reg = self.lower_expr(value);
                if self.is_current_block_terminated() {
                    self.end_scope();
                    return val_reg;
                }

                let field_index = self.property_indices
                    .get(name)
                    .copied()
                    .expect("checked property index");

                let idx_reg = self.new_reg();
                self.emit(Instruction::ConstInt {
                    dest: idx_reg,
                    value: field_index as i64,
                });

                let field_base_ty = self.object_type(object);
                let field_ptr = self.new_reg();
                let gep_indices = if matches!(field_base_ty, IrType::Struct(_, _)) {
                    let idx0_reg = self.new_reg();
                    self.emit(Instruction::ConstInt {
                        dest: idx0_reg,
                        value: 0,
                    });
                    vec![idx0_reg, idx_reg]
                } else {
                    vec![idx_reg]
                };
                self.emit(Instruction::GetElementPtr {
                    dest: field_ptr,
                    base_ty: field_base_ty,
                    base_ptr: obj_ptr,
                    indices: gep_indices,
                });

                let field_ty = self.get_property_ir_type(object, &name.lexeme);
                self.replace_owned_slot(field_ptr, val_reg, value, &field_ty);
                self.emit(Instruction::Store {
                    ty: field_ty.clone(),
                    ptr: field_ptr,
                    value: val_reg,
                });
                if pinned && self.is_arc_ir_type(&field_ty) {
                    self.emit(Instruction::Ownership(Physical::AcquireSharedEdge{value:val_reg,kind:EdgeKind::Evaluation}));
                }
                self.end_scope();
                self.string_temporaries.remove(&val_reg);
                val_reg
            }

            Expr::Logical { left, operator, right } => {
                let result_ptr = self.new_reg();
                self.emit(Instruction::Alloca {
                    dest: result_ptr,
                    name: "tmp_logic".into(),
                    ty: IrType::Bool,
                });
                let left_reg = self.lower_expr(left);
                let right_block = self.new_block("logic.rhs");
                let end_block = self.new_block("logic.end");

                if operator.token_type == TokenType::LogicalAnd {
                    self.emit(Instruction::Store {
                        ty: IrType::Bool,
                        ptr: result_ptr,
                        value: left_reg,
                    });
                    self.emit(Instruction::CondBr {
                        cond: left_reg,
                        if_true: right_block,
                        if_false: end_block,
                    });
                } else {
                    self.emit(Instruction::Store {
                        ty: IrType::Bool,
                        ptr: result_ptr,
                        value: left_reg,
                    });
                    self.emit(Instruction::CondBr {
                        cond: left_reg,
                        if_true: end_block,
                        if_false: right_block,
                    });
                }

                self.set_insert_point(right_block);
                let right_reg = self.lower_expr(right);
                self.emit(Instruction::Store {
                    ty: IrType::Bool,
                    ptr: result_ptr,
                    value: right_reg,
                });
                self.emit(Instruction::Br { target: end_block });
                self.set_insert_point(end_block);

                let final_val = self.new_reg();
                self.emit(Instruction::Load {
                    ty: IrType::Bool,
                    dest: final_val,
                    src_ptr: result_ptr,
                });
                final_val
            }

            Expr::TupleLiteral { elements, .. } => {
                if elements.is_empty() {
                    let unit = self.new_reg();
                    self.emit(Instruction::ConstInt { dest: unit, value: 0 });
                    return unit;
                }
                let types = elements
                    .iter()
                    .map(|e| self.infer_ir_type(e))
                    .collect::<Vec<_>>();
                // The checked type supplies the stable nominal tuple name through its fields.
                let source = Type::Tuple(
                    elements
                        .iter()
                        .map(|e| e.checked_type().cloned().expect("checked tuple element"))
                        .collect()
                );
                let name = format!("__tuple_{}", source);
                self.synthetic_layouts.insert(name.clone(), types.clone());
                let dest = self.new_reg();
                self.emit(Instruction::AllocStruct { dest, class_name: name.clone() });
                let tuple_ty = IrType::Ptr(Box::new(IrType::Struct(name.clone(), types.clone())));
                let construction = self.begin_construction(dest, tuple_ty.clone());

                for (i, (e, ty)) in elements.iter().zip(&types).enumerate() {
                    let value = self.lower_expr(e);
                    if self.is_current_block_terminated() {
                        self.end_scope();
                        return dest;
                    }
                    if self.infer_is_arc(e) && !self.has_result_obligation(e) {
                        self.emit(Instruction::Ownership(Physical::AcquireSharedEdge{value:value,kind:EdgeKind::Evaluation}));
                    }
                    let ptr = self.field_address(
                        dest,
                        IrType::Struct(name.clone(), types.clone()),
                        i
                    );
                    self.emit(Instruction::Store { ty: ty.clone(), value, ptr });
                }

                self.finish_construction(construction, tuple_ty);
                dest
            }
            Expr::Ternary { condition, then_branch, else_branch, .. } => {
                let ty = checked_type
                    .map(|ty| self.map_type(ty))
                    .unwrap_or_else(|| self.infer_ir_type(then_branch));
                let slot = self.new_reg();
                if ty != IrType::Void {
                    self.emit(Instruction::Alloca {
                        dest: slot,
                        name: "ternary.result".into(),
                        ty: ty.clone(),
                    });
                    if self.is_arc_ir_type(&ty) {self.register_owner_slot(slot,ty.clone(),SlotKind::Temporary);}
                }
                let ownership = self.string_ownership_slot(&ty);
                let cond = self.lower_expr(condition);
                let yes = self.new_block("ternary.then");
                let no = self.new_block("ternary.else");
                let end = self.new_block("ternary.end");
                self.emit(Instruction::CondBr { cond, if_true: yes, if_false: no });
                for (block, expr) in [
                    (yes, then_branch),
                    (no, else_branch),
                ] {
                    self.set_insert_point(block);
                    let value = self.lower_expr(expr);
                    if self.is_current_block_terminated() {
                        continue;
                    }
                    if self.infer_is_arc(expr) && !self.has_result_obligation(expr) {
                        self.emit(Instruction::Ownership(Physical::AcquireSharedEdge{value:value,kind:EdgeKind::Evaluation}));
                    }
                    if ty != IrType::Void {
                        self.emit(Instruction::Store { ty: ty.clone(), value, ptr: slot });
                    }
                    self.store_string_ownership(ownership, value);
                    self.emit(Instruction::Br { target: end });
                }
                self.set_insert_point(end);
                let dest = self.new_reg();
                if ty != IrType::Void {
                    if self.is_arc_ir_type(&ty){self.emit(Instruction::Ownership(Physical::ReadValue{dest,source:slot,ty,mode:ReadMode::Move}));}
                    else{self.emit(Instruction::Load { dest, ty, src_ptr: slot });}
                } else {
                    self.emit(Instruction::ConstInt { dest, value: 0 });
                }
                self.load_string_ownership(ownership, dest);
                dest
            }

            Expr::Array { elements, .. } =>
                self.lower_array(elements, self.infer_array_element_type(elements)),

            Expr::SubscriptGet { indexee, bracket: _, index } => {
                let element_ty = self.index_element_type(indexee);
                self.begin_scope();
                let base_ptr = self.lower_index_base(indexee);
                if self.is_current_block_terminated() {
                    self.end_scope();
                    return base_ptr;
                }
                let pinned = self.hold_expr_value(indexee, base_ptr);
                self.inject_null_check(base_ptr, "Null pointer dereference (Array Read)!");

                let idx_val = self.lower_expr(index);
                if self.is_current_block_terminated() {
                    self.end_scope();
                    return idx_val;
                }

                self.check_index(indexee, base_ptr, idx_val);

                let dest_ptr = self.new_reg();
                self.emit(Instruction::GetElementPtr {
                    dest: dest_ptr,
                    base_ty: element_ty.clone(),
                    base_ptr,
                    indices: vec![idx_val],
                });

                let result = self.new_reg();
                self.emit(Instruction::Load {
                    dest: result,
                    ty: element_ty.clone(),
                    src_ptr: dest_ptr,
                });
                if pinned && self.is_arc_ir_type(&element_ty) {
                    self.emit(Instruction::Ownership(Physical::AcquireSharedEdge{value:result,kind:EdgeKind::Evaluation}));
                }
                self.end_scope();
                result
            }

            Expr::SubscriptSet { indexee, bracket: _, index, value } => {
                let element_ty = self.index_element_type(indexee);
                self.begin_scope();
                let base_ptr = self.lower_index_base(indexee);
                if self.is_current_block_terminated() {
                    self.end_scope();
                    return base_ptr;
                }
                let pinned = self.hold_expr_value(indexee, base_ptr);
                self.inject_null_check(base_ptr, "Null pointer dereference (Array Write)!");

                let idx_val = self.lower_expr(index);
                if self.is_current_block_terminated() {
                    self.end_scope();
                    return idx_val;
                }
                self.check_index(indexee, base_ptr, idx_val);

                let val_reg = self.lower_expr(value);
                if self.is_current_block_terminated() {
                    self.end_scope();
                    return val_reg;
                }

                let dest_ptr = self.new_reg();
                self.emit(Instruction::GetElementPtr {
                    dest: dest_ptr,
                    base_ty: element_ty.clone(),
                    base_ptr,
                    indices: vec![idx_val],
                });

                self.replace_owned_slot(dest_ptr, val_reg, value, &element_ty);
                self.emit(Instruction::Store {
                    ty: element_ty.clone(),
                    ptr: dest_ptr,
                    value: val_reg,
                });
                if pinned && self.is_arc_ir_type(&element_ty) {
                    self.emit(Instruction::Ownership(Physical::AcquireSharedEdge{value:val_reg,kind:EdgeKind::Evaluation}));
                }
                self.end_scope();
                self.string_temporaries.remove(&val_reg);
                val_reg
            }

            Expr::AddressOf { operand, .. } => self.lower_lvalue(operand),

            Expr::Dereference { operator: _, operand } => {
                let ptr_val = self.lower_expr(operand);

                self.inject_null_check(ptr_val, "Null pointer dereference!");


                let load_ty = self.deref_type(&self.infer_ir_type(operand));
                let dest = self.new_reg();
                self.emit(Instruction::Load {
                    dest,
                    ty: load_ty,
                    src_ptr: ptr_val,
                });
                dest
            }

            Expr::DereferenceSet { ptr, value, .. } => {
                let target_address = self.lower_expr(ptr);

                self.inject_null_check(target_address, "Null pointer dereference on Assignment!");


                let val_reg = self.lower_expr(value);
                let store_ty = self.deref_type(&self.infer_ir_type(ptr));
                self.replace_owned_slot(target_address, val_reg, value, &store_ty);
                self.emit(Instruction::Store {
                    ty: store_ty,
                    value: val_reg,
                    ptr: target_address,
                });
                self.string_temporaries.remove(&val_reg);
                val_reg
            }

            Expr::StructInit { class_name, properties, .. } => {
                let obj_reg = self.new_reg();

                let real_name = match self.resolved_calls.get(class_name) {
                    Some(CallType::Static(c)) | Some(CallType::Instance(c)) => c.clone(),
                    None => class_name.lexeme.clone(),
                };

                let field_info = self.get_struct_field_info(&real_name);
                let has_struct_layout = !field_info.is_empty();
                self.emit(Instruction::AllocStruct {
                    dest: obj_reg,
                    class_name: real_name.clone(),
                });
                let owner_ty = IrType::Ptr(Box::new(IrType::Struct(real_name.clone(), vec![])));
                let construction = self.begin_construction(obj_reg, owner_ty.clone());

                let struct_ty = if has_struct_layout {
                    IrType::Struct(
                        real_name.clone(),
                        field_info
                            .iter()
                            .map(|(_, ty)| ty.clone())
                            .collect()
                    )
                } else {
                    IrType::Any
                };

                let property_map: std::collections::HashMap<String, &Expr> = properties
                    .iter()
                    .map(|(name, expr)| (name.lexeme.clone(), expr))
                    .collect();

                if has_struct_layout {
                    for (i, (field_name, field_ty)) in field_info.iter().enumerate() {
                        let arg_reg = if let Some(expr) = property_map.get(field_name) {
                            let val = self.lower_expr(expr);
                            if self.is_current_block_terminated() {
                                self.end_scope();
                                return obj_reg;
                            }
                            if self.infer_is_arc(expr) && !self.has_result_obligation(expr) {
                                self.emit(Instruction::Ownership(Physical::AcquireSharedEdge{value:val,kind:EdgeKind::Evaluation}));
                            }
                            val
                        } else {
                            let default_reg = self.new_reg();
                            self.emit(Instruction::ConstInt {
                                dest: default_reg,
                                value: 0,
                            });
                            default_reg
                        };

                        let idx0_reg = self.new_reg();
                        self.emit(Instruction::ConstInt {
                            dest: idx0_reg,
                            value: 0,
                        });

                        let idx_reg = self.new_reg();
                        self.emit(Instruction::ConstInt {
                            dest: idx_reg,
                            value: i as i64,
                        });

                        let field_ptr = self.new_reg();
                        self.emit(Instruction::GetElementPtr {
                            dest: field_ptr,
                            base_ty: struct_ty.clone(),
                            base_ptr: obj_reg,
                            indices: vec![idx0_reg, idx_reg],
                        });

                        self.emit(Instruction::Store {
                            ty: field_ty.clone(),
                            ptr: field_ptr,
                            value: arg_reg,
                        });
                    }
                } else {
                    for (i, (_, expr)) in properties.iter().enumerate() {
                        let arg_reg = self.lower_expr(expr);
                        if self.is_current_block_terminated() {
                            self.end_scope();
                            return obj_reg;
                        }

                        let idx0_reg = self.new_reg();
                        self.emit(Instruction::ConstInt {
                            dest: idx0_reg,
                            value: 0,
                        });

                        let idx_reg = self.new_reg();
                        self.emit(Instruction::ConstInt {
                            dest: idx_reg,
                            value: i as i64,
                        });

                        let field_ptr = self.new_reg();
                        self.emit(Instruction::GetElementPtr {
                            dest: field_ptr,
                            base_ty: struct_ty.clone(),
                            base_ptr: obj_reg,
                            indices: vec![idx_reg],
                        });

                        let field_ty = self.infer_ir_type(expr);

                        self.emit(Instruction::Store {
                            ty: field_ty,
                            ptr: field_ptr,
                            value: arg_reg,
                        });
                    }
                }

                self.finish_construction(construction, owner_ty);
                obj_reg
            }

            Expr::Match { value, cases, .. } => {
                self.begin_scope();
                let mut matched = self.lower_expr(value);
                let mut target = self.infer_ir_type(value);
                let input_reference = matched;
                let consuming = !matches!(
                    value.checked_type(),
                    Some(Type::Reference(_) | Type::MutReference(_))
                );
                let mut match_owner = None;

                    let mut source = value.checked_type();
                    while let Some(Type::Reference(inner) | Type::MutReference(inner)) = source {
                        let loaded = self.new_reg();
                        target = self.map_type(inner);
                        self.emit(Instruction::Load {
                            dest: loaded,
                            ty: target.clone(),
                            src_ptr: matched,
                        });
                        matched = loaded;
                        source = Some(inner);
                    }

                if self.infer_is_arc(value) {
                    if !self.has_result_obligation(value) {
                        self.emit(Instruction::Ownership(Physical::AcquireSharedEdge{value:matched,kind:EdgeKind::Evaluation}));
                    }
                    let slot = self.new_reg();
                    self.emit(Instruction::Alloca {
                        dest: slot,
                        name: "match.input".into(),
                        ty: target.clone(),
                    });
                    self.emit(Instruction::Store { ty: target.clone(), value: matched, ptr: slot });
                    self.declare_variable(
                        format!("__match_{}", slot.0),
                        slot,
                        target.clone(),
                        true
                    );
                    match_owner = Some(slot);
                }
                let result_ty = checked_type
                    .map(|ty| self.map_type(ty))
                    .unwrap_or_else(|| {
                        cases
                            .iter()
                            .find_map(|c| {
                                let body = if let [Stmt::Block(b)] = c.body.as_slice() {
                                    b.as_slice()
                                } else {
                                    c.body.as_slice()
                                };
                                if let Some(Stmt::Expression(e)) = body.last() {
                                    Some(self.infer_ir_type(e))
                                } else {
                                    None
                                }
                            })
                            .unwrap_or(IrType::Void)
                    });
                let slot = self.new_reg();
                if result_ty != IrType::Void {
                    self.emit(Instruction::Alloca {
                        dest: slot,
                        name: "match.result".into(),
                        ty: result_ty.clone(),
                    });
                    if self.is_arc_ir_type(&result_ty) {self.register_owner_slot(slot,result_ty.clone(),SlotKind::Temporary);}
                }
                let ownership = self.string_ownership_slot(&result_ty);
                let end = self.new_block("match.end");
                let mut reaches_end = false;
                for case in cases {
                    let next = self.new_block("match.next");
                    // Tests have no ownership effects; bindings are created only after all tests pass.
                    self.test_pattern(&case.pattern, matched, &target, next);
                    self.begin_scope();
                    let whole_binding =
                        matches!(case.pattern.untyped(), Expr::Variable(_)) &&
                        self.enum_pattern(&case.pattern, &target).is_none();
                    if whole_binding  {
                        if consuming {
                            if let Some(owner) = match_owner {
                                self.emit(Instruction::Ownership(Physical::TransferSlot{slot:owner,value:matched,ty:target.clone()}));
                            }
                            self.bind_pattern(&case.pattern, matched, &target, consuming);
                        } else {
                            let ty = self.map_type(
                                case.pattern.checked_type().expect("typed whole reference pattern")
                            );
                            self.bind_pattern(&case.pattern, input_reference, &ty, false);
                        }
                    } else {
                        self.bind_pattern(&case.pattern, matched, &target, consuming);
                    }
                    if let Some(guard) = &case.guard {
                        let cond = self.lower_expr(guard);
                        let yes = self.new_block("match.guard.yes");
                        let no = self.new_block("match.guard.no");
                        self.emit(Instruction::CondBr { cond, if_true: yes, if_false: no });
                        self.set_insert_point(no);
                        let scope = self.scopes.last().unwrap().clone();
                        self.emit_releases_for_scope(&scope);
                        self.emit(Instruction::Br { target: next });
                        self.set_insert_point(yes);
                    }
                    let body = if let [Stmt::Block(b)] = case.body.as_slice() {
                        b.as_slice()
                    } else {
                        case.body.as_slice()
                    };
                    for (i, stmt) in body.iter().enumerate() {
                        if self.is_current_block_terminated() {
                            break;
                        }
                        if i + 1 == body.len() {
                            if let Stmt::Expression(e) = stmt {
                                let value = self.lower_expr(e);
                                if self.is_current_block_terminated() {
                                    continue;
                                }
                                if self.infer_is_arc(e) && !self.has_result_obligation(e) {
                                    self.emit(Instruction::Ownership(Physical::AcquireSharedEdge{value:value,kind:EdgeKind::Evaluation}));
                                }
                                if result_ty != IrType::Void {
                                    self.emit(Instruction::Store {
                                        ty: result_ty.clone(),
                                        value,
                                        ptr: slot,
                                    });
                                }
                                self.store_string_ownership(ownership, value);
                                continue;
                            }
                        }
                        self.lower_stmt(stmt);
                    }
                    self.end_scope();
                    if !self.is_current_block_terminated() {
                        reaches_end = true;
                        self.emit(Instruction::Br { target: end });
                    }
                    self.set_insert_point(next);
                }
                self.inject_panic("Non-exhaustive match");
                self.set_insert_point(end);
                let dest = self.new_reg();
                if !reaches_end {
                    self.emit(Instruction::Unreachable);
                    self.scopes.pop();
                    return dest;
                }
                if result_ty != IrType::Void {
                    if self.is_arc_ir_type(&result_ty){self.emit(Instruction::Ownership(Physical::ReadValue{dest,source:slot,ty:result_ty,mode:ReadMode::Move}));}
                    else{self.emit(Instruction::Load { dest, ty: result_ty, src_ptr: slot });}
                } else {
                    self.emit(Instruction::ConstInt { dest, value: 0 });
                }
                self.load_string_ownership(ownership, dest);
                self.end_scope();
                dest
            }

            Expr::Lambda { keyword, params, body, return_type: stmt_ret_type, .. } => {
                let lambda_name = format!("__fishy_lambda_{}", self.lambda_count);
                self.lambda_count += 1;
                if stmt_ret_type.as_ref() == Some(&Type::Tuple(vec![])) {
                    self.unit_functions.insert(lambda_name.clone());
                }

                // The checker resolves free declaration identities, including captures
                // needed to construct nested closures. Never rediscover captures by name.
                let plan = self.capture_plans
                    .get(&keyword.id)
                    .expect("checked lambda capture plan")
                    .clone();
                let mut captures = plan
                    .iter()
                    .map(|binding| {
                        let (slot, ty, arc) = self.binding_slots
                            .get(&binding.declaration.id)
                            .expect("captured declaration has a lowering slot")
                            .clone();
                        assert_eq!(
                            ty,
                            self.map_type(&binding.ty),
                            "capture plan and storage type disagree"
                        );
                        (binding.declaration.lexeme.clone(), slot, ty, arc)
                    })
                    .collect::<Vec<_>>();
                let old_binding_slots = self.binding_slots.clone();
                for (_, _, ty, arc) in &mut captures {
                    *arc = self.is_arc_ir_type(ty);
                }
                let env_name = format!("{lambda_name}.env");
                let env_fields = captures
                    .iter()
                    .map(|(_, _, t, _)| t.clone())
                    .collect::<Vec<_>>();
                let env_type = IrType::Struct(env_name.clone(), env_fields.clone());
                let env_ptr = self.new_reg();
                if captures.is_empty() {
                    self.emit(Instruction::ConstInt { dest: env_ptr, value: 0 });
                } else {
                    self.synthetic_layouts.insert(env_name.clone(), env_fields);
                    self.emit(Instruction::AllocStruct { dest: env_ptr, class_name: env_name });
                    for (i, (_, reg, ty, _is_arc)) in captures.iter().enumerate() {
                        let value = self.new_reg();
                        let moved=self.ownership.as_ref().unwrap().moved_captures.contains(&(keyword.id,plan[i].declaration.id));
                        self.emit(Instruction::Ownership(Physical::ReadValue{dest:value,source:*reg,ty:ty.clone(),mode:if moved{ReadMode::Move}else{ReadMode::Copy}}));
                        let ptr = self.field_address(env_ptr, env_type.clone(), i);
                        self.emit(Instruction::Store { ty: ty.clone(), value, ptr });
                    }
                }

                // 2. modify the lambda's signature
                let mut ir_args = Vec::new();
                // the first argument of the function is always an env ptr
                ir_args.push((self.new_reg(), IrType::Ptr(Box::new(IrType::I64))));

                for param in params {
                    let p_type = param.type_annotation.as_ref().unwrap_or(&Type::I64);
                    ir_args.push((self.new_reg(), self.map_type(p_type)));
                }

                let ret_type = if let Some(rt) = stmt_ret_type {
                    self.map_type(rt)
                } else {
                    IrType::Void
                };

                let mut func_ir = FunctionIr {
                    name: lambda_name.clone(),
                    ret_type: ret_type.clone(),
                    args: ir_args.clone(),
                    blocks: Vec::new(),
                    is_variadic: false,
                };

                let old_function = self.current_function.replace(lambda_name.clone());
                let old_blocks = std::mem::take(&mut self.blocks);
                let old_current = self.current_block.take();
                let old_scopes = std::mem::take(&mut self.scopes);
                // Globals are addressed through their actual global slots, not
                // copied into every closure environment (including the private
                // canonical empty String root in the monolithic prelude).
                self.scopes.push(self.globals.clone());

                // 3. build the entire body
                let entry_block = self.new_block("entry");
                self.set_insert_point(entry_block);
                self.begin_scope();

                let env_arg_reg = ir_args[0].0;

                // unpack the env and recreate the original variables
                if !captures.is_empty() {
                    for (i, (_, _, ty, _)) in captures.iter().enumerate() {
                        let item_ptr=self.field_address(env_arg_reg,env_type.clone(),i);
                        self.declare_binding(&plan[i].declaration,item_ptr,ty.clone(),false);
                    }
                }

                // process the real arguments for the call
                for (i, param) in params.iter().enumerate() {
                    let arg_reg = ir_args[i + 1].0; // +1 because pos 0 is the env
                    let ptr_reg = self.new_reg();

                    let p_type = param.type_annotation.as_ref().unwrap_or(&Type::I64);
                    let ir_ty = self.map_type(p_type);
                    let is_arc = self.is_arc_ty(p_type);

                    self.emit(Instruction::Alloca {
                        dest: ptr_reg,
                        name: param.name.lexeme.clone(),
                        ty: ir_ty.clone(),
                    });

                    self.emit(Instruction::Store {
                        ty: ir_ty.clone(),
                        ptr: ptr_reg,
                        value: arg_reg,
                    });

                    self.declare_binding(&param.name, ptr_reg, ir_ty, is_arc);

                }

                for s in body {
                    self.lower_stmt(s);
                }

                self.end_scope();

                if !self.is_current_block_terminated() {
                    let zero = self.new_reg();
                    self.emit(Instruction::ConstInt {
                        dest: zero,
                        value: 0,
                    });
                    self.emit(Instruction::Ret {
                        value: if ret_type == IrType::Void {
                            None
                        } else {
                            Some(zero)
                        },
                    });
                }

                let mut sorted_blocks: Vec<_> = self.blocks
                    .drain()
                    .map(|(_, b)| b)
                    .collect();
                sorted_blocks.sort_by_key(|b| b.id.0);
                func_ir.blocks = sorted_blocks;
                self.functions.push(func_ir);

                self.blocks = old_blocks;
                self.current_block = old_current;
                self.scopes = old_scopes;
                self.current_function = old_function;

                self.binding_slots = old_binding_slots;
                // 4. create the closure
                let dest = self.new_reg();
                self.emit(Instruction::MakeClosure {
                    dest,
                    fn_name: lambda_name,
                    env_ptr,
                });
                dest
            }

            _ => unimplemented!("Lowering for {} expression not implemented yet", expr),
        }
    }

    pub fn lower_stmt(&mut self, stmt: &Stmt) {
        if self.is_current_block_terminated() {
            return;
        }
        match stmt {
            Stmt::Export { .. } => unreachable!("module exports are resolved before lowering"),
            Stmt::Alias { .. } | Stmt::Trait { .. } | Stmt::Using { .. } => {}

            Stmt::ExternFunction { name, params, return_type, is_variadic } => {
                let mut ir_args = Vec::new();
                for param in params {
                    let p_type = param.type_annotation.as_ref().unwrap_or(&Type::I64);
                    ir_args.push((self.new_reg(), self.map_type(p_type)));
                }

                self.functions.push(FunctionIr {
                    name: name.lexeme.clone(),
                    ret_type: return_type
                        .as_ref()
                        .map(|t| self.map_type(t))
                        .unwrap_or(IrType::Void),
                    args: ir_args,
                    blocks: vec![],
                    is_variadic: *is_variadic,
                });
            }

            Stmt::Expression(expr) => {
                self.discard_expr(expr);
            }

            Stmt::Var { name, type_annotation, initializer, .. } => {
                let ptr_reg = if self.current_function.is_none() {
                    self.globals
                        .get(&name.lexeme)
                        .map(|(r, _, _)| *r)
                        .unwrap_or_else(|| self.new_reg())
                } else {
                    self.new_reg()
                };

                let (ir_ty, is_arc) = if let Some(annot) = type_annotation {
                    (self.map_type(annot), self.is_arc_ty(annot))
                } else if let Some(init) = &initializer {
                    (self.infer_ir_type(init), self.infer_is_arc(init))
                } else {
                    (IrType::I64, false)
                };

                self.emit(Instruction::Alloca {
                    dest: ptr_reg,
                    name: name.lexeme.clone(),
                    ty: ir_ty.clone(),
                });

                if is_arc {self.register_owner_slot(ptr_reg,ir_ty.clone(),SlotKind::Local);}
                let zero = self.new_reg();
                self.emit(Instruction::ConstInt {
                    dest: zero,
                    value: 0,
                });
                self.emit(Instruction::Store {
                    ty: ir_ty.clone(),
                    ptr: ptr_reg,
                    value: zero,
                });

                if let Some(init_expr) = initializer {
                    let val_reg = self.lower_expr(init_expr);
                    if self.is_current_block_terminated() {
                        return;
                    }
                    if is_arc && !self.has_result_obligation(init_expr) {
                        self.emit(Instruction::Ownership(Physical::AcquireSharedEdge{value:val_reg,kind:EdgeKind::Evaluation}));
                    }
                    if matches!(init_expr.checked_type(), Some(Type::String | Type::Pointer(_))) {
                    }
                    {
                        self.emit(Instruction::Store {
                            ty: ir_ty.clone(),
                            ptr: ptr_reg,
                            value: val_reg,
                        });
                    }
                }
                self.declare_binding(name, ptr_reg, ir_ty, is_arc);
            }

            Stmt::Return { value, .. } => {
                let mut ret_val = value.as_ref().map(|e| self.lower_expr(e));
                if
                    value.is_none() &&
                    self.unit_functions.contains(self.current_function.as_deref().unwrap_or(""))
                {
                    let unit = self.new_reg();
                    self.emit(Instruction::ConstInt { dest: unit, value: 0 });
                    ret_val = Some(unit);
                }
                if self.is_current_block_terminated() {
                    return;
                }
                if let (Some(e), Some(ptr)) = (value, ret_val) {
                    if self.infer_is_arc(e) && !self.has_result_obligation(e) {
                        self.emit(Instruction::Ownership(Physical::AcquireSharedEdge{value:ptr,kind:EdgeKind::Evaluation}));
                    }
                }

                let active_scopes = self.scopes.clone();
                for scope in active_scopes.iter().rev() {
                    self.emit_releases_for_scope(scope);
                }

                let ret_val = ret_val.filter(
                    |_|
                        !value
                            .as_ref()
                            .is_some_and(|e| matches!(e.checked_type(), Some(Type::Void)))
                );
                if self.current_function.as_deref() == Some("main") {
                    let dest = self.new_reg();
                    self.emit(Instruction::Call {
                        dest,
                        func_name: "__fishy_program_exit".into(),
                        args: vec![],
                    });
                }
                self.emit(Instruction::Ret { value: ret_val });
            }

            Stmt::Block(stmts) => {
                self.begin_scope();
                for s in stmts {
                    self.lower_stmt(s);
                }
                self.end_scope();
            }

            Stmt::If { condition, then_branch, else_branch, .. } => {
                let cond_reg = self.lower_expr(condition);
                let then_block = self.new_block("if.then");
                let merge_block = self.new_block("if.end");
                let else_block = if else_branch.is_some() {
                    Some(self.new_block("if.else"))
                } else {
                    None
                };

                self.emit(Instruction::CondBr {
                    cond: cond_reg,
                    if_true: then_block,
                    if_false: else_block.unwrap_or(merge_block),
                });

                self.set_insert_point(then_block);
                self.lower_stmt(then_branch);
                let mut reaches_merge = false;
                if !self.is_current_block_terminated() {
                    reaches_merge = true;
                    self.emit(Instruction::Br {
                        target: merge_block,
                    });
                }

                if let Some(eb) = else_block {
                    self.set_insert_point(eb);
                    self.lower_stmt(else_branch.as_ref().unwrap());
                    if !self.is_current_block_terminated() {
                        reaches_merge = true;
                        self.emit(Instruction::Br {
                            target: merge_block,
                        });
                    }
                }
                self.set_insert_point(merge_block);
                if else_block.is_some() && !reaches_merge {
                    self.emit(Instruction::Unreachable);
                }
            }

            Stmt::While { condition, body, .. } => {
                let cond_block = self.new_block("while.cond");
                let body_block = self.new_block("while.body");
                let end_block = self.new_block("while.end");

                self.emit(Instruction::Br { target: cond_block });
                self.set_insert_point(cond_block);
                let cond_reg = self.lower_expr(condition);
                self.emit(Instruction::CondBr {
                    cond: cond_reg,
                    if_true: body_block,
                    if_false: end_block,
                });

                self.set_insert_point(body_block);
                self.lower_stmt(body);
                if !self.is_current_block_terminated() {
                    self.emit(Instruction::Br { target: cond_block });
                }
                self.set_insert_point(end_block);
            }

            Stmt::For { initializer, condition, increment, body, .. } => {
                self.begin_scope();
                if let Some(i) = initializer {
                    self.lower_stmt(i);
                }
                let test = self.new_block("for.test");
                let loop_body = self.new_block("for.body");
                let end = self.new_block("for.end");
                self.emit(Instruction::Br { target: test });
                self.set_insert_point(test);
                if let Some(c) = condition {
                    let cond = self.lower_expr(c);
                    self.emit(Instruction::CondBr { cond, if_true: loop_body, if_false: end });
                } else {
                    self.emit(Instruction::Br { target: loop_body });
                }
                self.set_insert_point(loop_body);
                self.begin_scope();
                for s in body {
                    self.lower_stmt(s);
                }
                self.end_scope();
                if !self.is_current_block_terminated() {
                    if let Some(i) = increment {
                        self.discard_expr(i);
                    }
                    self.emit(Instruction::Br { target: test });
                }
                self.set_insert_point(end);
                self.end_scope();
                if condition.is_none() {
                    self.emit(Instruction::Unreachable);
                }
            }

            Stmt::ForIn { keyword: _, key, value, iterable, body } => {
                if
                    matches!(
                        iterable.checked_type().map(Self::referenced_type),
                        Some(Type::Range(_))
                    )
                {
                    self.lower_range_loop(key, value.as_ref(), iterable, body);
                    return;
                }
                self.begin_scope();
                let arr_ptr = self.lower_index_base(iterable);

                let iterator = self.new_reg();
                let iterator_type = self.map_type(
                    Self::referenced_type(iterable.checked_type().expect("typed iterator"))
                );
                self.emit(Instruction::Alloca {
                    dest: iterator,
                    name: "forin.iterator".into(),
                    ty: iterator_type.clone(),
                });
                self.emit(Instruction::Store {
                    ty: iterator_type.clone(),
                    value: arr_ptr,
                    ptr: iterator,
                });
                self.declare_variable(
                    format!("__forin_{}", iterator.0),
                    iterator,
                    iterator_type,
                     self.has_result_obligation(iterable)
                );
                self.inject_null_check(arr_ptr, "Null pointer dereference no loop 'for in'!");

                let minus_one = self.new_reg();
                self.emit(Instruction::ConstInt {
                    dest: minus_one,
                    value: -1,
                });

                let size_ptr = self.new_reg();
                self.emit(Instruction::GetElementPtr {
                    dest: size_ptr,
                    base_ty: IrType::I64,
                    base_ptr: arr_ptr,
                    indices: vec![minus_one],
                });

                let len_reg = self.new_reg();
                self.emit(Instruction::Load {
                    dest: len_reg,
                    ty: IrType::I64,
                    src_ptr: size_ptr,
                });

                let counter_ptr = self.new_reg();
                self.emit(Instruction::Alloca {
                    dest: counter_ptr,
                    name: "for_in_counter".into(),
                    ty: IrType::I64,
                });

                let zero = self.new_reg();
                self.emit(Instruction::ConstInt {
                    dest: zero,
                    value: 0,
                });
                self.emit(Instruction::Store {
                    ty: IrType::I64,
                    ptr: counter_ptr,
                    value: zero,
                });

                let cond_block = self.new_block("forin.cond");
                let body_block = self.new_block("forin.body");
                let end_block = self.new_block("forin.end");

                self.emit(Instruction::Br { target: cond_block });
                self.set_insert_point(cond_block);

                let current_i = self.new_reg();
                self.emit(Instruction::Load {
                    dest: current_i,
                    ty: IrType::I64,
                    src_ptr: counter_ptr,
                });

                let is_less = self.new_reg();
                self.emit(Instruction::CmpLt {
                    dest: is_less,
                    left: current_i,
                    right: len_reg,
                    ty: IrType::I64,
                });
                self.emit(Instruction::CondBr {
                    cond: is_less,
                    if_true: body_block,
                    if_false: end_block,
                });

                self.set_insert_point(body_block);
                self.begin_scope();

                let item_ty = self.index_element_type(iterable);
                let mut item_val = self.new_reg();
                let item_ptr = self.new_reg();

                self.emit(Instruction::GetElementPtr {
                    dest: item_ptr,
                    base_ty: item_ty.clone(),
                    base_ptr: arr_ptr,
                    indices: vec![current_i],
                });

                self.emit(Instruction::Load {
                    dest: item_val,
                    ty: item_ty.clone(),
                    src_ptr: item_ptr,
                });

                let item_binding = value.as_ref().unwrap_or(key);
                let item_ty = if
                    self.ownership
                        .as_ref()
                        .is_some_and(|p| p.pattern_views.contains(&item_binding.id))
                {
                    item_val = item_ptr;
                    self.map_type(
                        &self.ownership.as_ref().unwrap().pattern_bindings[&item_binding.id]
                    )
                } else {
                    item_ty
                };

                if let Some(val_token) = value {
                    let index_var_ptr = self.new_reg();
                    self.emit(Instruction::Alloca {
                        dest: index_var_ptr,
                        name: key.lexeme.clone(),
                        ty: IrType::I64,
                    });
                    self.emit(Instruction::Store {
                        ty: IrType::I64,
                        ptr: index_var_ptr,
                        value: current_i,
                    });
                    self.declare_binding(key, index_var_ptr, IrType::I64, false);

                    let item_is_arc = self.is_arc_ir_type(&item_ty);
                    let item_var_ptr = self.new_reg();
                    self.emit(Instruction::Alloca {
                        dest: item_var_ptr,
                        name: val_token.lexeme.clone(),
                        ty: item_ty.clone(),
                    });
                    self.emit(Instruction::Store {
                        ty: item_ty.clone(),
                        ptr: item_var_ptr,
                        value: item_val,
                    });
                    self.declare_binding(val_token, item_var_ptr, item_ty.clone(), item_is_arc);
                } else {
                    let item_is_arc = self.is_arc_ir_type(&item_ty);
                    let item_var_ptr = self.new_reg();
                    self.emit(Instruction::Alloca {
                        dest: item_var_ptr,
                        name: key.lexeme.clone(),
                        ty: item_ty.clone(),
                    });
                    self.emit(Instruction::Store {
                        ty: item_ty.clone(),
                        ptr: item_var_ptr,
                        value: item_val,
                    });
                    self.declare_binding(key, item_var_ptr, item_ty.clone(), item_is_arc);
                }

                for s in body {
                    self.lower_stmt(s);
                }

                self.end_scope();
                if !self.is_current_block_terminated() {
                    let one = self.new_reg();
                    self.emit(Instruction::ConstInt {
                        dest: one,
                        value: 1,
                    });

                    let latest_i = self.new_reg();
                    self.emit(Instruction::Load {
                        dest: latest_i,
                        ty: IrType::I64,
                        src_ptr: counter_ptr,
                    });

                    let next_i = self.new_reg();
                    self.emit(Instruction::Add {
                        dest: next_i,
                        left: latest_i,
                        right: one,
                    });
                    self.emit(Instruction::Store {
                        ty: IrType::I64,
                        ptr: counter_ptr,
                        value: next_i,
                    });

                    self.emit(Instruction::Br { target: cond_block });
                }

                self.set_insert_point(end_block);
                self.end_scope();
            }

            Stmt::Function { name, params, body, return_type: stmt_ret_type, type_params, .. } => {
                if !type_params.is_empty() {
                    return;
                }

                let unit_return = stmt_ret_type.as_ref() == Some(&Type::Tuple(vec![]));
                if unit_return {
                    self.unit_functions.insert(name.lexeme.clone());
                }
                let mut ir_args = Vec::new();
                for param in params {
                    let p_type = param.type_annotation.as_ref().unwrap_or(&Type::I64);
                    ir_args.push((self.new_reg(), self.map_type(p_type)));
                }

                let ret_type = if let Some(rt) = stmt_ret_type {
                    self.map_type(rt)
                } else {
                    IrType::Void
                };

                let mut func_ir = FunctionIr {
                    name: name.lexeme.clone(),
                    ret_type: ret_type.clone(),
                    args: ir_args.clone(),
                    blocks: Vec::new(),
                    is_variadic: false,
                };

                if body.is_none() {
                    self.functions.push(func_ir);
                    return;
                }

                let old_function = self.current_function.replace(name.lexeme.clone());
                let old_blocks = std::mem::take(&mut self.blocks);
                let old_current = self.current_block.take();
                let old_scopes = std::mem::replace(&mut self.scopes, vec![self.globals.clone()]);

                let entry_block = self.new_block("entry");
                self.set_insert_point(entry_block);
                self.begin_scope();
                if name.lexeme == "main" {
                    let dest = self.new_reg();
                    self.emit(Instruction::Call {
                        dest,
                        func_name: "__global_init".into(),
                        args: vec![],
                    });
                }

                for (i, param) in params.iter().enumerate() {
                    let arg_reg = ir_args[i].0;
                    let ptr_reg = self.new_reg();
                    let p_type = param.type_annotation.as_ref().unwrap_or(&Type::I64);
                    let ir_ty = self.map_type(p_type);
                    let is_arc = self.is_arc_ty(p_type);

                    self.emit(Instruction::Alloca {
                        dest: ptr_reg,
                        name: param.name.lexeme.clone(),
                        ty: ir_ty.clone(),
                    });

                    self.emit(Instruction::Store {
                        ty: ir_ty.clone(),
                        ptr: ptr_reg,
                        value: arg_reg,
                    });

                    self.declare_binding(&param.name, ptr_reg, ir_ty, is_arc);


                }

                if let Some(stmts) = body {
                    for s in stmts {
                        self.lower_stmt(s);
                    }
                }

                self.end_scope();

                if !self.is_current_block_terminated() {
                    if name.lexeme == "main" {
                        let dest = self.new_reg();
                        self.emit(Instruction::Call {
                            dest,
                            func_name: "__fishy_program_exit".into(),
                            args: vec![],
                        });
                    }
                    let value = if unit_return {
                        let unit = self.new_reg();
                        self.emit(Instruction::ConstInt { dest: unit, value: 0 });
                        Some(unit)
                    } else {
                        None
                    };
                    self.emit(Instruction::Ret { value });
                }

                let mut sorted_blocks: Vec<_> = self.blocks
                    .drain()
                    .map(|(_, b)| b)
                    .collect();
                sorted_blocks.sort_by_key(|b| b.id.0);
                func_ir.blocks = sorted_blocks;
                self.functions.push(func_ir);

                self.blocks = old_blocks;
                self.current_block = old_current;
                self.scopes = old_scopes;
                self.current_function = old_function;
            }

            Stmt::Struct { .. } => {}

            Stmt::Impl { target_type, trait_name, methods, type_params, .. } => {
                if !type_params.is_empty() {
                    return;
                }
                if let Some(struct_name) = Self::impl_type_name(target_type) {
                    if let Some(Type::Custom(t_name)) = trait_name {
                        let vtable_id = crate::symbols::vtable(&struct_name, t_name);

                        if let Some(layout) = self.trait_vtable_layout.get(t_name) {
                            let mut ordered_methods = vec!["".to_string(); layout.len()];

                            for method in methods {
                                if let Stmt::Function { name: m_name, .. } = method {
                                    if let Some(&idx) = layout.get(&m_name.lexeme) {
                                        ordered_methods[idx] = crate::symbols::member(
                                            &struct_name,
                                            &m_name.lexeme
                                        );
                                    }
                                }
                            }
                            self.vtables.insert(vtable_id, ordered_methods);
                        }
                    }

                    for method in methods {
                        if
                            matches!(method,Stmt::Function {type_params,..} if !type_params.is_empty())
                        {
                            continue;
                        }
                        if
                            let Stmt::Function {
                                name: m_name,
                                params,
                                body,

                                return_type,
                                is_abstract,
                                is_public,
                                ..
                            } = method
                        {
                            let mangled_name = crate::symbols::member(&struct_name, &m_name.lexeme);

                            let modified_params = params.clone();
                            let mangled_func = Stmt::Function {
                                ownership_signature: None,
                                name: Token::synthetic(TokenType::Identifier, &mangled_name),
                                params: modified_params,
                                body: body.clone(),

                                return_type: return_type.clone(),
                                is_abstract: *is_abstract,
                                is_public: *is_public,
                                type_params: vec![],
                            };

                            self.lower_stmt(&mangled_func);
                        }
                    }
                }
            }

            Stmt::Enum { name, cases, type_params } => {
                if !type_params.is_empty() {
                    return;
                }
                for (tag, case) in cases.iter().enumerate() {
                    let layout_name = format!("{}#{}", name.lexeme, case.name.lexeme);
                    let types = self.user_types[&name.lexeme].static_methods[&case.name.lexeme].0
                        .iter()
                        .map(|t| self.map_type(t))
                        .collect::<Vec<_>>();
                    let mut fields = vec![IrType::I64];
                    fields.extend(types.clone());
                    self.synthetic_layouts.insert(layout_name.clone(), fields.clone());
                    let args = types
                        .into_iter()
                        .map(|t| (self.new_reg(), t))
                        .collect::<Vec<_>>();
                    let old_blocks = std::mem::take(&mut self.blocks);
                    let old_current = self.current_block.take();
                    let entry = self.new_block("entry");
                    self.set_insert_point(entry);
                    let dest = self.new_reg();
                    self.emit(Instruction::AllocStruct { dest, class_name: layout_name.clone() });
                    let tag_value = self.new_reg();
                    self.emit(Instruction::ConstInt { dest: tag_value, value: tag as i64 });
                    let base_ty = IrType::Struct(layout_name, fields);
                    let ptr = self.field_address(dest, base_ty.clone(), 0);
                    self.emit(Instruction::Store { ty: IrType::I64, value: tag_value, ptr });
                    for (i, (value, ty)) in args.iter().enumerate() {

                        let ptr = self.field_address(dest, base_ty.clone(), i + 1);
                        self.emit(Instruction::Store { ty: ty.clone(), value: *value, ptr });
                    }
                    self.emit(Instruction::Ret { value: Some(dest) });
                    let mut blocks = self.blocks
                        .drain()
                        .map(|(_, b)| b)
                        .collect::<Vec<_>>();
                    blocks.sort_by_key(|b| b.id.0);
                    let ret_type = self.map_type(&Type::Custom(name.lexeme.clone()));
                    self.functions.push(FunctionIr {
                        name: crate::symbols::member(&name.lexeme, &case.name.lexeme),
                        args,
                        ret_type,
                        blocks,
                        is_variadic: false,
                    });
                    self.blocks = old_blocks;
                    self.current_block = old_current;
                }
            }

            Stmt::ArrayDestructuring { keyword: _, bindings, initializer } => {
                let array_ptr = self.lower_expr(initializer);
                self.inject_null_check(array_ptr, "Null destructuring value");
                let aggregate = self.deref_type(&self.infer_ir_type(initializer));
                for (i, binding) in bindings.iter().enumerate() {
                    let (element_ty, element_ptr) = if
                        let IrType::Struct(_, ref fields) = aggregate
                    {
                        (fields[i].clone(), self.field_address(array_ptr, aggregate.clone(), i))
                    } else {
                        let ty = self.deref_type(&self.infer_ir_type(initializer));
                        let idx = self.new_reg();
                        self.emit(Instruction::ConstInt { dest: idx, value: i as i64 });
                        self.check_index(initializer, array_ptr, idx);
                        let ptr = self.new_reg();
                        self.emit(Instruction::GetElementPtr {
                            dest: ptr,
                            base_ty: ty.clone(),
                            base_ptr: array_ptr,
                            indices: vec![idx],
                        });
                        (ty, ptr)
                    };
                    let value = self.new_reg();
                    let arc = self.is_arc_ir_type(&element_ty);
                    self.emit(Instruction::Ownership(Physical::ReadValue{dest:value,source:element_ptr,ty:element_ty.clone(),mode:if arc||self.binding_moves(binding.id){ReadMode::Move}else{ReadMode::Copy}}));
                    let ptr = self.new_reg();
                    self.emit(Instruction::Alloca {
                        dest: ptr,
                        name: binding.lexeme.clone(),
                        ty: element_ty.clone(),
                    });
                    self.emit(Instruction::Store { ty: element_ty.clone(), value, ptr });
                    self.declare_binding(binding, ptr, element_ty, arc);
                }
                if self.infer_is_arc(initializer) && self.has_result_obligation(initializer) {
                    self.emit(Instruction::Ownership(Physical::DropValue{value:array_ptr,recipe:crate::physical_ownership::DropRecipe::Temporary}));
                }
            }

            _ => unimplemented!("Lowering for {stmt} not supported yet."),
        }
    }
}

#[cfg(test)]
#[path = "../tests/support/custom_ir_tests.rs"]
mod contract_tests;

#[cfg(test)]
#[path = "../tests/support/ownership_tests.rs"]
mod ownership_tests;

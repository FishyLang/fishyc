use std::collections::{ HashMap, HashSet };
use crate::ast::{ Expr, Stmt, Type };
use crate::token::{ Literal, Token, TokenType };
use crate::ir::{ VReg, BlockId, IrType, Instruction, BasicBlock, FunctionIr, ModuleIr };
use crate::type_checker::{ CallType, StructInfo };

pub struct IrBuilder {
    next_reg: usize,
    next_block: usize,
    current_block: Option<BlockId>,
    blocks: HashMap<BlockId, BasicBlock>,
    functions: Vec<FunctionIr>,
    scopes: Vec<HashMap<String, (VReg, IrType, bool)>>,
    pub property_indices: HashMap<Token, usize>,
    pub resolved_calls: HashMap<Token, CallType>,
    pub variant_tags: HashMap<String, usize>,
    pub traits: HashSet<String>,
    pub vtables: HashMap<String, Vec<String>>,
    pub trait_vtable_layout: HashMap<String, HashMap<String, usize>>,
    pub user_types: HashMap<String, StructInfo>,
    pub lambda_count: usize,
}

impl IrBuilder {
    pub fn new(
        property_indices: HashMap<Token, usize>,
        resolved_calls: HashMap<Token, CallType>,
        traits: HashSet<String>,
        trait_vtable_layout: HashMap<String, HashMap<String, usize>>, 
        user_types: HashMap<String, StructInfo>,
    ) -> Self {
        Self {
            next_reg: 0,
            next_block: 0,
            current_block: None,
            blocks: HashMap::new(),
            functions: Vec::new(),
            scopes: vec![HashMap::new()],
            property_indices,
            resolved_calls,
            variant_tags: HashMap::new(),
            traits,
            vtables: HashMap::new(),
            trait_vtable_layout,
            user_types,
            lambda_count: 0,
        }
    }

    pub fn build(mut self, stmts: &[Stmt]) -> ModuleIr {
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
                _ => self.lower_stmt(stmt),
            }
        }

        self.end_scope();

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

        ModuleIr {
            name: "Fishy_Module".to_string(),
            functions: self.functions,
            vtables: self.vtables,
        }
    }

    fn inject_panic(&mut self, message: &str) {
        let msg_reg = self.new_reg();
        self.emit(Instruction::ConstString { dest: msg_reg, value: message.to_string() });
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
        self.emit(Instruction::ConstInt { dest: null_val, value: 0 });

        let is_not_null = self.new_reg();
        self.emit(Instruction::CmpNeq { dest: is_not_null, left: ptr, right: null_val });

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
        self.emit(Instruction::ConstInt { dest: minus_one, value: -1 });

        let size_ptr = self.new_reg();
        self.emit(Instruction::GetElementPtr {
            dest: size_ptr,
            base_ty: IrType::I64,
            base_ptr: ptr,
            indices: vec![minus_one],
        });

        let limit_reg = self.new_reg();
        self.emit(Instruction::Load { dest: limit_reg, ty: IrType::I64, src_ptr: size_ptr });

        let zero_reg = self.new_reg();
        self.emit(Instruction::ConstInt { dest: zero_reg, value: 0 });

        let is_valid_upper = self.new_reg();
        self.emit(Instruction::CmpLt { dest: is_valid_upper, left: idx, right: limit_reg });

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
        self.emit(Instruction::CmpGe { dest: is_valid_lower, left: idx, right: zero_reg });
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
        if let Expr::Cast { value, .. } = target {
            target = &**value;
        }

        if let Expr::Variable(name) = target {
            if let Some((ptr_reg, var_ty, _)) = self.resolve_variable(&name.lexeme) {
                let null_val = self.new_reg();
                self.emit(Instruction::ConstInt { dest: null_val, value: 0 });
                self.emit(Instruction::Store { ty: var_ty, ptr: ptr_reg, value: null_val });
            }
        } else if let Expr::Get { object, name } = target {
            let obj_ptr = self.lower_expr(object);
            if let Some(field_index) = self.property_indices.get(name).copied() {
                let idx_reg = self.new_reg();
                self.emit(Instruction::ConstInt { dest: idx_reg, value: field_index as i64 });
                let field_ptr = self.new_reg();
                self.emit(Instruction::GetElementPtr {
                    dest: field_ptr,
                    base_ty: IrType::Any,
                    base_ptr: obj_ptr,
                    indices: vec![idx_reg],
                });
                let null_val = self.new_reg();
                self.emit(Instruction::ConstInt { dest: null_val, value: 0 });
                self.emit(Instruction::Store { ty: IrType::I64, ptr: field_ptr, value: null_val });
            }
        }
    }

    pub fn map_type(&self, ast_type: &Type) -> IrType {
        match ast_type {
            Type::Void => IrType::Void,
            Type::U8 | Type::I8 => IrType::I8,
            Type::U16 | Type::I16 => IrType::I16,
            Type::U32 | Type::I32 => IrType::I32,
            Type::U64 | Type::I64 => IrType::I64,
            Type::F16 | Type::F32 => IrType::F32,
            Type::F64 => IrType::F64,
            Type::Bool => IrType::Bool,
            Type::String => IrType::Ptr(Box::new(IrType::I8)),
            Type::Pointer(inner) | Type::Reference(inner) | Type::MutReference(inner) =>
                IrType::Ptr(Box::new(self.map_type(inner))),
            Type::Slice(inner) => IrType::Ptr(Box::new(self.map_type(inner))),
            Type::Array(size, inner) => IrType::Array(*size, Box::new(self.map_type(inner))),
            Type::Custom(name) => {
                if self.traits.contains(name) {
                    IrType::FatPtr
                } else {
                    IrType::Struct(name.clone(), vec![])
                }
            }
            _ => IrType::I64,
        }
    }

    fn infer_ir_type(&self, expr: &Expr) -> IrType {
        match expr {
            Expr::Literal(Literal::Number(_)) => IrType::F64,
            Expr::Literal(Literal::Integer(_)) => IrType::I64,
            Expr::Literal(Literal::String(_)) => IrType::Ptr(Box::new(IrType::I8)),
            Expr::Literal(Literal::Bool(_)) => IrType::Bool,

            Expr::Variable(name) =>
                self
                    .resolve_variable(&name.lexeme)
                    .map(|(_, ty, _)| ty)
                    .unwrap_or(IrType::I64),

            Expr::Call { callee, .. } => {
                if let Expr::Variable(name) = &**callee {
                    if let Some(func) = self.functions.iter().find(|f| f.name == name.lexeme) {
                        return func.ret_type.clone();
                    }
                }

                IrType::I64
            }

            Expr::Get { .. } => IrType::I64,

            Expr::SubscriptGet { indexee, .. } => self.infer_ir_type(indexee),

            _ => IrType::I64,
        }
    }

    fn deref_type(&self, ty: &IrType) -> IrType {
        match ty {
            IrType::Ptr(inner) => (**inner).clone(),
            IrType::Array(_, inner) => (**inner).clone(),
            other => other.clone(),
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
        if let Some(block_id) = self.current_block {
            if let Some(block) = self.blocks.get_mut(&block_id) {
                block.instructions.push(inst);
            }
        } else {
            eprintln!("WARNING: Instruction emitted with no current block: {:?}", inst);
        }
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
        for (name, (ptr_reg, ty, is_arc)) in scope {
            if *is_arc {
                let val_reg = self.new_reg();
                self.emit(Instruction::Load { dest: val_reg, ty: ty.clone(), src_ptr: *ptr_reg });

                let is_not_null = self.new_reg();
                let zero = self.new_reg();
                self.emit(Instruction::ConstInt { dest: zero, value: 0 });
                self.emit(Instruction::CmpNeq { dest: is_not_null, left: val_reg, right: zero });

                let release_block = self.new_block(&format!("arc.release.{}", name));
                let skip_block = self.new_block(&format!("arc.skip.{}", name));

                self.emit(Instruction::CondBr {
                    cond: is_not_null,
                    if_true: release_block,
                    if_false: skip_block,
                });

                self.set_insert_point(release_block);
                self.emit(Instruction::Release { ptr: val_reg });
                self.emit(Instruction::Br { target: skip_block });

                self.set_insert_point(skip_block);
            }
        }
    }

    fn declare_variable(&mut self, name: String, ptr_reg: VReg, ty: IrType, is_arc: bool) {
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
                            Instruction::Ret { .. }
                    );
                }
            }
        }
        false
    }

    fn is_arc_ty(&self, ty: &Type) -> bool {
        match ty {
            Type::Array(_, _) => true,
            Type::Custom(name) => !self.traits.contains(name),
            _ => false,
        }
    }

    fn infer_is_arc(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Array { .. } | Expr::Lambda { .. } | Expr::New { .. } => true,
            Expr::Variable(name) | Expr::This(name) => {
                self.resolve_variable(&name.lexeme)
                    .map(|(_, _, is_arc)| is_arc)
                    .unwrap_or(false)
            }

            _ => false,
        }
    }

    fn lower_lvalue(&mut self, expr: &Expr) -> VReg {
        match expr {
            Expr::Variable(name) | Expr::This(name) => {
                self.resolve_variable(&name.lexeme).expect("Variable not found!").0
            }
            Expr::Dereference { operand, .. } => self.lower_expr(operand),
            Expr::SubscriptGet { indexee, index, .. } => {
                let base_ptr = self.lower_expr(indexee);
                let idx_val = self.lower_expr(index);
                let dest = self.new_reg();

                self.emit(Instruction::GetElementPtr {
                    dest,
                    base_ty: IrType::I64,
                    base_ptr,
                    indices: vec![idx_val],
                });

                dest
            }
            _ => panic!("Expression is not a valid L-Value!"),
        }
    }

    pub fn lower_expr(&mut self, expr: &Expr) -> VReg {
        match expr {
            Expr::Literal(lit) =>
                match lit {
                    Literal::Number(n) => {
                        let dest = self.new_reg();
                        self.emit(Instruction::ConstFloat { dest, value: *n, ty: IrType::F64 });
                        dest
                    }

                    Literal::Integer(n) => {
                        let dest = self.new_reg();
                        self.emit(Instruction::ConstInt { dest, value: *n });
                        dest
                    }

                    Literal::String(s) => {
                        let dest = self.new_reg();
                        self.emit(Instruction::ConstString { dest, value: s.clone() });
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
                let (ptr_reg, var_ty, is_arc) = self
                    .resolve_variable(&name.lexeme)
                    .expect("Variable not declared!");

                let dest = self.new_reg();
                self.emit(Instruction::Load { dest, ty: var_ty, src_ptr: ptr_reg });

                // after reading a managed variable, we retain (+1)
                if is_arc {
                    let is_not_null = self.new_reg();
                    let zero = self.new_reg();
                    self.emit(Instruction::ConstInt { dest: zero, value: 0 });
                    self.emit(Instruction::CmpNeq { dest: is_not_null, left: dest, right: zero });

                    let retain_block = self.new_block("arc.retain");
                    let skip_block = self.new_block("arc.skip");

                    self.emit(Instruction::CondBr {
                        cond: is_not_null,
                        if_true: retain_block,
                        if_false: skip_block,
                    });

                    self.set_insert_point(retain_block);
                    self.emit(Instruction::Retain { ptr: dest });
                    self.emit(Instruction::Br { target: skip_block });

                    self.set_insert_point(skip_block);
                }
                dest
            }

            Expr::Assign { name, value } => {
                let val_reg = self.lower_expr(value);
                let (ptr_reg, var_ty, is_arc) = self
                    .resolve_variable(&name.lexeme)
                    .expect("Variable not declared!");

                // if we replace a variable, we free the old one and retain the new one
                if is_arc {
                    let old_val = self.new_reg();

                    self.emit(Instruction::Load {
                        dest: old_val,
                        ty: var_ty.clone(),
                        src_ptr: ptr_reg,
                    });

                    let is_not_null = self.new_reg();
                    let zero = self.new_reg();

                    self.emit(Instruction::ConstInt { dest: zero, value: 0 });

                    self.emit(Instruction::CmpNeq {
                        dest: is_not_null,
                        left: old_val,
                        right: zero,
                    });

                    let release_block = self.new_block("arc.assign.release");
                    let skip_block = self.new_block("arc.assign.skip");

                    self.emit(Instruction::CondBr {
                        cond: is_not_null,
                        if_true: release_block,
                        if_false: skip_block,
                    });

                    self.set_insert_point(release_block);
                    self.emit(Instruction::Release { ptr: old_val });
                    self.emit(Instruction::Br { target: skip_block });

                    self.set_insert_point(skip_block);

                    // retain the new one
                    self.emit(Instruction::Retain { ptr: val_reg });
                }

                self.emit(Instruction::Store { ty: var_ty, ptr: ptr_reg, value: val_reg });
                val_reg
            }

            Expr::Cast { value, target_type, .. } => {
                let val_reg = self.lower_expr(value);
                let dest = self.new_reg();
                let ir_ty = self.map_type(target_type);

                if ir_ty == IrType::FatPtr {
                    let trait_name = match target_type {
                        Type::Custom(n) => n.clone(),
                        _ => "UnknownTrait".to_string(),
                    };

                    let mut source_struct = "UnknownStruct".to_string();
                    if let Expr::Variable(v) = &**value {
                        if let Some((_, var_ty, _)) = self.resolve_variable(&v.lexeme) {
                            if let IrType::Struct(s_name, _) = var_ty {
                                source_struct = s_name.clone();
                            }
                        }
                    }

                    let vtable_name = format!("vtable_{}_{}", source_struct, trait_name);

                    self.emit(Instruction::MakeFatPtr {
                        dest,
                        data_ptr: val_reg,
                        vtable_name,
                    });
                } else {
                    self.emit(Instruction::Cast { dest, value: val_reg, target_ty: ir_ty });
                }
                dest
            }

            Expr::Unary { operator, right } => {
                match operator.token_type {
                    TokenType::PlusPlus | TokenType::MinusMinus => {
                        let ptr = self.lower_lvalue(right);
                        let val = self.new_reg();
                        self.emit(Instruction::Load { dest: val, ty: IrType::I64, src_ptr: ptr });

                        let one = self.new_reg();
                        self.emit(Instruction::ConstInt { dest: one, value: 1 });

                        let new_val = self.new_reg();
                        if operator.token_type == TokenType::PlusPlus {
                            self.emit(Instruction::Add { dest: new_val, left: val, right: one });
                        } else {
                            self.emit(Instruction::Sub { dest: new_val, left: val, right: one });
                        }

                        self.emit(Instruction::Store { ty: IrType::I64, ptr: ptr, value: new_val });

                        new_val
                    }

                    TokenType::Minus => {
                        let val = self.lower_expr(right);
                        let zero = self.new_reg();
                        self.emit(Instruction::ConstInt { dest: zero, value: 0 });

                        let new_val = self.new_reg();
                        self.emit(Instruction::Sub { dest: new_val, left: zero, right: val });

                        new_val
                    }

                    TokenType::Bang => {
                        let val = self.lower_expr(right);
                        let zero = self.new_reg();
                        self.emit(Instruction::ConstInt { dest: zero, value: 0 });

                        let new_val = self.new_reg();
                        self.emit(Instruction::CmpEq { dest: new_val, left: val, right: zero });

                        new_val
                    }
                    _ => unimplemented!("Unary operator not supported in IR yet."),
                }
            }

            Expr::Binary { left, operator, right } => {
                let left_reg = self.lower_expr(left);
                let right_reg = self.lower_expr(right);
                let dest = self.new_reg();

                let inst = match operator.token_type {
                    TokenType::Plus => Instruction::Add { dest, left: left_reg, right: right_reg },
                    TokenType::Minus => Instruction::Sub { dest, left: left_reg, right: right_reg },
                    TokenType::Star => Instruction::Mul { dest, left: left_reg, right: right_reg },
                    TokenType::Slash => Instruction::Div { dest, left: left_reg, right: right_reg },
                    TokenType::Percent =>
                        Instruction::Mod { dest, left: left_reg, right: right_reg },
                    TokenType::EqualEqual =>
                        Instruction::CmpEq { dest, left: left_reg, right: right_reg },
                    TokenType::BangEqual =>
                        Instruction::CmpNeq { dest, left: left_reg, right: right_reg },
                    TokenType::Less =>
                        Instruction::CmpLt { dest, left: left_reg, right: right_reg },
                    TokenType::LessEqual =>
                        Instruction::CmpLe { dest, left: left_reg, right: right_reg },
                    TokenType::Greater =>
                        Instruction::CmpGt { dest, left: left_reg, right: right_reg },
                    TokenType::GreaterEqual =>
                        Instruction::CmpGe { dest, left: left_reg, right: right_reg },
                    _ => panic!("Binary operator not supported in IR yet."),
                };

                self.emit(inst);
                dest
            }

            Expr::Grouping(inner) => self.lower_expr(inner),

            Expr::Call { callee, arguments, .. } => {
                let mut arg_regs = Vec::new();
                for arg in arguments {
                    arg_regs.push(self.lower_expr(arg));
                }

                let dest = self.new_reg();

                match &**callee {
                    Expr::Variable(t) => {
                        let func_name = t.lexeme.clone();

                        if let Some((ptr_reg, var_ty, _)) = self.resolve_variable(&func_name) {
                            let closure_ptr = self.new_reg();
                            self.emit(Instruction::Load {
                                dest: closure_ptr,
                                ty: var_ty,
                                src_ptr: ptr_reg,
                            });
                            let arg_types = arguments
                                .iter()
                                .map(|arg| self.infer_ir_type(arg))
                                .collect();
                            let ret_type = self.infer_ir_type(callee);
                            self.emit(Instruction::CallClosure {
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

                            self.emit(Instruction::Call {
                                dest,
                                func_name: final_func_name.clone(),
                                args: arg_regs,
                            });

                            if final_func_name == "free" && !arguments.is_empty() {
                                self.poison_expr(&arguments[0]);
                            }
                        }
                    }

                    Expr::Get { object, name } => {
                        let mut is_static = false;
                        if let Expr::Variable(var_name) = &**object {
                            if self.resolve_variable(&var_name.lexeme).is_none() {
                                is_static = true;
                            }
                        }

                        if is_static {
                            if let Expr::Variable(var_name) = &**object {
                                let class_name = match self.resolved_calls.get(name) {
                                    Some(CallType::Static(c)) | Some(CallType::Instance(c)) =>
                                        c.clone(),
                                    None => var_name.lexeme.clone(),
                                };
                                let mangled_func_name = format!("{}_{}", class_name, name.lexeme);

                                self.emit(Instruction::Call {
                                    dest,
                                    func_name: mangled_func_name,
                                    args: arg_regs,
                                });
                            }
                        } else {
                            let obj_reg = self.lower_expr(object);

                            let class_name_opt = match self.resolved_calls.get(name) {
                                Some(CallType::Static(c)) | Some(CallType::Instance(c)) =>
                                    Some(c.clone()),
                                None => None,
                            };

                            if let Some(class_name) = class_name_opt {
                                let mut method_args = vec![obj_reg];
                                method_args.extend(arg_regs.clone());

                                if self.traits.contains(&class_name) {
                                    let vtable_index = *self.trait_vtable_layout
                                        .get(&class_name)
                                        .and_then(|layout| layout.get(&name.lexeme))
                                        .expect("ERROR: Method not found in trait layout!");

                                    let (arg_types, ret_type) = if let Some(info) = self.user_types.get(&class_name) {
                                        if let Some((params, ret_t, _)) = info.methods.get(&name.lexeme) {
                                            let arg_types: Vec<IrType> = params
                                                .iter()
                                                .skip(1)
                                                .map(|t| self.map_type(t))
                                                .collect();
                                            let ret_type = self.map_type(ret_t);
                                            (arg_types, ret_type)
                                        } else {
                                            (
                                                arguments.iter().map(|arg| self.infer_ir_type(arg)).collect(),
                                                IrType::I64,
                                            )
                                        }
                                    } else {
                                        (
                                            arguments.iter().map(|arg| self.infer_ir_type(arg)).collect(),
                                            IrType::I64,
                                        )
                                    };

                                    self.emit(Instruction::DynamicCall {
                                        dest,
                                        vtable_index,
                                        fat_ptr: obj_reg,
                                        args: arg_regs,
                                        arg_types,
                                        ret_type,
                                    });
                                } else {
                                    let mangled_func_name = format!(
                                        "{}_{}",
                                        class_name,
                                        name.lexeme
                                    );
                                    self.emit(Instruction::Call {
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
                        let arg_types = arguments
                            .iter()
                            .map(|arg| self.infer_ir_type(arg))
                            .collect();
                        let ret_type = self.infer_ir_type(callee);
                        self.emit(Instruction::CallClosure {
                            dest,
                            closure_ptr,
                            args: arg_regs,
                            arg_types,
                            ret_type,
                        });
                    }
                }
                dest
            }

            Expr::Get { object, name } => {
                let obj_ptr = self.lower_expr(object);

                self.inject_null_check(obj_ptr, "Null pointer dereference on Property Read!");

                let field_index = self.property_indices
                    .get(name)
                    .copied()
                    .unwrap_or_else(|| {
                        eprintln!("ICE: property index missing for '{}'", name.lexeme);
                        0
                    });

                let idx_reg = self.new_reg();
                self.emit(Instruction::ConstInt { dest: idx_reg, value: field_index as i64 });

                let field_base_ty = self.deref_type(&self.infer_ir_type(object));
                let field_ptr = self.new_reg();
                self.emit(Instruction::GetElementPtr {
                    dest: field_ptr,
                    base_ty: field_base_ty,
                    base_ptr: obj_ptr,
                    indices: vec![idx_reg],
                });

                let dest = self.new_reg();
                self.emit(Instruction::Load { dest, ty: IrType::I64, src_ptr: field_ptr });
                dest
            }

            Expr::Set { object, name, value } => {
                let obj_ptr = self.lower_expr(object);
                self.inject_null_check(obj_ptr, "Null pointer dereference on Property Write!");

                let val_reg = self.lower_expr(value);

                let field_index = self.property_indices
                    .get(name)
                    .copied()
                    .unwrap_or_else(|| {
                        eprintln!("ICE: property index missing for '{}'", name.lexeme);
                        0
                    });

                let idx_reg = self.new_reg();
                self.emit(Instruction::ConstInt { dest: idx_reg, value: field_index as i64 });

                let field_base_ty = self.deref_type(&self.infer_ir_type(object));
                let field_ptr = self.new_reg();
                self.emit(Instruction::GetElementPtr {
                    dest: field_ptr,
                    base_ty: field_base_ty,
                    base_ptr: obj_ptr,
                    indices: vec![idx_reg],
                });

                self.emit(Instruction::Store { ty: IrType::I64, ptr: field_ptr, value: val_reg });
                val_reg
            }

            Expr::Logical { left, operator, right } => {
                let result_ptr = self.new_reg();
                self.emit(Instruction::Alloca {
                    dest: result_ptr,
                    name: "tmp_logic".into(),
                    ty: IrType::I64,
                });
                let left_reg = self.lower_expr(left);
                let right_block = self.new_block("logic.rhs");
                let end_block = self.new_block("logic.end");

                if operator.token_type == TokenType::LogicalAnd {
                    self.emit(Instruction::Store {
                        ty: IrType::I64,
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
                        ty: IrType::I64,
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
                    ty: IrType::I64,
                    ptr: result_ptr,
                    value: right_reg,
                });
                self.emit(Instruction::Br { target: end_block });
                self.set_insert_point(end_block);

                let final_val = self.new_reg();
                self.emit(Instruction::Load {
                    ty: IrType::I64,
                    dest: final_val,
                    src_ptr: result_ptr,
                });
                final_val
            }

            Expr::Array { elements, .. } => {
                let size_reg = self.new_reg();
                self.emit(Instruction::ConstInt { dest: size_reg, value: elements.len() as i64 });

                let arr_reg = self.new_reg();
                let element_ty = elements
                    .first()
                    .map(|el| self.infer_ir_type(el))
                    .unwrap_or(IrType::I64);

                self.emit(Instruction::AllocArray {
                    dest: arr_reg,
                    size: size_reg,
                    ty: element_ty.clone(),
                });

                for (i, el) in elements.iter().enumerate() {
                    let val_reg = self.lower_expr(el);
                    let idx_reg = self.new_reg();
                    self.emit(Instruction::ConstInt { dest: idx_reg, value: i as i64 });

                    let primitive_ty = self.deref_type(&element_ty);
                    let element_ptr = self.new_reg();
                    self.emit(Instruction::GetElementPtr {
                        dest: element_ptr,
                        base_ty: primitive_ty,
                        base_ptr: arr_reg,
                        indices: vec![idx_reg],
                    });

                    self.emit(Instruction::Store {
                        ty: element_ty.clone(),
                        ptr: element_ptr,
                        value: val_reg,
                    });
                }
                arr_reg
            }

            Expr::SubscriptGet { indexee, bracket: _, index } => {
                let base_ptr = self.lower_expr(indexee);
                self.inject_null_check(base_ptr, "Null pointer dereference (Array Read)!");

                let idx_val = self.lower_expr(index);

                self.inject_bounds_check(base_ptr, idx_val);

                let base_ty = self.deref_type(&self.infer_ir_type(indexee));

                let dest_ptr = self.new_reg();
                self.emit(Instruction::GetElementPtr {
                    dest: dest_ptr,
                    base_ty,
                    base_ptr,
                    indices: vec![idx_val],
                });

                let result_ty = self.infer_ir_type(indexee);
                let result = self.new_reg();
                self.emit(Instruction::Load { dest: result, ty: self.deref_type(&result_ty), src_ptr: dest_ptr });
                result
            }

            Expr::SubscriptSet { indexee, bracket: _, index, value } => {
                let base_ptr = self.lower_expr(indexee);
                self.inject_null_check(base_ptr, "Null pointer dereference (Array Write)!");

                let idx_val = self.lower_expr(index);
                self.inject_bounds_check(base_ptr, idx_val);

                let val_reg = self.lower_expr(value);

                let base_ty = self.deref_type(&self.infer_ir_type(indexee));

                let dest_ptr = self.new_reg();
                self.emit(Instruction::GetElementPtr {
                    dest: dest_ptr,
                    base_ty: base_ty.clone(),
                    base_ptr,
                    indices: vec![idx_val],
                });

                self.emit(Instruction::Store {
                    ty: base_ty.clone(),
                    ptr: dest_ptr,
                    value: val_reg,
                });

                val_reg
            }

            Expr::AddressOf { operand, .. } => self.lower_lvalue(operand),

            Expr::Dereference { operator: _, operand } => {
                let ptr_val = self.lower_expr(operand);

                self.inject_null_check(ptr_val, "Null pointer dereference!");

                let operand_ty = self.infer_ir_type(operand);
                let load_ty = self.deref_type(&operand_ty);

                let dest = self.new_reg();
                self.emit(Instruction::Load { dest, ty: load_ty, src_ptr: ptr_val });
                dest
            }

            Expr::DereferenceSet { ptr, value, .. } => {
                let target_address = self.lower_expr(ptr);

                self.inject_null_check(target_address, "Null pointer dereference on Assignment!");

                let val_reg = self.lower_expr(value);
                let target_ty = self.infer_ir_type(ptr);
                self.emit(Instruction::Store {
                    ty: self.deref_type(&target_ty),
                    value: val_reg,
                    ptr: target_address,
                });
                val_reg
            }

            Expr::New { class_name, arguments, .. } => {
                let obj_reg = self.new_reg();

                let real_name = match self.resolved_calls.get(class_name) {
                    Some(CallType::Static(c)) | Some(CallType::Instance(c)) => c.clone(),
                    None => class_name.lexeme.clone(),
                };

                let mut size_bytes = arguments.len() * 8;
                if size_bytes == 0 {
                    size_bytes = 8;
                }

                self.emit(Instruction::AllocStruct {
                    dest: obj_reg,
                    class_name: real_name,
                    size: size_bytes,
                });

                for (i, arg) in arguments.iter().enumerate() {
                    let arg_reg = self.lower_expr(arg);
                    let idx_reg = self.new_reg();
                    self.emit(Instruction::ConstInt { dest: idx_reg, value: i as i64 });

                    let field_ptr = self.new_reg();
                    self.emit(Instruction::GetElementPtr {
                        dest: field_ptr,
                        base_ty: IrType::Any,
                        base_ptr: obj_reg,
                        indices: vec![idx_reg],
                    });

                    self.emit(Instruction::Store {
                        ty: IrType::I64,
                        ptr: field_ptr,
                        value: arg_reg,
                    });
                }
                obj_reg
            }

            Expr::Match { value, cases, .. } => {
                let enum_ptr = self.lower_expr(value);

                let mut match_ty = IrType::Void;
                if let Some(first_case) = cases.first() {
                    if let Some(last_stmt) = first_case.body.last() {
                        if let Stmt::Expression(e) = last_stmt {
                            match_ty = self.infer_ir_type(e);
                        } else if let Stmt::Return { value: Some(e), .. } = last_stmt {
                            match_ty = self.infer_ir_type(e);
                        }
                    }
                }

                let result_reg = self.new_reg();
                if match_ty != IrType::Void {
                    self.emit(Instruction::Alloca {
                        dest: result_reg,
                        name: "match_res".into(),
                        ty: match_ty.clone(),
                    });
                }

                let is_enum_match = cases
                    .first()
                    .map_or(false, |c| { matches!(c.pattern, Expr::UnionPattern { .. }) });

                let actual_val_to_cmp = if is_enum_match {
                    let tag_idx = self.new_reg();
                    self.emit(Instruction::ConstInt { dest: tag_idx, value: 0 });
                    let tag_ptr = self.new_reg();
                    self.emit(Instruction::GetElementPtr {
                        dest: tag_ptr,
                        base_ty: IrType::I64,
                        base_ptr: enum_ptr,
                        indices: vec![tag_idx],
                    });
                    let actual_tag = self.new_reg();
                    self.emit(Instruction::Load {
                        dest: actual_tag,
                        ty: IrType::I64,
                        src_ptr: tag_ptr,
                    });
                    actual_tag
                } else {
                    enum_ptr
                };

                let end_block = self.new_block("match.end");

                for case in cases {
                    let case_block = self.new_block("match.case");
                    let next_case = self.new_block("match.next");

                    match &case.pattern {
                        Expr::UnionPattern { case_name, bindings } => {
                            let tag_id = *self.variant_tags
                                .get(&case_name.lexeme)
                                .unwrap_or_else(|| {
                                    eprintln!(
                                        "ICE: variant tag missing for '{}'",
                                        case_name.lexeme
                                    );
                                    &0
                                });

                            let expected_tag = self.new_reg();

                            self.emit(Instruction::ConstInt {
                                dest: expected_tag,
                                value: tag_id as i64,
                            });

                            let cmp_res = self.new_reg();
                            self.emit(Instruction::CmpEq {
                                dest: cmp_res,
                                left: actual_val_to_cmp,
                                right: expected_tag,
                            });

                            self.emit(Instruction::CondBr {
                                cond: cmp_res,
                                if_true: case_block,
                                if_false: next_case,
                            });

                            self.set_insert_point(case_block);
                            self.begin_scope();

                            for (i, binding) in bindings.iter().enumerate() {
                                let bind_idx = self.new_reg();
                                self.emit(Instruction::ConstInt {
                                    dest: bind_idx,
                                    value: (i + 1) as i64,
                                });

                                let bind_ptr = self.new_reg();
                                self.emit(Instruction::GetElementPtr {
                                    dest: bind_ptr,
                                    base_ty: IrType::I64,
                                    base_ptr: enum_ptr,
                                    indices: vec![bind_idx],
                                });

                                let bind_val = self.new_reg();
                                self.emit(Instruction::Load {
                                    dest: bind_val,
                                    ty: IrType::I64,
                                    src_ptr: bind_ptr,
                                });

                                let var_ptr = self.new_reg();
                                self.emit(Instruction::Alloca {
                                    dest: var_ptr,
                                    name: binding.lexeme.clone(),
                                    ty: IrType::I64,
                                });

                                self.emit(Instruction::Store {
                                    ty: IrType::I64,
                                    ptr: var_ptr,
                                    value: bind_val,
                                });

                                self.declare_variable(
                                    binding.lexeme.clone(),
                                    var_ptr,
                                    IrType::I64,
                                    false
                                );
                            }
                        }

                        Expr::Literal(_) => {
                            let expected_val = self.lower_expr(&case.pattern);
                            let cmp_res = self.new_reg();

                            self.emit(Instruction::CmpEq {
                                dest: cmp_res,
                                left: actual_val_to_cmp,
                                right: expected_val,
                            });

                            self.emit(Instruction::CondBr {
                                cond: cmp_res,
                                if_true: case_block,
                                if_false: next_case,
                            });

                            self.set_insert_point(case_block);
                            self.begin_scope();
                        }

                        Expr::WildcardPattern(_) => {
                            self.emit(Instruction::Br { target: case_block });
                            self.set_insert_point(case_block);
                            self.begin_scope();
                        }
                        _ => unimplemented!("Pattern not supported in Match instruction."),
                    }

                    let mut last_val = None;
                    for stmt in &case.body {
                        if let Stmt::Expression(e) = stmt {
                            last_val = Some(self.lower_expr(e));
                        } else if let Stmt::Return { value: Some(e), .. } = stmt {
                            last_val = Some(self.lower_expr(e));
                        } else {
                            self.lower_stmt(stmt);
                        }
                    }

                    if let Some(val) = last_val {
                        if match_ty != IrType::Void {
                            self.emit(Instruction::Store {
                                ty: match_ty.clone(),
                                ptr: result_reg,
                                value: val,
                            });
                        }
                    }

                    if !self.is_current_block_terminated() {
                        self.emit(Instruction::Br { target: end_block });
                    }

                    self.end_scope();
                    self.set_insert_point(next_case);
                }

                if !self.is_current_block_terminated() {
                    self.emit(Instruction::Br { target: end_block });
                }

                self.set_insert_point(end_block);

                if match_ty != IrType::Void {
                    let final_res = self.new_reg();

                    self.emit(Instruction::Load {
                        dest: final_res,
                        ty: match_ty.clone(),
                        src_ptr: result_reg,
                    });

                    final_res
                } else {
                    let dummy = self.new_reg();
                    self.emit(Instruction::ConstInt { dest: dummy, value: 0 });
                    dummy
                }
            }

            Expr::Lambda { params, body, return_type: stmt_ret_type, is_async: _ } => {
                let lambda_name = format!("__lambda_{}", self.lambda_count);
                self.lambda_count += 1;

                // 1. capture the env
                let mut captures = Vec::new();
                for scope in &self.scopes {
                    for (name, (reg, ty, is_arc)) in scope {
                        captures.push((name.clone(), *reg, ty.clone(), *is_arc));
                    }
                }

                let env_ptr = self.new_reg();
                if captures.is_empty() {
                    self.emit(Instruction::ConstInt { dest: env_ptr, value: 0 }); // no env
                } else {
                    let size_reg = self.new_reg();
                    self.emit(Instruction::ConstInt {
                        dest: size_reg,
                        value: captures.len() as i64,
                    });
                    self.emit(Instruction::AllocArray {
                        dest: env_ptr,
                        size: size_reg,
                        ty: IrType::I64,
                    });

                    // store each variable in the env
                    for (i, (_, reg, ty, _)) in captures.iter().enumerate() {
                        let idx_reg = self.new_reg();
                        self.emit(Instruction::ConstInt { dest: idx_reg, value: i as i64 });

                        let item_ptr = self.new_reg();
                        self.emit(Instruction::GetElementPtr {
                            dest: item_ptr,
                            base_ty: IrType::I64,
                            base_ptr: env_ptr,
                            indices: vec![idx_reg],
                        });

                        // read the value and cast to an i64 (to fit the array)
                        let current_val = self.new_reg();
                        self.emit(Instruction::Load {
                            dest: current_val,
                            ty: ty.clone(),
                            src_ptr: *reg,
                        });

                        let val_as_i64 = self.new_reg();
                        self.emit(Instruction::Cast {
                            dest: val_as_i64,
                            value: current_val,
                            target_ty: IrType::I64,
                        });

                        self.emit(Instruction::Store {
                            ty: IrType::I64,
                            ptr: item_ptr,
                            value: val_as_i64,
                        });
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
                    IrType::I64
                };

                let mut func_ir = FunctionIr {
                    name: lambda_name.clone(),
                    ret_type: ret_type.clone(),
                    args: ir_args.clone(),
                    blocks: Vec::new(),
                    is_variadic: false,
                };

                let old_blocks = std::mem::take(&mut self.blocks);
                let old_current = self.current_block.take();
                let old_scopes = std::mem::take(&mut self.scopes);

                // 3. build the entire body
                let entry_block = self.new_block("entry");
                self.set_insert_point(entry_block);
                self.begin_scope();

                let env_arg_reg = ir_args[0].0;

                // unpack the env and recreate the original variables
                if !captures.is_empty() {
                    for (i, (name, _, ty, is_arc)) in captures.iter().enumerate() {
                        let idx_reg = self.new_reg();
                        self.emit(Instruction::ConstInt { dest: idx_reg, value: i as i64 });

                        let item_ptr = self.new_reg();
                        self.emit(Instruction::GetElementPtr {
                            dest: item_ptr,
                            base_ty: IrType::I64,
                            base_ptr: env_arg_reg,
                            indices: vec![idx_reg],
                        });

                        let item_val_i64 = self.new_reg();
                        self.emit(Instruction::Load {
                            dest: item_val_i64,
                            ty: IrType::I64,
                            src_ptr: item_ptr,
                        });

                        let item_val = self.new_reg();
                        self.emit(Instruction::Cast {
                            dest: item_val,
                            value: item_val_i64,
                            target_ty: ty.clone(),
                        });

                        let local_ptr = self.new_reg();
                        self.emit(Instruction::Alloca {
                            dest: local_ptr,
                            name: name.clone(),
                            ty: ty.clone(),
                        });

                        self.emit(Instruction::Store {
                            ty: ty.clone(),
                            ptr: local_ptr,
                            value: item_val,
                        });

                        self.declare_variable(name.clone(), local_ptr, ty.clone(), *is_arc);
                        if *is_arc {
                            self.emit(Instruction::Retain { ptr: item_val });
                        }
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

                    self.declare_variable(param.name.lexeme.clone(), ptr_reg, ir_ty, is_arc);
                    if is_arc {
                        self.emit(Instruction::Retain { ptr: arg_reg });
                    }
                }

                for s in body {
                    self.lower_stmt(s);
                }

                self.end_scope();

                if !self.is_current_block_terminated() {
                    let zero = self.new_reg();
                    self.emit(Instruction::ConstInt { dest: zero, value: 0 });
                    self.emit(Instruction::Ret { value: Some(zero) });
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

                // 4. create the closure
                let dest = self.new_reg();
                self.emit(Instruction::MakeClosure { dest, fn_name: lambda_name, env_ptr });
                dest
            }

            _ => unimplemented!("Lowering for {} expression not implemented yet", expr),
        }
    }

    pub fn lower_stmt(&mut self, stmt: &Stmt) {
        match stmt {
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
                self.lower_expr(expr);
            }

            Stmt::Var { name, type_annotation, initializer } => {
                let ptr_reg = self.new_reg();

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

                self.declare_variable(name.lexeme.clone(), ptr_reg, ir_ty.clone(), is_arc);

                let zero = self.new_reg();
                self.emit(Instruction::ConstInt { dest: zero, value: 0 });
                self.emit(Instruction::Store { ty: ir_ty.clone(), ptr: ptr_reg, value: zero });

                if let Some(init_expr) = initializer {
                    let val_reg = self.lower_expr(init_expr);
                    self.emit(Instruction::Store { ty: ir_ty, ptr: ptr_reg, value: val_reg });
                }
            }

            Stmt::Return { value, .. } => {
                let ret_val = value.as_ref().map(|e| self.lower_expr(e));

                let active_scopes = self.scopes.clone();
                for scope in active_scopes.iter().rev() {
                    self.emit_releases_for_scope(scope);
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
                if !self.is_current_block_terminated() {
                    self.emit(Instruction::Br { target: merge_block });
                }

                if let Some(eb) = else_block {
                    self.set_insert_point(eb);
                    self.lower_stmt(else_branch.as_ref().unwrap());
                    if !self.is_current_block_terminated() {
                        self.emit(Instruction::Br { target: merge_block });
                    }
                }
                self.set_insert_point(merge_block);
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

            Stmt::ForIn { keyword: _, key, value, iterable, body } => {
                let arr_ptr = self.lower_expr(iterable);
                self.inject_null_check(arr_ptr, "Null pointer dereference no loop 'for in'!");

                let minus_one = self.new_reg();
                self.emit(Instruction::ConstInt { dest: minus_one, value: -1 });

                let size_ptr = self.new_reg();
                self.emit(Instruction::GetElementPtr {
                    dest: size_ptr,
                    base_ty: IrType::I64,
                    base_ptr: arr_ptr,
                    indices: vec![minus_one],
                });

                let len_reg = self.new_reg();
                self.emit(Instruction::Load { dest: len_reg, ty: IrType::I64, src_ptr: size_ptr });

                let counter_ptr = self.new_reg();
                self.emit(Instruction::Alloca {
                    dest: counter_ptr,
                    name: "for_in_counter".into(),
                    ty: IrType::I64,
                });

                let zero = self.new_reg();
                self.emit(Instruction::ConstInt { dest: zero, value: 0 });
                self.emit(Instruction::Store { ty: IrType::I64, ptr: counter_ptr, value: zero });

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
                self.emit(Instruction::CmpLt { dest: is_less, left: current_i, right: len_reg });
                self.emit(Instruction::CondBr {
                    cond: is_less,
                    if_true: body_block,
                    if_false: end_block,
                });

                self.set_insert_point(body_block);
                self.begin_scope();

                let item_val = self.new_reg();
                let item_ptr = self.new_reg();

                self.emit(Instruction::GetElementPtr {
                    dest: item_ptr,
                    base_ty: IrType::I64,
                    base_ptr: arr_ptr,
                    indices: vec![current_i],
                });

                self.emit(Instruction::Load { dest: item_val, ty: IrType::I64, src_ptr: item_ptr });

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
                    self.declare_variable(key.lexeme.clone(), index_var_ptr, IrType::I64, false);

                    let item_var_ptr = self.new_reg();
                    self.emit(Instruction::Alloca {
                        dest: item_var_ptr,
                        name: val_token.lexeme.clone(),
                        ty: IrType::I64,
                    });
                    self.emit(Instruction::Store {
                        ty: IrType::I64,
                        ptr: item_var_ptr,
                        value: item_val,
                    });
                    self.declare_variable(
                        val_token.lexeme.clone(),
                        item_var_ptr,
                        IrType::I64,
                        false
                    );
                } else {
                    let item_var_ptr = self.new_reg();
                    self.emit(Instruction::Alloca {
                        dest: item_var_ptr,
                        name: key.lexeme.clone(),
                        ty: IrType::I64,
                    });
                    self.emit(Instruction::Store {
                        ty: IrType::I64,
                        ptr: item_var_ptr,
                        value: item_val,
                    });
                    self.declare_variable(key.lexeme.clone(), item_var_ptr, IrType::I64, false);
                }

                for s in body {
                    self.lower_stmt(s);
                }

                if !self.is_current_block_terminated() {
                    let one = self.new_reg();
                    self.emit(Instruction::ConstInt { dest: one, value: 1 });

                    let latest_i = self.new_reg();
                    self.emit(Instruction::Load {
                        dest: latest_i,
                        ty: IrType::I64,
                        src_ptr: counter_ptr,
                    });

                    let next_i = self.new_reg();
                    self.emit(Instruction::Add { dest: next_i, left: latest_i, right: one });
                    self.emit(Instruction::Store {
                        ty: IrType::I64,
                        ptr: counter_ptr,
                        value: next_i,
                    });

                    self.emit(Instruction::Br { target: cond_block });
                }

                self.end_scope();
                self.set_insert_point(end_block);
            }

            Stmt::Function { name, params, body, return_type: stmt_ret_type, type_params, .. } => {
                if !type_params.is_empty() {
                    return;
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
                    ret_type,
                    args: ir_args.clone(),
                    blocks: Vec::new(),
                    is_variadic: false,
                };

                let old_blocks = std::mem::take(&mut self.blocks);
                let old_current = self.current_block.take();

                let entry_block = self.new_block("entry");
                self.set_insert_point(entry_block);
                self.begin_scope();

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

                    self.declare_variable(param.name.lexeme.clone(), ptr_reg, ir_ty, is_arc);

                    if is_arc {
                        self.emit(Instruction::Retain { ptr: arg_reg });
                    }
                }

                if let Some(stmts) = body {
                    for s in stmts {
                        self.lower_stmt(s);
                    }
                }

                self.end_scope();

                if !self.is_current_block_terminated() {
                    self.emit(Instruction::Ret { value: None });
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
            }

            Stmt::Struct { .. } => {}

            Stmt::Impl { target_type, trait_name, methods, .. } => {
                if let Type::Custom(struct_name) = target_type {
                    if let Some(Type::Custom(t_name)) = trait_name {
                        let vtable_id = format!("vtable_{}_{}", struct_name, t_name);

                        if let Some(layout) = self.trait_vtable_layout.get(t_name) {
                            let mut ordered_methods = vec!["".to_string(); layout.len()];

                            for method in methods {
                                if let Stmt::Function { name: m_name, .. } = method {
                                    if let Some(&idx) = layout.get(&m_name.lexeme) {
                                        ordered_methods[idx] = format!(
                                            "{}_{}",
                                            struct_name,
                                            m_name.lexeme
                                        );
                                    }
                                }
                            }
                            self.vtables.insert(vtable_id, ordered_methods);
                        }
                    }

                    for method in methods {
                        if
                            let Stmt::Function {
                                name: m_name,
                                params,
                                body,
                                is_async,
                                return_type,
                                is_abstract,
                                is_public,
                                ..
                            } = method
                        {
                            let mangled_name = format!("{}_{}", struct_name, m_name.lexeme);

                            let mangled_func = Stmt::Function {
                                name: Token::synthetic(TokenType::Identifier, &mangled_name),
                                params: params.clone(),
                                body: body.clone(),
                                is_async: *is_async,
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

            Stmt::Enum { name, cases, .. } => {
                for (tag_id, case) in cases.iter().enumerate() {
                    self.variant_tags.insert(case.name.lexeme.clone(), tag_id);
                    let mut ir_args = Vec::new();
                    for _ in &case.parameters {
                        ir_args.push((self.new_reg(), IrType::I64));
                    }

                    let func_name = format!("{}_{}", name.lexeme, case.name.lexeme);

                    let mut func_ir = FunctionIr {
                        name: func_name,
                        ret_type: IrType::I64,
                        args: ir_args.clone(),
                        blocks: Vec::new(),
                        is_variadic: false,
                    };

                    let old_blocks = std::mem::take(&mut self.blocks);
                    let old_current = self.current_block.take();

                    let entry_block = self.new_block("entry");
                    self.set_insert_point(entry_block);

                    let obj_reg = self.new_reg();
                    let size_reg = self.new_reg();
                    self.emit(Instruction::ConstInt {
                        dest: size_reg,
                        value: (1 + case.parameters.len()) as i64,
                    });
                    self.emit(Instruction::AllocArray {
                        dest: obj_reg,
                        size: size_reg,
                        ty: IrType::Any,
                    });

                    let tag_val = self.new_reg();
                    self.emit(Instruction::ConstInt { dest: tag_val, value: tag_id as i64 });
                    let tag_idx = self.new_reg();
                    self.emit(Instruction::ConstInt { dest: tag_idx, value: 0 });
                    let tag_ptr = self.new_reg();
                    self.emit(Instruction::GetElementPtr {
                        dest: tag_ptr,
                        base_ty: IrType::I64,
                        base_ptr: obj_reg,
                        indices: vec![tag_idx],
                    });
                    self.emit(Instruction::Store { ty: IrType::I64, ptr: tag_ptr, value: tag_val });

                    for (i, (arg_reg, _)) in ir_args.iter().enumerate() {
                        let p_idx = self.new_reg();
                        self.emit(Instruction::ConstInt { dest: p_idx, value: (i + 1) as i64 });
                        let p_ptr = self.new_reg();
                        self.emit(Instruction::GetElementPtr {
                            dest: p_ptr,
                            base_ty: IrType::I64,
                            base_ptr: obj_reg,
                            indices: vec![p_idx],
                        });
                        self.emit(Instruction::Store {
                            ty: IrType::I64,
                            ptr: p_ptr,
                            value: *arg_reg,
                        });
                    }

                    self.emit(Instruction::Ret { value: Some(obj_reg) });

                    let mut sorted_blocks: Vec<_> = self.blocks
                        .drain()
                        .map(|(_, b)| b)
                        .collect();
                    sorted_blocks.sort_by_key(|b| b.id.0);
                    func_ir.blocks = sorted_blocks;

                    self.functions.push(func_ir);
                    self.blocks = old_blocks;
                    self.current_block = old_current;
                }
            }

            Stmt::ArrayDestructuring { keyword: _, bindings, initializer } => {
                let array_ptr = self.lower_expr(initializer);

                for (i, binding) in bindings.iter().enumerate() {
                    let idx_reg = self.new_reg();
                    self.emit(Instruction::ConstInt { dest: idx_reg, value: i as i64 });

                    let element_ptr = self.new_reg();
                    self.emit(Instruction::GetElementPtr {
                        dest: element_ptr,
                        base_ty: IrType::I64,
                        base_ptr: array_ptr,
                        indices: vec![idx_reg],
                    });

                    let val_reg = self.new_reg();
                    self.emit(Instruction::Load {
                        dest: val_reg,
                        ty: IrType::I64,
                        src_ptr: element_ptr,
                    });

                    let local_ptr = self.new_reg();
                    self.emit(Instruction::Alloca {
                        dest: local_ptr,
                        name: binding.lexeme.clone(),
                        ty: IrType::I64,
                    });

                    self.emit(Instruction::Store {
                        ty: IrType::I64,
                        ptr: local_ptr,
                        value: val_reg,
                    });

                    self.declare_variable(binding.lexeme.clone(), local_ptr, IrType::I64, false);
                }
            }

            _ => unimplemented!("Lowering for {stmt} not supported yet."),
        }
    }
}

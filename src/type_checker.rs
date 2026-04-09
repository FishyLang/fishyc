use std::collections::{ HashMap, HashSet };
use crate::ast::{ Expr, Stmt, Type, MatchCase, GenericParam };
use crate::token::{ Token, TokenType, Literal };

#[derive(Debug, Clone, PartialEq)]
pub enum VarState {
    Valid,
    Dropped(usize), // the line where the `drop` was called
}

#[derive(Debug, Clone)]
pub struct TypeError {
    pub token: Token,
    pub message: String,
    pub hints: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct StructInfo {
    pub type_params: Vec<GenericParam>,
    // (index, type, is_public)
    pub fields: HashMap<String, (usize, Type, bool)>,
    pub methods: HashMap<String, (Vec<Type>, Type, bool)>,
    pub static_methods: HashMap<String, (Vec<Type>, Type, bool)>,
    pub case_names: Vec<String>,
}

pub struct TypeChecker {
    scopes: Vec<HashMap<String, (Type, VarState)>>,
    pub errors: Vec<TypeError>,
    current_return_type: Option<Type>,

    pub aliases: HashMap<String, Type>,
    pub user_types: HashMap<String, StructInfo>,
    pub struct_templates: HashMap<String, Stmt>,

    pub impl_templates: HashMap<String, Vec<Stmt>>,
    pub instantiations: Vec<Stmt>,
    pub instantiated_types: HashSet<String>,

    pub traits: HashSet<String>,
    pub trait_vtable_layout: HashMap<String, HashMap<String, usize>>,

    pub property_indices: HashMap<Token, usize>,
    pub resolved_constructors: HashMap<Token, String>,
    pub resolved_methods: HashMap<Token, String>,

    pub struct_sizes: HashMap<String, usize>,

    current_struct: Option<String>,
    is_in_async: bool,
    is_in_static: bool,
    loop_depth: usize,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            errors: Vec::new(),
            current_return_type: None,

            aliases: HashMap::new(),
            user_types: HashMap::new(),
            struct_templates: HashMap::new(),

            impl_templates: HashMap::new(),
            instantiations: Vec::new(),
            instantiated_types: HashSet::new(),

            traits: HashSet::new(),
            trait_vtable_layout: HashMap::new(),

            property_indices: HashMap::new(),
            resolved_constructors: HashMap::new(),
            resolved_methods: HashMap::new(),

            struct_sizes: HashMap::new(),

            current_struct: None,
            is_in_async: false,
            is_in_static: false,
            loop_depth: 0,
        }
    }

    pub fn pre_scan(&mut self, statements: &[Stmt]) {
        for stmt in statements {
            match stmt {
                Stmt::Alias { name, target } => {
                    let resolved = self.resolve_type(target);
                    self.aliases.insert(name.lexeme.clone(), resolved);
                }

                Stmt::ExternFunction { name, params, return_type, is_variadic } => {
                    let p_types: Vec<Type> = params
                        .iter()
                        .map(|p| {
                            if let Some(t) = &p.type_annotation {
                                self.resolve_type(t)
                            } else {
                                self.error(
                                    &p.name,
                                    "External function parameters require explicit typing."
                                );
                                Type::Void
                            }
                        })
                        .collect();

                    let r_type = return_type
                        .as_ref()
                        .map(|t| self.resolve_type(t))
                        .unwrap_or(Type::Void);

                    if let Some(global_scope) = self.scopes.first_mut() {
                        global_scope.insert(name.lexeme.clone(), (
                            Type::Function(p_types, Box::new(r_type), *is_variadic),
                            VarState::Valid,
                        ));
                    }
                }

                Stmt::Struct { name, type_params, fields } => {
                    if !type_params.is_empty() {
                        self.struct_templates.insert(name.lexeme.clone(), stmt.clone());
                    } else {
                        let mut info = StructInfo {
                            type_params: vec![],
                            fields: HashMap::new(),
                            methods: HashMap::new(),
                            static_methods: HashMap::new(),
                            case_names: vec![],
                        };
                        for (idx, (f_name, f_type, is_pub)) in fields.iter().enumerate() {
                            info.fields.insert(f_name.lexeme.clone(), (
                                idx,
                                self.resolve_type(f_type),
                                *is_pub,
                            ));
                        }
                        self.user_types.insert(name.lexeme.clone(), info);
                    }
                }

                Stmt::Impl { target_type, methods, .. } => {
                    let original_name = match target_type {
                        Type::Generic(name, _) => name.clone(),
                        Type::Custom(name) => name.clone(),
                        _ => String::new(),
                    };

                    if self.struct_templates.contains_key(&original_name) {
                        self.impl_templates.insert(original_name, methods.clone());
                    }

                    let resolved_target = self.resolve_type(target_type);
                    if let Type::Custom(struct_name) = resolved_target {
                        if self.struct_templates.contains_key(&struct_name) {
                            self.impl_templates.insert(struct_name.clone(), methods.clone());
                        }

                        for method in methods {
                            if
                                let Stmt::Function {
                                    name: m_name,
                                    params,
                                    return_type,
                                    is_public,
                                    ..
                                } = method
                            {
                                let is_instance = params
                                    .first()
                                    .map_or(
                                        false,
                                        |p|
                                            p.name.lexeme == "self" ||
                                            p.name.token_type == TokenType::This
                                    );

                                let p_ts: Vec<Type> = params
                                    .iter()
                                    .map(|p| {
                                        if p.name.lexeme == "self" {
                                            Type::MutReference(
                                                Box::new(Type::Custom(struct_name.clone()))
                                            )
                                        } else {
                                            self.resolve_type(
                                                p.type_annotation.as_ref().unwrap_or(&Type::Void)
                                            )
                                        }
                                    })
                                    .collect();

                                let ret_t = return_type
                                    .as_ref()
                                    .map(|t| self.resolve_type(t))
                                    .unwrap_or(Type::Void);

                                if let Some(info) = self.user_types.get_mut(&struct_name) {
                                    if is_instance {
                                        info.methods.insert(m_name.lexeme.clone(), (
                                            p_ts,
                                            ret_t,
                                            *is_public,
                                        ));
                                    } else {
                                        info.static_methods.insert(m_name.lexeme.clone(), (
                                            p_ts.clone(),
                                            ret_t.clone(),
                                            *is_public,
                                        ));

                                        let mangled_name = format!(
                                            "{}_{}",
                                            struct_name,
                                            m_name.lexeme
                                        );
                                        if let Some(global_scope) = self.scopes.first_mut() {
                                            global_scope.insert(mangled_name, (
                                                Type::Function(p_ts, Box::new(ret_t), false),
                                                VarState::Valid,
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                Stmt::Enum { name, type_params, .. } => {
                    self.user_types.entry(name.lexeme.clone()).or_insert(StructInfo {
                        type_params: type_params.clone(),
                        fields: HashMap::new(),
                        methods: HashMap::new(),
                        static_methods: HashMap::new(),
                        case_names: vec![],
                    });
                }

                Stmt::Trait { name, type_params, methods, .. } => {
                    self.traits.insert(name.lexeme.clone());

                    let mut info = StructInfo {
                        type_params: type_params.clone(),
                        fields: HashMap::new(),
                        methods: HashMap::new(),
                        static_methods: HashMap::new(),
                        case_names: vec![],
                    };

                    let mut method_layout = HashMap::new();
                    let mut method_index = 0;

                    for method in methods {
                        if let Stmt::Function { name: m_name, params, return_type, .. } = method {
                            method_layout.insert(m_name.lexeme.clone(), method_index);
                            method_index += 1;

                            let p_ts: Vec<Type> = params
                                .iter()
                                .map(|p| {
                                    if p.name.lexeme == "self" {
                                        Type::MutReference(
                                            Box::new(Type::Custom(name.lexeme.clone()))
                                        )
                                    } else {
                                        self.resolve_type(
                                            p.type_annotation.as_ref().unwrap_or(&Type::Void)
                                        )
                                    }
                                })
                                .collect();

                            let ret_t = return_type
                                .as_ref()
                                .map(|t| self.resolve_type(t))
                                .unwrap_or(Type::Void);
                            info.methods.insert(m_name.lexeme.clone(), (p_ts, ret_t, true));
                        }
                    }
                    self.trait_vtable_layout.insert(name.lexeme.clone(), method_layout);
                    self.user_types.insert(name.lexeme.clone(), info);
                }

                _ => {}
            }
        }
    }

    pub fn check(&mut self, statements: &[Stmt]) {
        self.pre_scan(statements);
        for stmt in statements {
            self.check_stmt(stmt);
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn mark_as_dropped(&mut self, name: &str, line: usize) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some((_, state)) = scope.get_mut(name) {
                *state = VarState::Dropped(line);
                return;
            }
        }
    }

    fn declare_variable(&mut self, name: &Token, var_type: Type) {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(&name.lexeme) {
                self.error(name, "Variable already declared in scope.");
            } else {
                scope.insert(name.lexeme.clone(), (var_type, VarState::Valid));
            }
        }
    }

    fn resolve_variable(&self, name: &Token) -> Option<(Type, VarState)> {
        for scope in self.scopes.iter().rev() {
            if let Some((ty, state)) = scope.get(&name.lexeme) {
                return Some((ty.clone(), state.clone()));
            }
        }
        None
    }

    pub fn resolve_type(&mut self, ty: &Type) -> Type {
        match ty {
            Type::Custom(name) => {
                let aliased_type = self.aliases.get(name).cloned();
                if let Some(real_type) = aliased_type {
                    return self.resolve_type(&real_type);
                }
                ty.clone()
            }
            Type::Pointer(inner) => Type::Pointer(Box::new(self.resolve_type(inner))),
            Type::Reference(inner) => Type::Reference(Box::new(self.resolve_type(inner))),
            Type::MutReference(inner) => Type::MutReference(Box::new(self.resolve_type(inner))),
            Type::Slice(inner) => Type::Slice(Box::new(self.resolve_type(inner))),
            Type::Array(size, inner) => Type::Array(*size, Box::new(self.resolve_type(inner))),
            Type::Tuple(types) =>
                Type::Tuple(
                    types
                        .iter()
                        .map(|t| self.resolve_type(t))
                        .collect()
                ),
            Type::Function(params, ret, is_var) => {
                Type::Function(
                    params
                        .iter()
                        .map(|t| self.resolve_type(t))
                        .collect(),
                    Box::new(self.resolve_type(ret)),
                    *is_var
                )
            }
            Type::Generic(name, args) => {
                let resolved_args: Vec<Type> = args
                    .iter()
                    .map(|a| self.resolve_type(a))
                    .collect();
                let mangled_name = self.mangle_generic_name(name, &resolved_args);

                if self.user_types.contains_key(&mangled_name) {
                    return Type::Custom(mangled_name);
                }

                if
                    let Some(Stmt::Struct { type_params, fields, .. }) = self.struct_templates
                        .get(name)
                        .cloned()
                {
                    if type_params.len() != resolved_args.len() {
                        self.error(
                            &Token::synthetic(TokenType::Identifier, name),
                            &format!(
                                "Generic type '{}' receives {} parameter(s), but got {} instead.",
                                name,
                                type_params.len(),
                                resolved_args.len()
                            )
                        );
                        return Type::Void;
                    }

                    let mut substitutions = HashMap::new();
                    for (i, param) in type_params.iter().enumerate() {
                        substitutions.insert(param.name.lexeme.clone(), resolved_args[i].clone());
                    }

                    let mut new_info = StructInfo {
                        type_params: vec![],
                        fields: HashMap::new(),
                        methods: HashMap::new(),
                        static_methods: HashMap::new(),
                        case_names: vec![],
                    };

                    for (index, (f_name, f_type, is_pub)) in fields.iter().enumerate() {
                        let final_type = self.substitute_type(f_type, &substitutions);
                        let final_resolved = self.resolve_type(&final_type);
                        new_info.fields.insert(f_name.lexeme.clone(), (
                            index,
                            final_resolved,
                            *is_pub,
                        ));
                    }

                    let template_type_args: Vec<Type> = type_params
                        .iter()
                        .map(|p| Type::Custom(p.name.lexeme.clone()))
                        .collect();
                    let template_mangled_name = self.mangle_generic_name(name, &template_type_args);

                    if
                        let Some(generic_struct_info) = self.user_types
                            .get(&template_mangled_name)
                            .cloned()
                    {
                        for (m_name, (m_params, m_ret, is_pub)) in generic_struct_info.methods {
                            let sub_params = m_params
                                .iter()
                                .map(|p| self.substitute_type(p, &substitutions))
                                .collect();
                            let sub_ret = self.substitute_type(&m_ret, &substitutions);
                            new_info.methods.insert(m_name, (sub_params, sub_ret, is_pub));
                        }

                        for (
                            m_name,
                            (m_params, m_ret, is_pub),
                        ) in generic_struct_info.static_methods {
                            let sub_params: Vec<Type> = m_params
                                .iter()
                                .map(|p| self.substitute_type(p, &substitutions))
                                .collect();
                            let sub_ret = self.substitute_type(&m_ret, &substitutions);
                            new_info.static_methods.insert(m_name.clone(), (
                                sub_params.clone(),
                                sub_ret.clone(),
                                is_pub,
                            ));

                            let global_mangled = format!("{}_{}", mangled_name, m_name);
                            if let Some(global_scope) = self.scopes.first_mut() {
                                global_scope.insert(global_mangled, (
                                    Type::Function(sub_params, Box::new(sub_ret), false),
                                    VarState::Valid,
                                ));
                            }
                        }
                    }

                    self.user_types.insert(mangled_name.clone(), new_info);

                    if !self.instantiated_types.contains(&mangled_name) {
                        self.instantiated_types.insert(mangled_name.clone());

                        if let Some(methods_ast) = self.impl_templates.get(name).cloned() {
                            let mut instantiated_methods_ast = Vec::new();

                            for method in methods_ast {
                                instantiated_methods_ast.push(
                                    self.monomorphize_stmt(&method, &substitutions)
                                );
                            }

                            let concrete_impl_ast = Stmt::Impl {
                                target_type: Type::Custom(mangled_name.clone()),
                                methods: instantiated_methods_ast,
                                keyword: Token::synthetic(TokenType::Identifier, "impl"),
                                type_params: vec![],
                                trait_name: None,
                            };

                            self.instantiations.push(concrete_impl_ast);
                        }
                    }

                    return Type::Custom(mangled_name);
                }

                self.error(
                    &Token::synthetic(TokenType::Identifier, name),
                    &format!("Generic type '{}' was not found.", name)
                );
                Type::Void
            }
            Type::Union(variants) =>
                Type::Union(
                    variants
                        .iter()
                        .map(|v| self.resolve_type(v))
                        .collect()
                ),
            _ => ty.clone(),
        }
    }

    fn type_to_mangle_segment(&mut self, ty: &Type) -> String {
        match self.resolve_type(ty) {
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
            Type::Bool => "bool".into(),
            Type::String => "str".into(),
            Type::Void => "void".into(),
            Type::Custom(name) => name,
            Type::Pointer(inner) => format!("ptr_{}", self.type_to_mangle_segment(&inner)),
            Type::Reference(inner) => format!("ref_{}", self.type_to_mangle_segment(&inner)),
            Type::MutReference(inner) => format!("mutref_{}", self.type_to_mangle_segment(&inner)),
            Type::Array(size, inner) => {
                format!("arr{}_{}", size, self.type_to_mangle_segment(&inner))
            }
            Type::Slice(inner) => format!("slice_{}", self.type_to_mangle_segment(&inner)),
            Type::Tuple(ts) => format!("tuple{}", ts.len()),
            Type::Function(_, _, _) => "fn".into(),
            Type::Generic(name, args) => {
                let segments: Vec<String> = args
                    .iter()
                    .map(|a| self.type_to_mangle_segment(a))
                    .collect();
                format!("{}_{}", name, segments.join("_"))
            }
            Type::Union(_) => "union".into(),
        }
    }

    fn mangle_generic_name(&mut self, base_name: &str, type_args: &[Type]) -> String {
        let mut mangled = base_name.to_string();
        for arg in type_args {
            mangled.push('_');
            let segment = self.type_to_mangle_segment(arg);
            mangled.push_str(&segment);
        }
        mangled
    }

    pub fn substitute_type(&self, ty: &Type, substitutions: &HashMap<String, Type>) -> Type {
        match ty {
            Type::Custom(name) => {
                if let Some(replacement) = substitutions.get(name) {
                    return replacement.clone();
                }
                ty.clone()
            }
            Type::Pointer(inner) =>
                Type::Pointer(Box::new(self.substitute_type(inner, substitutions))),
            Type::Reference(inner) =>
                Type::Reference(Box::new(self.substitute_type(inner, substitutions))),
            Type::MutReference(inner) =>
                Type::MutReference(Box::new(self.substitute_type(inner, substitutions))),
            Type::Array(size, inner) =>
                Type::Array(*size, Box::new(self.substitute_type(inner, substitutions))),
            Type::Slice(inner) => Type::Slice(Box::new(self.substitute_type(inner, substitutions))),
            Type::Tuple(types) =>
                Type::Tuple(
                    types
                        .iter()
                        .map(|t| self.substitute_type(t, substitutions))
                        .collect()
                ),
            Type::Function(params, ret, is_var) => {
                Type::Function(
                    params
                        .iter()
                        .map(|t| self.substitute_type(t, substitutions))
                        .collect(),
                    Box::new(self.substitute_type(ret, substitutions)),
                    *is_var
                )
            }
            Type::Generic(name, args) => {
                let new_args = args
                    .iter()
                    .map(|a| self.substitute_type(a, substitutions))
                    .collect();
                Type::Generic(name.clone(), new_args)
            }
            _ => ty.clone(),
        }
    }

    pub fn monomorphize_stmt(&mut self, stmt: &Stmt, subs: &HashMap<String, Type>) -> Stmt {
        match stmt {
            Stmt::Function {
                name,
                params,
                return_type,
                body,
                is_async,
                is_public,
                type_params,
                is_abstract,
            } => {
                let new_params = params
                    .iter()
                    .map(|p| {
                        let mut new_p = p.clone();
                        if let Some(t) = &p.type_annotation {
                            new_p.type_annotation = Some(self.substitute_type(t, subs));
                        }
                        new_p
                    })
                    .collect();

                let new_ret = return_type.as_ref().map(|t| self.substitute_type(t, subs));
                let new_body = body.as_ref().map(|b|
                    b
                        .iter()
                        .map(|s| self.monomorphize_stmt(s, subs))
                        .collect()
                );

                Stmt::Function {
                    name: name.clone(),
                    params: new_params,
                    return_type: new_ret,
                    body: new_body,
                    is_async: *is_async,
                    is_public: *is_public,
                    type_params: type_params.clone(),
                    is_abstract: *is_abstract,
                }
            }
            Stmt::Block(stmts) => {
                Stmt::Block(
                    stmts
                        .iter()
                        .map(|s| self.monomorphize_stmt(s, subs))
                        .collect()
                )
            }
            Stmt::Var { name, type_annotation, initializer } => {
                Stmt::Var {
                    name: name.clone(),
                    type_annotation: type_annotation
                        .as_ref()
                        .map(|t| self.substitute_type(t, subs)),
                    initializer: initializer.as_ref().map(|e| self.monomorphize_expr(e, subs)),
                }
            }
            Stmt::Expression(expr) => Stmt::Expression(self.monomorphize_expr(expr, subs)),
            Stmt::Return { keyword, value } => {
                Stmt::Return {
                    keyword: keyword.clone(),
                    value: value.as_ref().map(|e| self.monomorphize_expr(e, subs)),
                }
            }
            Stmt::If { condition, then_branch, else_branch, if_token } => {
                Stmt::If {
                    condition: self.monomorphize_expr(condition, subs),
                    then_branch: Box::new(self.monomorphize_stmt(then_branch, subs)),
                    else_branch: else_branch
                        .as_ref()
                        .map(|b| Box::new(self.monomorphize_stmt(b, subs))),
                    if_token: if_token.clone(),
                }
            }
            Stmt::While { condition, body, while_token } => {
                Stmt::While {
                    condition: self.monomorphize_expr(condition, subs),
                    body: Box::new(self.monomorphize_stmt(body, subs)),
                    while_token: while_token.clone(),
                }
            }
            _ => stmt.clone(),
        }
    }

    pub fn monomorphize_expr(&mut self, expr: &Expr, subs: &HashMap<String, Type>) -> Expr {
        match expr {
            Expr::New { keyword, class_name, type_args, arguments, paren } => {
                Expr::New {
                    keyword: keyword.clone(),
                    class_name: class_name.clone(),
                    type_args: type_args
                        .iter()
                        .map(|t| self.substitute_type(t, subs))
                        .collect(),
                    arguments: arguments
                        .iter()
                        .map(|e| self.monomorphize_expr(e, subs))
                        .collect(),
                    paren: paren.clone(),
                }
            }
            Expr::Cast { value, target_type, operator } => {
                Expr::Cast {
                    value: Box::new(self.monomorphize_expr(value, subs)),
                    target_type: self.substitute_type(target_type, subs),
                    operator: operator.clone(),
                }
            }
            Expr::Call { callee, arguments, paren } => {
                Expr::Call {
                    callee: Box::new(self.monomorphize_expr(callee, subs)),
                    arguments: arguments
                        .iter()
                        .map(|e| self.monomorphize_expr(e, subs))
                        .collect(),
                    paren: paren.clone(),
                }
            }
            Expr::Get { object, name } => {
                Expr::Get {
                    object: Box::new(self.monomorphize_expr(object, subs)),
                    name: name.clone(),
                }
            }
            Expr::Set { object, name, value } => {
                Expr::Set {
                    object: Box::new(self.monomorphize_expr(object, subs)),
                    name: name.clone(),
                    value: Box::new(self.monomorphize_expr(value, subs)),
                }
            }
            Expr::Assign { name, value } => {
                Expr::Assign {
                    name: name.clone(),
                    value: Box::new(self.monomorphize_expr(value, subs)),
                }
            }
            Expr::Binary { left, operator, right } => {
                Expr::Binary {
                    left: Box::new(self.monomorphize_expr(left, subs)),
                    operator: operator.clone(),
                    right: Box::new(self.monomorphize_expr(right, subs)),
                }
            }
            Expr::Unary { operator, right } => {
                Expr::Unary {
                    operator: operator.clone(),
                    right: Box::new(self.monomorphize_expr(right, subs)),
                }
            }
            Expr::SubscriptGet { indexee, bracket, index } => {
                Expr::SubscriptGet {
                    indexee: Box::new(self.monomorphize_expr(indexee, subs)),
                    bracket: bracket.clone(),
                    index: Box::new(self.monomorphize_expr(index, subs)),
                }
            }
            Expr::SubscriptSet { indexee, bracket, index, value } => {
                Expr::SubscriptSet {
                    indexee: Box::new(self.monomorphize_expr(indexee, subs)),
                    bracket: bracket.clone(),
                    index: Box::new(self.monomorphize_expr(index, subs)),
                    value: Box::new(self.monomorphize_expr(value, subs)),
                }
            }
            Expr::Grouping(e) => Expr::Grouping(Box::new(self.monomorphize_expr(e, subs))),
            Expr::AddressOf { operator, operand } =>
                Expr::AddressOf {
                    operator: operator.clone(),
                    operand: Box::new(self.monomorphize_expr(operand, subs)),
                },
            Expr::Dereference { operator, operand } =>
                Expr::Dereference {
                    operator: operator.clone(),
                    operand: Box::new(self.monomorphize_expr(operand, subs)),
                },
            Expr::DereferenceSet { operator, ptr, value } =>
                Expr::DereferenceSet {
                    operator: operator.clone(),
                    ptr: Box::new(self.monomorphize_expr(ptr, subs)),
                    value: Box::new(self.monomorphize_expr(value, subs)),
                },
            _ => expr.clone(),
        }
    }

    fn error(&mut self, token: &Token, message: &str) {
        self.errors.push(TypeError {
            token: token.clone(),
            message: message.to_string(),
            hints: vec![],
        });
    }

    fn error_hint(&mut self, token: &Token, message: &str, hint: impl Into<String>) {
        self.errors.push(TypeError {
            token: token.clone(),
            message: message.to_string(),
            hints: vec![hint.into()],
        });
    }

    fn suggest_similar_var(&self, name: &str) -> Option<String> {
        let mut best: Option<(usize, &str)> = None;
        for scope in &self.scopes {
            for key in scope.keys() {
                if key == name {
                    continue;
                }
                let dist = Self::levenshtein(name, key);
                if dist <= 2 {
                    if best.map_or(true, |(b, _)| dist < b) {
                        best = Some((dist, key.as_str()));
                    }
                }
            }
        }
        best.map(|(_, s)| s.to_owned())
    }

    fn levenshtein(a: &str, b: &str) -> usize {
        let a: Vec<char> = a.chars().collect();
        let b: Vec<char> = b.chars().collect();
        let m = a.len();
        let n = b.len();
        let mut dp = vec![vec![0usize; n + 1]; m + 1];
        for i in 0..=m {
            dp[i][0] = i;
        }
        for j in 0..=n {
            dp[0][j] = j;
        }
        for i in 1..=m {
            for j in 1..=n {
                dp[i][j] = if a[i - 1] == b[j - 1] {
                    dp[i - 1][j - 1]
                } else {
                    1 + dp[i - 1][j].min(dp[i][j - 1]).min(dp[i - 1][j - 1])
                };
            }
        }
        dp[m][n]
    }

    fn is_assignable(&self, actual: &Type, expected: &Type) -> bool {
        if actual == expected {
            return true;
        }

        match (actual, expected) {
            (Type::String, Type::Pointer(inner)) if **inner == Type::U8 => true,

            (act, Type::Union(variants)) => variants.iter().any(|v| self.is_assignable(act, v)),

            | (Type::Custom(a_name), Type::MutReference(e_inner))
            | (Type::Custom(a_name), Type::Reference(e_inner))
            | (Type::Custom(a_name), Type::Pointer(e_inner)) => {
                self.is_assignable(&Type::Custom(a_name.clone()), e_inner)
            }

            (Type::MutReference(a_inner), Type::Reference(e_inner)) =>
                self.is_assignable(a_inner, e_inner),
            (Type::Reference(a_inner), Type::Reference(e_inner)) =>
                self.is_assignable(a_inner, e_inner),
            (Type::MutReference(a_inner), Type::MutReference(e_inner)) =>
                self.is_assignable(a_inner, e_inner),
            (Type::Pointer(a_inner), Type::Pointer(e_inner)) =>
                self.is_assignable(a_inner, e_inner),

            (Type::Generic(a_name, a_args), Type::Generic(e_name, e_args)) => {
                if a_name != e_name || a_args.len() != e_args.len() {
                    return false;
                }
                a_args
                    .iter()
                    .zip(e_args.iter())
                    .all(|(a, e)| self.is_assignable(a, e))
            }

            (Type::Array(a_size, a_inner), Type::Array(e_size, e_inner)) => {
                a_size == e_size && self.is_assignable(a_inner, e_inner)
            }

            (Type::Slice(a_inner), Type::Slice(e_inner)) => self.is_assignable(a_inner, e_inner),
            (Type::Array(_, a_inner), Type::Slice(e_inner)) => self.is_assignable(a_inner, e_inner),

            (Type::Void, Type::Pointer(_)) => true,

            _ => false,
        }
    }

    fn require_type(&mut self, actual: &Type, expected: &Type, token: &Token) {
        if !self.is_assignable(actual, expected) {
            let msg = format!(
                "Mismatched types: Expected '{}', but found '{}' instead.",
                expected,
                actual
            );

            if self.is_number_type(actual) && self.is_number_type(expected) {
                self.error_hint(token, &msg, format!("use `as {}` to cast the value", expected));
            } else {
                self.error(token, &msg);
            }
        }
    }

    fn check_expr_with_expectation(&mut self, expr: &Expr, expected: &Type, token: &Token) -> Type {
        if let Expr::Literal(Literal::Number(val)) = expr {
            if self.is_number_type(expected) {
                self.check_literal_bounds(*val, expected, token);
                return expected.clone();
            }
        }

        if let Expr::Literal(Literal::Integer(val)) = expr {
            if self.is_number_type(expected) {
                self.check_literal_bounds(*val as f64, expected, token);
                return expected.clone();
            }
        }

        let actual = self.check_expr(expr);
        self.require_type(&actual, expected, token);
        actual
    }

    fn is_number_type(&self, ty: &Type) -> bool {
        matches!(
            ty,
            Type::U8 |
                Type::U16 |
                Type::U32 |
                Type::U64 |
                Type::I8 |
                Type::I16 |
                Type::I32 |
                Type::I64 |
                Type::F16 |
                Type::F32 |
                Type::F64
        )
    }

    fn check_literal_bounds(&mut self, val: f64, expected: &Type, token: &Token) -> bool {
        let is_int = val.fract() == 0.0;

        match expected {
            Type::U8 => if is_int && val >= 0.0 && val <= 255.0 {
                true
            } else {
                self.emit_bounds_error(val, "u8", "0 to 255", "u16", token);
                false
            }
            Type::U16 => if is_int && val >= 0.0 && val <= 65535.0 {
                true
            } else {
                self.emit_bounds_error(val, "u16", "0 to 65.535", "u32", token);
                false
            }
            Type::U32 => if is_int && val >= 0.0 && val <= 4294967295.0 {
                true
            } else {
                self.emit_bounds_error(val, "u32", "0 to 4.294.967.295", "u64", token);
                false
            }
            Type::U64 => if is_int && val >= 0.0 {
                true
            } else {
                self.emit_bounds_error(
                    val,
                    "u64",
                    "0 to 2^64-1",
                    "none (value is too high)",
                    token
                );
                false
            }

            Type::I8 => if is_int && val >= -128.0 && val <= 127.0 {
                true
            } else {
                self.emit_bounds_error(val, "i8", "-128 to 127", "i16", token);
                false
            }
            Type::I16 => if is_int && val >= -32768.0 && val <= 32767.0 {
                true
            } else {
                self.emit_bounds_error(val, "i16", "-32.768 to 32.767", "i32", token);
                false
            }
            Type::I32 => if is_int && val >= -2147483648.0 && val <= 2147483647.0 {
                true
            } else {
                self.emit_bounds_error(val, "i32", "-2.147.483.648 to 2.147.483.647", "i64", token);
                false
            }
            Type::I64 => if is_int {
                true
            } else {
                self.emit_bounds_error(val, "i64", "integer numbers", "f64", token);
                false
            }

            Type::F16 | Type::F32 | Type::F64 => true,
            _ => false,
        }
    }

    fn emit_bounds_error(
        &mut self,
        val: f64,
        type_name: &str,
        limits: &str,
        suggestion: &str,
        token: &Token
    ) {
        self.error_hint(
            token,
            &format!(
                "Value '{}' exceeds limits for type '{}' (range: '{}').",
                val,
                type_name,
                limits
            ),
            format!("consider using `{}` for a higher range", suggestion)
        );
    }

    fn finalizes_execution(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Return { .. } | Stmt::Throw { .. } => true,
            Stmt::Block(stmts) => stmts.iter().any(|s| self.finalizes_execution(s)),
            Stmt::If { then_branch, else_branch, .. } => {
                if let Some(else_b) = else_branch {
                    self.finalizes_execution(then_branch) && self.finalizes_execution(else_b)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn unwrap_indirection(ty: Type) -> Type {
        match ty {
            Type::MutReference(inner) | Type::Reference(inner) | Type::Pointer(inner) => *inner,
            other => other,
        }
    }

    fn find_member_info(
        &self,
        struct_name: &str,
        member_name: &str,
        is_static: bool
    ) -> Option<(Type, bool, Option<usize>)> {
        if let Some(info) = self.user_types.get(struct_name) {
            let map = if is_static { &info.static_methods } else { &info.methods };
            if let Some((params, ret, is_public)) = map.get(member_name) {
                return Some((
                    Type::Function(params.clone(), Box::new(ret.clone()), false),
                    !*is_public,
                    None,
                ));
            }

            if !is_static {
                if let Some((index, t, is_public)) = info.fields.get(member_name) {
                    return Some((t.clone(), !*is_public, Some(*index)));
                }
            }
        }
        None
    }

    fn check_match_exhaustiveness(
        &mut self,
        enum_name: &str,
        cases: &[MatchCase],
        keyword: &Token
    ) {
        let has_wildcard = cases.iter().any(|c| matches!(c.pattern, Expr::WildcardPattern(_)));
        if has_wildcard {
            return;
        }

        let case_names = match self.user_types.get(enum_name) {
            Some(info) if !info.case_names.is_empty() => info.case_names.clone(),
            _ => {
                return;
            }
        };

        let covered: HashSet<String> = cases
            .iter()
            .filter_map(|c| {
                if let Expr::UnionPattern { case_name, .. } = &c.pattern {
                    Some(case_name.lexeme.clone())
                } else if let Expr::Variable(name) = &c.pattern {
                    Some(name.lexeme.clone())
                } else {
                    None
                }
            })
            .collect();

        let missing: Vec<&String> = case_names
            .iter()
            .filter(|n| !covered.contains(*n))
            .collect();

        if !missing.is_empty() {
            self.error(
                keyword,
                &format!(
                    "Non-exhaustive match: unhandled variants: '{}'.",
                    missing
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            );
        }
    }

    fn check_pattern(&mut self, pattern: &Expr, target_type: &Type) {
        match pattern {
            Expr::Variable(name) => self.declare_variable(name, target_type.clone()),
            Expr::UnionPattern { case_name, bindings } => {
                if
                    let Some((Type::Function(param_types, _, _), _)) =
                        self.resolve_variable(case_name)
                {
                    if bindings.len() != param_types.len() {
                        self.error(
                            case_name,
                            "Incorrect number of variables extracted in pattern."
                        );
                    }
                    for (i, binding) in bindings.iter().enumerate() {
                        let t = param_types.get(i).unwrap_or(&Type::Void);
                        self.declare_variable(binding, t.clone());
                    }
                } else {
                    self.error(case_name, "Enum variant not found or is not a function.");
                }
            }
            Expr::WildcardPattern(_) => {}
            _ => {}
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Literal(lit) =>
                match lit {
                    Literal::Number(_) => Type::F64,
                    Literal::Integer(_) => Type::I64,
                    Literal::String(_) => Type::String,
                    Literal::None => Type::Void,
                }

            Expr::Variable(name) => {
                if let Some((t, state)) = self.resolve_variable(name) {
                    if let VarState::Dropped(drop_line) = state {
                        self.error(
                            name,
                            &format!(
                                "Use-After-Free: Variable '{}' has been freed from memory on line '{}' and cannot be used here.",
                                name.lexeme,
                                drop_line
                            )
                        );

                        return t;
                    }
                    t
                } else {
                    let mut real_name = name.lexeme.clone();
                    if let Some(Type::Custom(aliased)) = self.aliases.get(&name.lexeme) {
                        real_name = aliased.clone();
                    }

                    if let Some(info) = self.user_types.get(&real_name) {
                        self.resolved_constructors.insert(name.clone(), real_name.clone());

                        let params = if let Some((p, _, _)) = info.methods.get("init") {
                            p.clone()
                        } else {
                            let mut ordered_fields: Vec<_> = info.fields.values().collect();
                            ordered_fields.sort_by_key(|(idx, _, _)| idx);
                            ordered_fields
                                .into_iter()
                                .map(|(_, t, _)| t.clone())
                                .collect()
                        };
                        Type::Function(params, Box::new(Type::Custom(real_name)), false)
                    } else {
                        let msg = format!("Variable '{}' not declared.", name.lexeme);
                        if let Some(similar) = self.suggest_similar_var(&name.lexeme) {
                            self.error_hint(name, &msg, format!("did you mean `{}`?", similar));
                        } else {
                            self.error(name, &msg);
                        }
                        Type::Void
                    }
                }
            }

            Expr::Assign { name, value } => {
                if let Some((decl_t, state)) = self.resolve_variable(name) {
                    if let VarState::Dropped(drop_line) = state {
                        self.error(
                            name,
                            &format!(
                                "Use-After-Free: Variable '{}' has been freed from memory on line '{}' and cannot be assigned to.",
                                name.lexeme,
                                drop_line
                            )
                        );
                        return Type::Void;
                    }

                    self.check_expr_with_expectation(value, &decl_t, name);
                    decl_t
                } else {
                    self.error(name, "Assigning to non-declared variable.");
                    self.check_expr(value)
                }
            }

            Expr::Binary { left, operator, right } => {
                let lt = self.check_expr(left);
                let rt = self.check_expr_with_expectation(right, &lt, operator);

                match operator.token_type {
                    TokenType::Plus => {
                        if lt == Type::String || rt == Type::String {
                            Type::String
                        } else {
                            self.require_type(&rt, &lt, operator);
                            lt
                        }
                    }
                    TokenType::Minus | TokenType::Star | TokenType::Slash | TokenType::Percent => {
                        self.require_type(&rt, &lt, operator);
                        lt
                    }
                    | TokenType::Greater
                    | TokenType::GreaterEqual
                    | TokenType::Less
                    | TokenType::LessEqual => {
                        self.require_type(&rt, &lt, operator);
                        Type::Bool
                    }
                    TokenType::EqualEqual | TokenType::BangEqual => Type::Bool,
                    TokenType::Exponentiation => {
                        self.require_type(&rt, &lt, operator);
                        lt
                    }
                    TokenType::BitwiseAnd | TokenType::BitwiseOr => {
                        self.require_type(&rt, &lt, operator);
                        lt
                    }
                    _ => {
                        self.error(
                            operator,
                            &format!(
                                "Binary operator '{}' not supported by typechecker.",
                                operator.lexeme
                            )
                        );
                        Type::Void
                    }
                }
            }

            Expr::Logical { left, operator, right } => {
                let left_t = self.check_expr(left);
                self.require_type(&left_t, &Type::Bool, operator);
                let right_t = self.check_expr(right);
                self.require_type(&right_t, &Type::Bool, operator);
                Type::Bool
            }

            Expr::Unary { operator, right } => {
                let rt = self.check_expr(right);
                match operator.token_type {
                    TokenType::Minus | TokenType::MinusMinus | TokenType::PlusPlus => { rt }
                    TokenType::Bang => {
                        self.require_type(&rt, &Type::Bool, operator);
                        Type::Bool
                    }
                    TokenType::Caret => rt,
                    _ => {
                        self.error(
                            operator,
                            &format!(
                                "Unary operator '{}' not supported by typechecker.",
                                operator.lexeme
                            )
                        );
                        Type::Void
                    }
                }
            }

            Expr::Call { callee, arguments, paren } => {
                let ct = self.check_expr(callee);
                if let Type::Function(params, ret, is_var) = ct {
                    let mut is_instance_method = false;
                    if let Expr::Get { object, .. } = &**callee {
                        is_instance_method = true;
                        if let Expr::Variable(var_name) = &**object {
                            if self.user_types.contains_key(&var_name.lexeme) {
                                is_instance_method = false;
                            }
                        }
                    }

                    let expected_args = if is_instance_method && !params.is_empty() {
                        &params[1..]
                    } else {
                        &params[..]
                    };

                    if is_var {
                        if arguments.len() < expected_args.len() {
                            self.error(
                                paren,
                                &format!(
                                    "Expected at least {} arguments, but got {} instead.",
                                    expected_args.len(),
                                    arguments.len()
                                )
                            );
                        }
                    } else if arguments.len() != expected_args.len() {
                        self.error(
                            paren,
                            &format!(
                                "Expected {} arguments, but got {} instead.",
                                expected_args.len(),
                                arguments.len()
                            )
                        );
                    }

                    for (i, arg) in arguments.iter().enumerate() {
                        if i < expected_args.len() {
                            self.check_expr_with_expectation(arg, &expected_args[i], paren);
                        } else {
                            self.check_expr(arg);
                        }
                    }

                    if let Expr::Get { object, name: method_name } = &**callee {
                        if method_name.lexeme == "drop" {
                            if let Expr::Variable(base_var) = &**object {
                                self.mark_as_dropped(&base_var.lexeme, method_name.line);
                            }
                        }
                    }

                    *ret
                } else {
                    self.error(paren, "Target is not callable.");
                    Type::Void
                }
            }

            Expr::Get { object, name } => {
                if let Expr::Variable(var_name) = &**object {
                    let static_method_data = self.user_types
                        .get(&var_name.lexeme)
                        .and_then(|info| info.static_methods.get(&name.lexeme))
                        .map(|(params, ret_t, is_public)| (
                            params.clone(),
                            ret_t.clone(),
                            *is_public,
                        ));

                    if let Some((params, ret_t, is_public)) = static_method_data {
                        if !is_public && self.current_struct.as_ref() != Some(&var_name.lexeme) {
                            self.error(name, "Cannot access private member.");
                        }

                        self.resolved_methods.insert(name.clone(), var_name.lexeme.clone());

                        return Type::Function(params, Box::new(ret_t), false);
                    }
                }

                let obj_t = Self::unwrap_indirection(self.check_expr(object));
                if let Type::Custom(class_name) = obj_t {
                    let member = self
                        .find_member_info(&class_name, &name.lexeme, false)
                        .or_else(|| self.find_member_info(&class_name, &name.lexeme, true));

                    if let Some((m_type, is_private, index_opt)) = member {
                        if is_private && self.current_struct.as_ref() != Some(&class_name) {
                            self.error(name, "Cannot access private member.");
                        }

                        if let Some(idx) = index_opt {
                            self.property_indices.insert(name.clone(), idx);
                        } else {
                            self.resolved_methods.insert(name.clone(), class_name.clone());
                        }
                        return m_type;
                    }
                    self.error(name, "Member not found in struct.");
                } else {
                    self.error(name, "Only struct instances have properties.");
                }
                Type::Void
            }

            Expr::Set { object, name, value } => {
                let obj_t = Self::unwrap_indirection(self.check_expr(object));

                if let Type::Custom(class_name) = obj_t {
                    if
                        let Some((et, is_priv, index_opt)) = self.find_member_info(
                            &class_name,
                            &name.lexeme,
                            false
                        )
                    {
                        if is_priv && self.current_struct.as_ref() != Some(&class_name) {
                            self.error(name, "Cannot access private member.");
                            self.check_expr(value);
                            return Type::Void;
                        } else {
                            let val_t = self.check_expr_with_expectation(value, &et, name);
                            if let Some(idx) = index_opt {
                                self.property_indices.insert(name.clone(), idx);
                            }
                            return val_t;
                        }
                    } else {
                        self.error(name, "Field not found.");
                    }
                } else {
                    self.error(name, "Only struct instances can be modified.");
                }
                self.check_expr(value)
            }

            Expr::Ternary { true_token, condition, then_branch, else_branch } => {
                let cond_t = self.check_expr(condition);
                self.require_type(&cond_t, &Type::Bool, true_token);
                let tt = self.check_expr(then_branch);
                let et = self.check_expr(else_branch);
                if tt == et {
                    tt
                } else {
                    Type::Union(vec![tt, et])
                }
            }

            Expr::TupleLiteral { elements, .. } => {
                Type::Tuple(
                    elements
                        .iter()
                        .map(|e| self.check_expr(e))
                        .collect()
                )
            }

            Expr::Array { elements, bracket } => {
                if elements.is_empty() {
                    self.error(bracket, "Cannot infer type in empty array.");
                    return Type::Array(0, Box::new(Type::Void));
                }

                let inner = self.check_expr(&elements[0]);
                for el in elements.iter().skip(1) {
                    self.check_expr_with_expectation(el, &inner, bracket);
                }
                Type::Array(elements.len(), Box::new(inner))
            }

            Expr::SubscriptGet { indexee, bracket, index } => {
                let target_type = self.check_expr(indexee);
                let index_type = self.check_expr(index);

                if
                    !matches!(
                        index_type,
                        Type::U8 |
                            Type::U16 |
                            Type::U32 |
                            Type::U64 |
                            Type::I8 |
                            Type::I16 |
                            Type::I32 |
                            Type::I64
                    )
                {
                    self.error(bracket, "Array index must be an integer number.");
                }

                match target_type {
                    | Type::Array(_, inner_type)
                    | Type::Slice(inner_type)
                    | Type::Pointer(inner_type) => *inner_type,
                    _ => {
                        self.error(
                            bracket,
                            "Subscript operator can only be used with arrays, slices or pointers."
                        );
                        Type::Void
                    }
                }
            }

            Expr::SubscriptSet { indexee, bracket, index, value } => {
                let target_type = self.check_expr(indexee);
                let index_type = self.check_expr(index);

                if
                    !matches!(
                        index_type,
                        Type::U8 |
                            Type::U16 |
                            Type::U32 |
                            Type::U64 |
                            Type::I8 |
                            Type::I16 |
                            Type::I32 |
                            Type::I64
                    )
                {
                    self.error(bracket, "Array index must be an integer number.");
                }

                match target_type {
                    | Type::Array(_, inner_type)
                    | Type::Slice(inner_type)
                    | Type::Pointer(inner_type) => {
                        self.check_expr_with_expectation(value, &inner_type, bracket);
                        *inner_type
                    }
                    _ => {
                        self.error(
                            bracket,
                            "Subscript operator can only be used with arrays, slices or pointers."
                        );
                        Type::Void
                    }
                }
            }

            Expr::Lambda { params, return_type, body, .. } => {
                self.begin_scope();
                let mut p_types = Vec::new();
                for p in params {
                    if let Some(t) = &p.type_annotation {
                        p_types.push(t.clone());
                        self.declare_variable(&p.name, t.clone());
                    } else {
                        self.error(&p.name, "Lambda parameters require explicit typing.");
                        p_types.push(Type::Void);
                    }
                }

                let ret_t = return_type.clone().unwrap_or(Type::Void);
                let old_ret = self.current_return_type.replace(ret_t.clone());

                for s in body {
                    self.check_stmt(s);
                }

                self.current_return_type = old_ret;
                self.end_scope();
                Type::Function(p_types, Box::new(ret_t), false)
            }

            Expr::Await { keyword, value } => {
                if !self.is_in_async {
                    self.error(keyword, "'await' can only be used in an 'async' context.");
                }
                self.check_expr(value)
            }

            Expr::New { keyword: _, class_name, type_args, arguments, paren } => {
                let base_type = if type_args.is_empty() {
                    Type::Custom(class_name.lexeme.clone())
                } else {
                    Type::Generic(class_name.lexeme.clone(), type_args.clone())
                };

                let resolved_type = self.resolve_type(&base_type);

                if let Type::Custom(real_name) = &resolved_type {
                    if let Some(info) = self.user_types.get(real_name).cloned() {
                        self.resolved_constructors.insert(class_name.clone(), real_name.clone());

                        let expected_args: Vec<Type> = if
                            let Some((params, _, _)) = info.methods.get("init")
                        {
                            params.clone()
                        } else {
                            let mut ordered_fields: Vec<_> = info.fields.values().collect();
                            ordered_fields.sort_by_key(|(idx, _, _)| *idx);
                            ordered_fields
                                .into_iter()
                                .map(|(_, t, _)| t.clone())
                                .collect()
                        };

                        if arguments.len() != expected_args.len() {
                            self.error(
                                paren,
                                &format!(
                                    "Expected {} arguments in constructor, but got {} instead.",
                                    expected_args.len(),
                                    arguments.len()
                                )
                            );
                        }

                        for (i, arg) in arguments.iter().enumerate() {
                            if i < expected_args.len() {
                                self.check_expr_with_expectation(arg, &expected_args[i], paren);
                            } else {
                                self.check_expr(arg);
                            }
                        }

                        return resolved_type;
                    }
                }

                self.error(class_name, &format!("Struct '{}' not found.", class_name.lexeme));
                Type::Void
            }

            Expr::Match { value, cases, keyword } => {
                let val_t = self.check_expr(value);
                if let Type::Custom(name) = &val_t {
                    self.check_match_exhaustiveness(name, cases, keyword);
                }

                let mut return_types = Vec::new();
                for case in cases {
                    self.begin_scope();
                    self.check_pattern(&case.pattern, &val_t);

                    if let Some(guard) = &case.guard {
                        let guard_t = self.check_expr(guard);
                        self.require_type(&guard_t, &Type::Bool, keyword);
                    }

                    for stmt in &case.body {
                        if let Stmt::Expression(e) = stmt {
                            return_types.push(self.check_expr(e));
                        } else if let Stmt::Return { value: Some(e), .. } = stmt {
                            return_types.push(self.check_expr(e));
                        } else {
                            self.check_stmt(stmt);
                        }
                    }

                    self.end_scope();
                }

                if return_types.is_empty() {
                    Type::Void
                } else {
                    let first = return_types[0].clone();
                    for t in &return_types[1..] {
                        if *t != first {
                            self.error(keyword, "All match branches must return the same type.");
                        }
                    }
                    first
                }
            }

            Expr::WildcardPattern(_) | Expr::UnionPattern { .. } => Type::Void,

            Expr::This(keyword) => {
                if self.current_struct.is_none() {
                    self.error(keyword, "Cannot use 'self' outside of a class.");
                    return Type::Void;
                }
                if self.is_in_static {
                    self.error(keyword, "Cannot use 'self' inside of a static method.");
                    return Type::Void;
                }

                if let Some((t, state)) = self.resolve_variable(keyword) {
                    if let VarState::Dropped(drop_line) = state {
                        self.error(
                            keyword,
                            &format!("Use-After-Free: 'self' instance was destroyed on line {}.", drop_line)
                        );
                        return Type::Void;
                    }
                    t
                } else {
                    Type::Void
                }
            }

            Expr::Grouping(expr) => self.check_expr(expr),

            Expr::Spread { operator, right } => {
                let rt = self.check_expr(right);
                match &rt {
                    Type::Array(_, _) | Type::Tuple(_) => rt,
                    _ => {
                        self.error(
                            operator,
                            "Can only use spread operator (...) in arrays or tuples."
                        );
                        Type::Void
                    }
                }
            }

            Expr::Typeof(_) => Type::String,

            Expr::Lazy { expr, statements } => {
                self.begin_scope();
                if let Some(stmts) = statements {
                    for s in stmts {
                        self.check_stmt(s);
                    }
                }
                let t = if let Some(e) = expr { self.check_expr(e) } else { Type::Void };
                self.end_scope();
                t
            }

            Expr::AddressOf { operand, .. } => {
                let inner_type = self.check_expr(operand);
                Type::Pointer(Box::new(inner_type))
            }

            Expr::Dereference { operator, operand } => {
                let operand_type = self.check_expr(operand);
                match operand_type {
                    Type::Pointer(inner) => *inner,
                    _ => {
                        self.error(operator, "Cannot dereference a non-pointer value.");
                        Type::Void
                    }
                }
            }

            Expr::DereferenceSet { operator, ptr, value } => {
                let ptr_type = self.check_expr(ptr);

                match ptr_type {
                    Type::Pointer(inner_type) => {
                        self.check_expr_with_expectation(value, &inner_type, operator);
                        *inner_type
                    }
                    _ => {
                        self.error(
                            operator,
                            "Cannot dereference and assign to a non-pointer value."
                        );
                        Type::Void
                    }
                }
            }

            Expr::Cast { value, target_type, .. } => {
                self.check_expr(value);
                target_type.clone()
            }

            Expr::ListPattern { .. } | Expr::ObjectPattern { .. } => Type::Void,
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Alias { .. } | Stmt::ExternFunction { .. } => {}

            Stmt::Expression(expr) => {
                self.check_expr(expr);
            }

            Stmt::Block(stmts) => {
                self.begin_scope();
                for s in stmts {
                    self.check_stmt(s);
                }
                self.end_scope();
            }

            Stmt::Var { name, type_annotation, initializer } => {
                let mut var_type = Type::Void;
                if let Some(expr) = initializer {
                    var_type = self.check_expr(expr);
                }
                if let Some(annot) = type_annotation {
                    let resolved_annot = self.resolve_type(annot);
                    if initializer.is_some() {
                        self.require_type(&var_type, &resolved_annot, name);
                    }
                    var_type = resolved_annot;
                }

                self.declare_variable(name, var_type);
            }

            Stmt::If { if_token, condition, then_branch, else_branch } => {
                let cond_t = self.check_expr(condition);
                self.require_type(&cond_t, &Type::Bool, if_token);

                self.check_stmt(then_branch);
                if let Some(eb) = else_branch {
                    self.check_stmt(eb);
                }
            }

            Stmt::While { while_token, condition, body } => {
                let cond_t = self.check_expr(condition);
                self.require_type(&cond_t, &Type::Bool, while_token);

                self.loop_depth += 1;
                self.check_stmt(body);
                self.loop_depth -= 1;
            }

            Stmt::For { keyword, initializer, condition, increment, body, .. } => {
                self.begin_scope();
                if let Some(init) = initializer {
                    self.check_stmt(init);
                }
                if let Some(cond) = condition {
                    let cond_t = self.check_expr(cond);
                    self.require_type(&cond_t, &Type::Bool, keyword);
                }
                if let Some(inc) = increment {
                    self.check_expr(inc);
                }

                self.loop_depth += 1;
                for s in body {
                    self.check_stmt(s);
                }
                self.loop_depth -= 1;
                self.end_scope();
            }

            Stmt::ForIn { key, value, iterable, body, .. } => {
                let iter_t = self.check_expr(iterable);
                self.begin_scope();

                let (k_type, v_type) = match iter_t {
                    Type::Array(_, inner) | Type::Slice(inner) => (Type::I64, *inner),
                    _ => {
                        self.error(
                            key,
                            "'for in' iterator can only be used with arrays or slices."
                        );
                        (Type::I64, Type::Void)
                    }
                };

                self.declare_variable(key, k_type);
                if let Some(v) = value {
                    self.declare_variable(v, v_type);
                }

                self.loop_depth += 1;
                for s in body {
                    self.check_stmt(s);
                }
                self.loop_depth -= 1;
                self.end_scope();
            }

            Stmt::Function { name, params, return_type, body, is_async, .. } => {
                let ret_t = return_type
                    .as_ref()
                    .map(|t| self.resolve_type(t))
                    .unwrap_or(Type::Void);

                let p_types: Vec<Type> = params
                    .iter()
                    .map(|p| {
                        if p.name.lexeme == "self" {
                            if let Some(s_name) = &self.current_struct {
                                Type::MutReference(Box::new(Type::Custom(s_name.clone())))
                            } else {
                                self.error(&p.name, "'self' can only be used inside of methods.");
                                Type::Void
                            }
                        } else if let Some(t) = &p.type_annotation {
                            self.resolve_type(t)
                        } else {
                            self.error(&p.name, "Function parameters require explicit typing.");
                            Type::Void
                        }
                    })
                    .collect();

                self.declare_variable(
                    name,
                    Type::Function(p_types.clone(), Box::new(ret_t.clone()), false)
                );

                self.begin_scope();
                let old_async = self.is_in_async;
                self.is_in_async = *is_async;
                let old_ret = self.current_return_type.replace(ret_t.clone());

                for (i, p) in params.iter().enumerate() {
                    self.declare_variable(&p.name, p_types[i].clone());
                }

                if let Some(body_stmts) = body {
                    for s in body_stmts {
                        self.check_stmt(s);
                    }

                    if
                        ret_t != Type::Void &&
                        !body_stmts.iter().any(|s| self.finalizes_execution(s))
                    {
                        self.error(name, "Function must return a value in all branches.");
                    }
                }

                self.current_return_type = old_ret;
                self.is_in_async = old_async;
                self.end_scope();
            }

            // this has already been proccessed in pre_scan
            Stmt::Struct { .. } => {}

            Stmt::Return { keyword, value } => {
                if let Some(expected) = self.current_return_type.clone() {
                    if let Some(val_expr) = value {
                        self.check_expr_with_expectation(val_expr, &expected, keyword);
                    } else if expected != Type::Void {
                        self.error(keyword, "Function requires an explicit return type.");
                    }
                } else {
                    self.error(keyword, "'return' cannot be used outside of a function.");
                }
            }

            Stmt::Throw { thrown, .. } => {
                let _ = self.check_expr(thrown);
            }

            Stmt::TryCatch { try_body, catch_body, exception, exception_type } => {
                for s in try_body {
                    self.check_stmt(s);
                }
                self.begin_scope();
                self.declare_variable(
                    exception,
                    exception_type.clone().unwrap_or(Type::Custom("Exception".into()))
                );
                for s in catch_body {
                    self.check_stmt(s);
                }
                self.end_scope();
            }

            Stmt::ArrayDestructuring { keyword, bindings, initializer } => {
                let init_t = self.check_expr(initializer);

                if let Type::Tuple(ref ts) = init_t {
                    if bindings.len() > ts.len() {
                        self.error(
                            keyword,
                            &format!(
                                "Tuple has {} element(s), but {} were given in destructuring.",
                                ts.len(),
                                bindings.len()
                            )
                        );
                    }
                }

                for (i, b) in bindings.iter().enumerate() {
                    let b_t = match &init_t {
                        Type::Tuple(ts) => ts.get(i).cloned().unwrap_or(Type::Void),
                        Type::Array(_, inner) | Type::Slice(inner) => *inner.clone(),
                        _ => Type::Void,
                    };
                    self.declare_variable(b, b_t);
                }
            }

            Stmt::Using { .. } => {}

            Stmt::Trait { name, methods, .. } => {
                self.declare_variable(name, Type::Custom(name.lexeme.clone()));
                let old_struct = self.current_struct.replace(name.lexeme.clone());
                for m in methods {
                    self.check_stmt(m);
                }
                self.current_struct = old_struct;
            }

            Stmt::Impl { keyword, target_type, methods, .. } => {
                let resolved_target = self.resolve_type(target_type);
                if let Type::Custom(struct_name) = resolved_target {
                    let old_struct = self.current_struct.replace(struct_name.clone());

                    for m in methods {
                        if let Stmt::Function { params, .. } = m {
                            let is_instance = params
                                .first()
                                .map_or(false, |p| {
                                    p.name.lexeme == "self" || p.name.token_type == TokenType::This
                                });

                            self.begin_scope();

                            if is_instance {
                                self.declare_variable(
                                    &Token::synthetic(TokenType::This, "self"),
                                    Type::MutReference(Box::new(Type::Custom(struct_name.clone())))
                                );
                            }

                            let old_static = self.is_in_static;
                            self.is_in_static = !is_instance;

                            self.check_stmt(m);

                            self.is_in_static = old_static;
                            self.end_scope();
                        }
                    }
                    self.current_struct = old_struct;
                } else {
                    self.error(keyword, "Can only implement methods in custom structs.");
                }
            }

            Stmt::Enum { name, cases, .. } => {
                let enum_type = Type::Custom(name.lexeme.clone());

                let case_names: Vec<String> = cases
                    .iter()
                    .map(|c| c.name.lexeme.clone())
                    .collect();

                if let Some(info) = self.user_types.get_mut(&name.lexeme) {
                    info.case_names = case_names;
                }

                for case in cases {
                    let mut p_types = Vec::new();
                    for (_, t_opt) in &case.parameters {
                        let t = t_opt.as_ref().unwrap_or(&Type::Void);
                        p_types.push(self.resolve_type(t));
                    }

                    if let Some(global_scope) = self.scopes.first_mut() {
                        global_scope.insert(case.name.lexeme.clone(), (
                            Type::Function(p_types, Box::new(enum_type.clone()), false),
                            VarState::Valid,
                        ));
                    }
                }
            }
        }
    }
}

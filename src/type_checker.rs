use std::collections::{ HashMap, HashSet };
use crate::ast::{ Expr, Stmt, Type, MatchCase, GenericParam };
use crate::token::{ Token, TokenType, Literal };

#[derive(Debug, Clone, PartialEq)]
pub enum VarState {
    Valid,
    Dropped(usize),
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
    pub fields: HashMap<String, (usize, Type, bool)>,
    pub methods: HashMap<String, (Vec<Type>, Type, bool)>,
    pub static_methods: HashMap<String, (Vec<Type>, Type, bool)>,
    pub case_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CallType {
    Static(String),
    Instance(String),
}

pub struct TypeChecker {
    scopes: Vec<HashMap<String, (Type, VarState)>>,
    pub errors: Vec<TypeError>,
    current_return_type: Option<Type>,

    pub aliases: HashMap<String, Type>,
    pub user_types: HashMap<String, StructInfo>,

    pub templates: HashMap<String, Stmt>,

    pub impl_templates: HashMap<String, Vec<Stmt>>,
    pub impl_targets: HashSet<String>,
    pub instantiations: Vec<Stmt>,
    pub instantiated_types: HashSet<String>,

    pub traits: HashSet<String>,
    pub trait_vtable_layout: HashMap<String, HashMap<String, usize>>,

    pub property_indices: HashMap<Token, usize>,

    pub resolved_calls: HashMap<Token, CallType>,

    current_struct: Option<String>,
    is_in_async: bool,
    is_in_static: bool,
    loop_depth: usize,

    // hindley-milner type inference
    next_type_var: usize,
    expression_types: HashMap<usize, Type>,
    unification_table: HashMap<usize, Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            errors: Vec::new(),
            current_return_type: None,

            aliases: HashMap::new(),
            user_types: HashMap::new(),
            templates: HashMap::new(),

            impl_templates: HashMap::new(),
            instantiations: Vec::new(),
            impl_targets: HashSet::new(),
            instantiated_types: HashSet::new(),

            traits: HashSet::new(),
            trait_vtable_layout: HashMap::new(),

            property_indices: HashMap::new(),
            resolved_calls: HashMap::new(),

            current_struct: None,
            is_in_async: false,
            is_in_static: false,
            loop_depth: 0,

            next_type_var: 0,
            expression_types: HashMap::new(),
            unification_table: HashMap::new(),
        }
    }

    fn type_name(&self, ty: &Type) -> Option<String> {
        Some(match ty {
            Type::Custom(name) => name.clone(),
            Type::Generic(name, _) => name.clone(),
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

                Stmt::Function { name, type_params, params,return_type,.. } => {
                    if !type_params.is_empty() {
                        self.templates.insert(name.lexeme.clone(), stmt.clone());
                    } else {
                        let ps=params.iter().map(|p|self.resolve_type(p.type_annotation.as_ref().unwrap_or(&Type::Void))).collect();
                        let r=self.resolve_type(return_type.as_ref().unwrap_or(&Type::Void));
                        self.scopes[0].insert(name.lexeme.clone(),(Type::Function(ps,Box::new(r),false),VarState::Valid));
                    }
                }

                Stmt::Struct { name, type_params, fields } => {
                    if !type_params.is_empty() {
                        self.templates.insert(name.lexeme.clone(), stmt.clone());
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

                Stmt::Impl { keyword, target_type, methods, .. } => {
                    let resolved_target = self.resolve_type(target_type);
                    if let Some(type_name) = self.type_name(&resolved_target) {
                        if !self.impl_targets.insert(type_name.clone()) {
                            self.error(
                                keyword,
                                &format!("Multiple impl blocks for '{}'. Only one impl block is supported.", type_name)
                            );
                        }

                        let original_name = match target_type {
                            Type::Generic(name, _) => name.clone(),
                            Type::Custom(name) => name.clone(),
                            _ => type_name.clone(),
                        };

                        if self.templates.contains_key(&original_name) {
                            self.impl_templates.insert(original_name.clone(), methods.clone());
                        }
                        if self.templates.contains_key(&type_name) {
                            self.impl_templates.insert(type_name.clone(), methods.clone());
                        }

                        self.user_types.entry(type_name.clone()).or_insert_with(|| StructInfo {
                            type_params: vec![],
                            fields: HashMap::new(),
                            methods: HashMap::new(),
                            static_methods: HashMap::new(),
                            case_names: vec![],
                        });

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
                                            if
                                                self.is_number_type(&resolved_target) ||
                                                resolved_target == Type::Bool ||
                                                resolved_target == Type::String
                                            {
                                                resolved_target.clone()
                                            } else {
                                                Type::MutReference(
                                                    Box::new(resolved_target.clone())
                                                )
                                            }
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

                                if let Some(info) = self.user_types.get_mut(&type_name) {
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
                                            type_name,
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

                Stmt::Enum { name, type_params, cases, .. } => {
                    if !type_params.is_empty() {
                        self.templates.insert(name.lexeme.clone(), stmt.clone());
                    } else {
                        let enum_type = Type::Custom(name.lexeme.clone());
                        let mut info = StructInfo {
                            type_params: vec![],
                            fields: HashMap::new(),
                            methods: HashMap::new(),
                            static_methods: HashMap::new(),
                            case_names: cases
                                .iter()
                                .map(|c| c.name.lexeme.clone())
                                .collect(),
                        };

                        for case in cases {
                            let mut p_types = Vec::new();
                            for (n, t_opt) in &case.parameters {
                                let t = t_opt
                                    .as_ref()
                                    .cloned()
                                    .unwrap_or(Type::Custom(n.lexeme.clone()));
                                p_types.push(self.resolve_type(&t));
                            }

                            info.static_methods.insert(case.name.lexeme.clone(), (
                                p_types.clone(),
                                enum_type.clone(),
                                true,
                            ));
                        }
                        self.user_types.insert(name.lexeme.clone(), info);
                    }
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

    pub fn fresh_type_var(&mut self) -> Type {
        let id = self.next_type_var;
        self.next_type_var += 1;
        Type::TypeVar(id)
    }

    pub fn unify(&mut self, t1: &Type, t2: &Type) -> Result<(), String> {
        let t1_actual = self.resolve_type_vars(t1.clone());
        let t2_actual = self.resolve_type_vars(t2.clone());

        if t1_actual == t2_actual {
            return Ok(());
        }

        match (t1_actual, t2_actual) {
            (Type::TypeVar(id), other) | (other, Type::TypeVar(id)) => {
                if self.occurs_in_type(id, &other) {
                    return Err(format!("Cannot construct recursive type for '?{}'.", id));
                }
                self.unification_table.insert(id, other);
                Ok(())
            }

            (Type::Custom(name1), Type::Custom(name2)) if name1 == name2 => { Ok(()) }

            (Type::Generic(name1, args1), Type::Generic(name2, args2)) => {
                if name1 != name2 || args1.len() != args2.len() {
                    return Err(format!("Cannot unify '{}' with '{}'", name1, name2));
                }

                for (a1, a2) in args1.iter().zip(args2.iter()) {
                    self.unify(a1, a2)?;
                }

                Ok(())
            }

            (Type::Function(p1, r1, _), Type::Function(p2, r2, _)) => {
                if p1.len() != p2.len() {
                    return Err("Function arity mismatch".into());
                }

                for (arg1, arg2) in p1.iter().zip(p2.iter()) {
                    self.unify(arg1, arg2)?;
                }

                self.unify(&*r1, &*r2)
            }

            (Type::Pointer(i1), Type::Pointer(i2)) => self.unify(&i1, &i2),
            (Type::Reference(i1), Type::Reference(i2)) => self.unify(&i1, &i2),
            (Type::MutReference(i1), Type::MutReference(i2)) => self.unify(&i1, &i2),
            (Type::Slice(i1), Type::Slice(i2)) => self.unify(&i1, &i2),
            (Type::Array(s1, i1), Type::Array(s2, i2)) if s1 == s2 => self.unify(&i1, &i2),

            (a, b) => {
                if self.is_assignable(&a, &b) {
                    Ok(())
                } else {
                    Err(format!("Mismatched types: Expected '{}', but found '{}'", b, a))
                }
            }
        }
    }

    pub fn resolve_type_vars(&self, ty: Type) -> Type {
        match ty {
            Type::TypeVar(id) => {
                if let Some(resolved) = self.unification_table.get(&id) {
                    self.resolve_type_vars(resolved.clone())
                } else {
                    Type::TypeVar(id)
                }
            }

            Type::Generic(name, args) => {
                Type::Generic(
                    name,
                    args
                        .into_iter()
                        .map(|t| self.resolve_type_vars(t))
                        .collect()
                )
            }

            Type::Function(params, ret, is_vararg) => {
                Type::Function(
                    params
                        .into_iter()
                        .map(|t| self.resolve_type_vars(t))
                        .collect(),
                    Box::new(self.resolve_type_vars(*ret)),
                    is_vararg
                )
            }

            _ => ty,
        }
    }

    fn occurs_in_type(&self, var_id: usize, ty: &Type) -> bool {
        match ty {
            Type::TypeVar(id) => *id == var_id,
            | Type::Pointer(inner)
            | Type::Reference(inner)
            | Type::MutReference(inner)
            | Type::Slice(inner) => self.occurs_in_type(var_id, inner),
            Type::Array(_, inner) => self.occurs_in_type(var_id, inner),
            Type::Tuple(types) | Type::Union(types) =>
                types.iter().any(|t| self.occurs_in_type(var_id, t)),
            Type::Function(params, ret, _) =>
                params.iter().any(|t| self.occurs_in_type(var_id, t)) ||
                    self.occurs_in_type(var_id, ret),
            Type::Generic(_, args) => args.iter().any(|t| self.occurs_in_type(var_id, t)),
            _ => false,
        }
    }

    pub fn check(&mut self, statements: &mut [Stmt]) {
        self.pre_scan(statements);
        for stmt in statements.iter() { self.check_stmt(stmt); }
        // AST addresses are transient keys only while a body is borrowed by checking.
        // Materialize types before moving/cloning it; no address reaches lowering.
        for stmt in statements.iter_mut() { self.materialize(stmt); }
        self.expression_types.clear();
        let mut checked = 0;
        while checked < self.instantiations.len() {
            let mut inst = self.instantiations[checked].clone();
            crate::ast_visit::fresh_instance(&mut inst);
            self.check_stmt(&inst);
            self.materialize(&mut inst);
            self.instantiations[checked] = inst;
            self.expression_types.clear();
            checked += 1;
        }
    }

    fn final_type(&mut self, ty: Type) -> Type {
        let ty = self.resolve_type_vars(ty);
        match ty {
            Type::TypeVar(_) => Type::I64,
            Type::Custom(ref n) if self.aliases.contains_key(n) => {
                let resolved = self.resolve_type(&ty); self.final_type(resolved)
            }
            Type::Pointer(t) => Type::Pointer(Box::new(self.final_type(*t))),
            Type::Reference(t) => Type::Reference(Box::new(self.final_type(*t))),
            Type::MutReference(t) => Type::MutReference(Box::new(self.final_type(*t))),
            Type::Array(n,t) => Type::Array(n,Box::new(self.final_type(*t))),
            Type::Slice(t) => Type::Slice(Box::new(self.final_type(*t))),
            Type::Function(ps,r,v) => Type::Function(ps.into_iter().map(|t|self.final_type(t)).collect(),Box::new(self.final_type(*r)),v),
            Type::Generic(ref n,ref args) => {
                let key=self.mangle_generic_name(n,args);
                if self.user_types.contains_key(&key) {Type::Custom(key)} else {ty}
            }
            other => other,
        }
    }

    fn materialize(&mut self, stmt: &mut Stmt) {
        if matches!(stmt, Stmt::Function {type_params,..}|Stmt::Impl {type_params,..}|Stmt::Struct {type_params,..}|Stmt::Enum {type_params,..} if !type_params.is_empty()) { return; }
        struct Resolve<'a>(&'a mut TypeChecker);
        impl crate::ast_visit::Visitor for Resolve<'_> {
            fn ty(&mut self, ty: &mut Type) { *ty=self.0.final_type(ty.clone()); }
            fn expr(&mut self, expr: &mut Expr) {
                if let Some(ty)=self.0.expression_types.remove(&(expr as *const Expr as usize)) {
                    let ty=self.0.final_type(ty);
                    let inner=std::mem::replace(expr,Expr::Literal(Literal::None));
                    *expr=Expr::Typed {expr:Box::new(inner),ty};
                }
            }
        }
        crate::ast_visit::stmt(stmt,&mut Resolve(self));
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
        let mut visited = HashSet::new();
        self.resolve_type_inner(ty, &mut visited)
    }

    fn resolve_type_inner(&mut self, ty: &Type, visited: &mut HashSet<String>) -> Type {
        match ty {
            Type::Custom(name) => {
                if !visited.insert(name.clone()) {
                    return ty.clone();
                }
                if let Some(real_type) = self.aliases.get(name).cloned() {
                    return self.resolve_type_inner(&real_type, visited);
                }
                ty.clone()
            }
            Type::Pointer(inner) =>
                Type::Pointer(Box::new(self.resolve_type_inner(inner, visited))),
            Type::Reference(inner) =>
                Type::Reference(Box::new(self.resolve_type_inner(inner, visited))),
            Type::MutReference(inner) =>
                Type::MutReference(Box::new(self.resolve_type_inner(inner, visited))),
            Type::Slice(inner) => Type::Slice(Box::new(self.resolve_type_inner(inner, visited))),
            Type::Array(size, inner) =>
                Type::Array(*size, Box::new(self.resolve_type_inner(inner, visited))),
            Type::Tuple(types) =>
                Type::Tuple(
                    types
                        .iter()
                        .map(|t| self.resolve_type_inner(t, visited))
                        .collect()
                ),
            Type::Function(params, ret, is_var) => {
                Type::Function(
                    params
                        .iter()
                        .map(|t| self.resolve_type_inner(t, visited))
                        .collect(),
                    Box::new(self.resolve_type_inner(ret, visited)),
                    *is_var
                )
            }
            Type::Generic(name, args) => {
                let resolved_args: Vec<Type> = args
                    .iter()
                    .map(|a| self.resolve_type_inner(a, visited))
                    .collect();
                let mangled_name = self.mangle_generic_name(name, &resolved_args);

                if self.user_types.contains_key(&mangled_name) {
                    return Type::Custom(mangled_name);
                }

                if let Some(template_stmt) = self.templates.get(name).cloned() {
                    let type_params = match &template_stmt {
                        Stmt::Struct { type_params, .. } | Stmt::Enum { type_params, .. } =>
                            type_params,
                        _ => {
                            return Type::Void;
                        }
                    };

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

                    let template_type_args: Vec<Type> = type_params
                        .iter()
                        .map(|p| Type::Custom(p.name.lexeme.clone()))
                        .collect();
                    let template_mangled_name = self.mangle_generic_name(name, &template_type_args);

                    substitutions.insert(
                        template_mangled_name.clone(),
                        Type::Custom(mangled_name.clone())
                    );

                    let mut new_info = StructInfo {
                        type_params: vec![],
                        fields: HashMap::new(),
                        methods: HashMap::new(),
                        static_methods: HashMap::new(),
                        case_names: vec![],
                    };

                    match &template_stmt {
                        Stmt::Struct { fields, .. } => {
                            for (index, (f_name, f_type, is_pub)) in fields.iter().enumerate() {
                                let final_type = self.substitute_type(f_type, &substitutions);
                                let final_resolved = self.resolve_type_inner(&final_type, visited);
                                new_info.fields.insert(f_name.lexeme.clone(), (
                                    index,
                                    final_resolved,
                                    *is_pub,
                                ));
                            }
                        }
                        Stmt::Enum { cases, .. } => {
                            let enum_type = Type::Custom(mangled_name.clone());
                            for case in cases {
                                new_info.case_names.push(case.name.lexeme.clone());
                                let mut p_types = Vec::new();
                                for (n, t_opt) in &case.parameters {
                                    let t = t_opt
                                        .as_ref()
                                        .cloned()
                                        .unwrap_or(Type::Custom(n.lexeme.clone()));
                                    let sub_t = self.substitute_type(&t, &substitutions);
                                    p_types.push(self.resolve_type_inner(&sub_t, visited));
                                }
                                new_info.static_methods.insert(case.name.lexeme.clone(), (
                                    p_types,
                                    enum_type.clone(),
                                    true,
                                ));
                            }
                        }
                        _ => {}
                    }

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

                        let mut instantiated_decl = self.monomorphize_stmt(
                            &template_stmt,
                            &substitutions
                        );
                        if
                            let Stmt::Struct { name, .. } | Stmt::Enum { name, .. } =
                                &mut instantiated_decl
                        {
                            name.lexeme = mangled_name.clone();
                        }

                        self.instantiations.push(instantiated_decl);

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
                        .map(|v| self.resolve_type_inner(v, visited))
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
            Type::Array(size, inner) =>
                format!("arr{}_{}", size, self.type_to_mangle_segment(&inner)),
            Type::Slice(inner) => format!("slice_{}", self.type_to_mangle_segment(&inner)),
            Type::Tuple(ts) => {
                let inner_segments: Vec<String> = ts
                    .iter()
                    .map(|t| self.type_to_mangle_segment(t))
                    .collect();
                if ts.is_empty() {
                    "tuple0".into()
                } else {
                    format!("tuple{}_{}", ts.len(), inner_segments.join("_"))
                }
            }
            Type::Function(params, ret, is_var) => {
                let params_segment = params
                    .iter()
                    .map(|t| self.type_to_mangle_segment(t))
                    .collect::<Vec<_>>()
                    .join("_");
                let ret_segment = self.type_to_mangle_segment(&ret);
                let vararg_segment = if is_var { "_vararg" } else { "" };
                let params_segment = if params_segment.is_empty() {
                    "void".into()
                } else {
                    params_segment
                };
                format!("fn_{}_{}{}", params_segment, ret_segment, vararg_segment)
            }
            Type::Generic(name, args) => {
                let segments: Vec<String> = args
                    .iter()
                    .map(|a| self.type_to_mangle_segment(a))
                    .collect();
                if segments.is_empty() {
                    name
                } else {
                    format!("{}_{}", name, segments.join("_"))
                }
            }
            Type::Union(variants) => {
                let segments: Vec<String> = variants
                    .iter()
                    .map(|v| self.type_to_mangle_segment(v))
                    .collect();
                if segments.is_empty() {
                    "union0".into()
                } else {
                    format!("union_{}", segments.join("_"))
                }
            }
            Type::TypeVar(id) => format!("T{}", id),
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
            Type::Union(variants) =>
                Type::Union(
                    variants
                        .iter()
                        .map(|a| self.substitute_type(a, substitutions))
                        .collect()
                ),
            _ => ty.clone(),
        }
    }

    pub fn monomorphize_stmt(&mut self,stmt:&Stmt,subs:&HashMap<String,Type>)->Stmt {
        struct Substitute<'a>(&'a TypeChecker,&'a HashMap<String,Type>);
        impl crate::ast_visit::Visitor for Substitute<'_> {
            fn ty(&mut self,t:&mut Type){*t=self.0.substitute_type(t,self.1);}
        }
        let mut result=stmt.clone();
        crate::ast_visit::stmt(&mut result,&mut Substitute(self,subs));
        if let Stmt::Enum {type_params,..}=&mut result {type_params.clear();}
        result
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
            (Type::Pointer(inner), Type::String) if **inner == Type::U8 => true,

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
        if let Err(msg) = self.unify(actual, expected) {
            if self.is_number_type(actual) && self.is_number_type(expected) {
                self.error_hint(token, &msg, format!("use `as {}` to cast the value", expected));
            } else {
                self.error(token, &msg);
            }
        }
    }

    fn check_expr_with_expectation(&mut self, expr: &Expr, expected: &Type, token: &Token) -> Type {
        let ty=self.check_expected_inner(expr,expected,token);
        self.expression_types.insert(expr as *const Expr as usize,ty.clone());
        ty
    }

    fn check_integer_bounds(&mut self,value:i128,expected:&Type,token:&Token) {
        let range=match expected {
            Type::I8=>Some((i8::MIN as i128,i8::MAX as i128)),Type::U8=>Some((0,u8::MAX as i128)),
            Type::I16=>Some((i16::MIN as i128,i16::MAX as i128)),Type::U16=>Some((0,u16::MAX as i128)),
            Type::I32=>Some((i32::MIN as i128,i32::MAX as i128)),Type::U32=>Some((0,u32::MAX as i128)),
            Type::I64=>Some((i64::MIN as i128,i64::MAX as i128)),Type::U64=>Some((0,u64::MAX as i128)),_=>None,
        };
        if let Some((min,max))=range {
            if value<min || value>max {self.error(token,&format!("Integer literal is out of range for {expected}"));}
        }else{self.check_literal_bounds(value as f64,expected,token);}
    }

    fn check_expected_inner(&mut self, expr: &Expr, expected: &Type, token: &Token) -> Type {
        let mut target_expr = expr;
        while let Expr::Grouping(inner) = target_expr {
            target_expr = inner;
        }

        if let Expr::Unary { operator, right } = target_expr {
            if operator.token_type==TokenType::Minus && self.is_number_type(expected) {
                if let Expr::Literal(Literal::Number(val)) = right.as_ref() {
                    let value = if operator.token_type == TokenType::Minus { -*val } else { *val };
                    self.check_literal_bounds(value, expected, token);
                    return expected.clone();
                }

                if let Expr::Literal(Literal::Integer(val)) = right.as_ref() {
                    let value = if operator.token_type == TokenType::Minus {
                        -*val
                    } else {
                        *val
                    };
                    self.check_integer_bounds(value, expected, token);
                    return expected.clone();
                }
            }
        }

        if let Expr::Literal(Literal::Number(val)) = target_expr {
            if self.is_number_type(expected) {
                self.check_literal_bounds(*val, expected, token);
                return expected.clone();
            }
        }

        if let Expr::Literal(Literal::Integer(val)) = target_expr {
            if self.is_number_type(expected) {
                self.check_integer_bounds(*val, expected, token);
                return expected.clone();
            }
        }

        let actual = self.check_expr(expr);
        let actual_resolved = self.resolve_type_vars(actual);
        let expected_resolved = self.resolve_type_vars(expected.clone());

        if let Err(msg) = self.unify(&actual_resolved, &expected_resolved) {
            if self.is_number_type(&actual_resolved) && self.is_number_type(&expected_resolved) {
                self.error_hint(
                    token,
                    &msg,
                    format!("use `as {}` to cast the value", expected_resolved)
                );
            } else {
                self.error(token, &msg);
            }
        }

        expected_resolved
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

    fn is_integer_type(&self, ty: &Type) -> bool {
        matches!(
            ty,
            Type::U8 |
                Type::U16 |
                Type::U32 |
                Type::U64 |
                Type::I8 |
                Type::I16 |
                Type::I32 |
                Type::I64
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
            Type::U64 => if is_int && val >= 0.0 && val <= 18446744073709551615.0 {
                true
            } else {
                self.emit_bounds_error(
                    val,
                    "u64",
                    "0 to 18.446.744.073.709.551.615",
                    "none (value is too high)",
                    token
                );
                false
            }

            Type::I64 => if is_int && val >= -9223372036854775808.0 && val <= 9223372036854775807.0 {
                true
            } else {
                self.emit_bounds_error(
                    val,
                    "i64",
                    "-9.223.372.036.854.775.808 to 9.223.372.036.854.775.807",
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
        let has_wildcard = cases
            .iter()
            .any(|c| {
                matches!(c.pattern, Expr::WildcardPattern(_)) ||
                    matches!(c.pattern, Expr::Variable(_))
            });
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
                let mut param_types = Vec::new();

                let unref_target = Self::unwrap_indirection(target_type.clone());
                let unref_target = self.resolve_type_vars(unref_target);

                if let Type::Custom(enum_name) = unref_target {
                    if let Some(info) = self.user_types.get(&enum_name) {
                        if let Some((p_ts, _, _)) = info.static_methods.get(&case_name.lexeme) {
                            param_types = p_ts.clone();
                        }
                    }
                }

                if param_types.is_empty() {
                    self.error(case_name, "Enum variant not found on the matched type.");
                } else {
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
                }
            }

            Expr::WildcardPattern(_) => {}
            _ => {}
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> Type {
        let unsupported=match expr {
            Expr::Ternary {true_token,..}=>Some(true_token.clone()),
            Expr::TupleLiteral {token,..}|Expr::Typeof(token)=>Some(token.clone()),
            Expr::Await {keyword,..}=>Some(keyword.clone()),
            Expr::Spread {operator,..}=>Some(operator.clone()),
            Expr::Lazy {..}=>Some(Token::synthetic(TokenType::Lazy,"lazy")),
            _=>None,
        };
        if let Some(token)=unsupported {self.error(&token,"This expression is not implemented by the backend.");return Type::Void;}
        let ty=self.check_expr_inner(expr);
        self.expression_types.insert(expr as *const Expr as usize,ty.clone());
        ty
    }

    fn check_expr_inner(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Typed {ty,..} => ty.clone(),
            Expr::Literal(lit) =>
                match lit {
                    Literal::Number(_) => Type::F64,
                    Literal::Integer(_) => self.fresh_type_var(),
                    Literal::String(_) => Type::String,
                    Literal::Bool(_) => Type::Bool,
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
                        self.resolved_calls.insert(
                            name.clone(),
                            CallType::Static(real_name.clone())
                        );

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
                if matches!(operator.lexeme.as_str(),"**"|"&"|"|") {self.error(operator,"This binary operator is not implemented.");}
                let lt = self.check_expr(left);
                let rt = self.check_expr_with_expectation(right, &lt, operator);

                let lt = self.resolve_type_vars(lt);
                let rt = self.resolve_type_vars(rt);

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
                    TokenType::EqualEqual | TokenType::BangEqual => {
                        self.require_type(&rt, &lt, operator);
                        Type::Bool
                    }
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
                if matches!(operator.token_type,TokenType::PlusPlus|TokenType::MinusMinus) {
                    fn place(e:&Expr)->bool {match e.untyped() {
                        Expr::Variable(_)|Expr::Get {..}|Expr::SubscriptGet {..}|Expr::Dereference {..}=>true,
                        Expr::Grouping(e)=>place(e),_=>false,
                    }}
                    if !place(right) {self.error(operator,"Increment/decrement requires an assignable place.");}
                }
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
                if let Expr::Variable(var_name) = &**callee {
                    if
                        let Some(Stmt::Function { type_params, params, return_type, .. }) =
                            self.templates.get(&var_name.lexeme).cloned()
                    {
                        let mut mapping = HashMap::new();
                        let mut fresh_args = Vec::new();
                        for param in &type_params {
                            let fresh = self.fresh_type_var();
                            mapping.insert(param.name.lexeme.clone(), fresh.clone());
                            fresh_args.push(fresh);
                        }

                        let mut expected_params = Vec::new();
                        for p in &params {
                            let t = p.type_annotation.as_ref().cloned().unwrap_or(Type::Void);
                            expected_params.push(self.substitute_type(&t, &mapping));
                        }

                        if arguments.len() != expected_params.len() {
                            self.error(
                                paren,
                                &format!(
                                    "Expected {} arguments but got {}.",
                                    expected_params.len(),
                                    arguments.len()
                                )
                            );
                        }

                        for (i, arg) in arguments.iter().enumerate() {
                            if i < expected_params.len() {
                                self.check_expr_with_expectation(arg, &expected_params[i], paren);
                            } else {
                                self.check_expr(arg);
                            }
                        }

                        let mut concrete_args = Vec::new();
                        let mut concrete_mapping = HashMap::new();
                        for (i, arg) in fresh_args.into_iter().enumerate() {
                            let res_arg = self.final_type(arg);
                            concrete_args.push(res_arg.clone());
                            concrete_mapping.insert(type_params[i].name.lexeme.clone(), res_arg);
                        }

                        let mangled_name = self.mangle_generic_name(
                            &var_name.lexeme,
                            &concrete_args
                        );

                        self.resolved_calls.insert(
                            var_name.clone(),
                            CallType::Static(mangled_name.clone())
                        );

                        if !self.instantiated_types.contains(&mangled_name) {
                            self.instantiated_types.insert(mangled_name.clone());
                            let template = self.templates.get(&var_name.lexeme).unwrap().clone();
                            let mut instantiated_decl = self.monomorphize_stmt(
                                &template,
                                &concrete_mapping
                            );
                            if
                                let Stmt::Function {
                                    name,
                                    type_params: tp,
                                    ..
                                } = &mut instantiated_decl
                            {
                                name.lexeme = mangled_name.clone();
                                tp.clear();
                            }
                            self.instantiations.push(instantiated_decl);
                        }

                        let ret_t = return_type.as_ref().cloned().unwrap_or(Type::Void);
                        let final_ret = self.substitute_type(&ret_t, &concrete_mapping);
                        return self.resolve_type(&final_ret);
                    }
                }

                let callee_type = self.check_expr(callee);
                let callee_type = self.resolve_type_vars(callee_type);

                match callee_type {
                    Type::Function(param_types, ret_type, is_variadic) => {
                        let mut implicit_self = false;
                        if let Expr::Get { name, .. } = &**callee {
                            if let Some(CallType::Instance(_)) = self.resolved_calls.get(name) {
                                implicit_self = true;
                            }
                        }

                        let expected_param_count = if implicit_self {
                            param_types.len().saturating_sub(1)
                        } else {
                            param_types.len()
                        };

                        if is_variadic {
                            if arguments.len() < expected_param_count {
                                self.error(
                                    paren,
                                    &format!(
                                        "Expected at least {} arguments but got {}.",
                                        expected_param_count,
                                        arguments.len()
                                    )
                                );
                            }
                        } else if arguments.len() != expected_param_count {
                            self.error(
                                paren,
                                &format!(
                                    "Expected {} arguments but got {}.",
                                    expected_param_count,
                                    arguments.len()
                                )
                            );
                        }

                        let param_offset = if implicit_self { 1 } else { 0 };

                        for (i, arg) in arguments.iter().enumerate() {
                            if i + param_offset < param_types.len() {
                                self.check_expr_with_expectation(
                                    arg,
                                    &param_types[i + param_offset],
                                    paren
                                );
                            } else {
                                self.check_expr(arg);
                            }
                        }

                        let mut final_ret = self.resolve_type_vars(*ret_type);

                        if let Type::Generic(base_name, args) = &final_ret {
                            let mut concrete_args = Vec::new();
                            for arg in args {
                                concrete_args.push(self.resolve_type_vars(arg.clone()));
                            }

                            let concrete_generic = Type::Generic(base_name.clone(), concrete_args);

                            let mut overridden = false;
                            if let Some(Type::Custom(expected_mangled)) = &self.current_return_type {
                                if expected_mangled.starts_with(base_name) {
                                    final_ret = Type::Custom(expected_mangled.clone());
                                    overridden = true;
                                }
                            }

                            if !overridden {
                                final_ret = self.resolve_type(&concrete_generic);
                            }

                            if let Expr::Get { name, .. } = &**callee {
                                if let Type::Custom(mangled) = &final_ret {
                                    self.resolved_calls.insert(
                                        name.clone(),
                                        CallType::Static(mangled.clone())
                                    );
                                }
                            }
                        }

                        final_ret
                    }
                    _ => {
                        self.error(paren, "Target is not callable.");
                        Type::Void
                    }
                }
            }

            Expr::Get { object, name } => {
                if let Expr::Variable(var_name) = &**object {
                    let mut real_name = var_name.lexeme.clone();
                    if let Some(Type::Custom(aliased)) = self.aliases.get(&var_name.lexeme) {
                        real_name = aliased.clone();
                    }

                    if let Some(template_stmt) = self.templates.get(&real_name).cloned() {
                        let type_params = match &template_stmt {
                            Stmt::Struct { type_params, .. } | Stmt::Enum { type_params, .. } =>
                                type_params,
                            _ => panic!("ICE: Invalid template"),
                        };

                        let mut mapping = HashMap::new();
                        let mut fresh_args = Vec::new();
                        for param in type_params {
                            let fresh = self.fresh_type_var();
                            mapping.insert(param.name.lexeme.clone(), fresh.clone());
                            fresh_args.push(fresh);
                        }

                        if let Stmt::Enum { cases, .. } = &template_stmt {
                            for case in cases {
                                if &case.name.lexeme == &name.lexeme {
                                    let mut p_types = Vec::new();
                                    for (n, t_opt) in &case.parameters {
                                        let t = t_opt
                                            .as_ref()
                                            .cloned()
                                            .unwrap_or(Type::Custom(n.lexeme.clone()));
                                        p_types.push(self.substitute_type(&t, &mapping));
                                    }

                                    let ret_type = Type::Generic(real_name.clone(), fresh_args);
                                    return Type::Function(p_types, Box::new(ret_type), false);
                                }
                            }
                        }
                    }

                    let static_method_data = self.user_types
                        .get(&real_name)
                        .and_then(|info| info.static_methods.get(&name.lexeme))
                        .map(|(params, ret_t, is_public)| (
                            params.clone(),
                            ret_t.clone(),
                            *is_public,
                        ));

                    if let Some((params, ret_t, is_public)) = static_method_data {
                        if !is_public && self.current_struct.as_ref() != Some(&real_name) {
                            self.error(name, "Cannot access private member.");
                        }

                        self.resolved_calls.insert(
                            name.clone(),
                            CallType::Static(real_name.clone())
                        );
                        return Type::Function(params, Box::new(ret_t), false);
                    }
                }

                let obj_t = Self::unwrap_indirection(self.check_expr(object));
                let obj_t = self.resolve_type_vars(obj_t);

                if let Some(class_name) = self.type_name(&obj_t) {
                    let mut is_instance = false;
                    let member = self
                        .find_member_info(&class_name, &name.lexeme, false)
                        .map(|(t, p, idx)| {
                            is_instance = true;
                            (t, p, idx)
                        })
                        .or_else(|| self.find_member_info(&class_name, &name.lexeme, true));

                    if let Some((m_type, is_private, index_opt)) = member {
                        if is_private && self.current_struct.as_ref() != Some(&class_name) {
                            self.error(name, "Cannot access private member.");
                        }

                        if let Some(idx) = index_opt {
                            self.property_indices.insert(name.clone(), idx);
                        } else {
                            self.resolved_calls.insert(name.clone(), if is_instance {
                                CallType::Instance(class_name.clone())
                            } else {
                                CallType::Static(class_name.clone())
                            });
                        }
                        return m_type;
                    }
                    self.error(name, "Member not found on this type.");
                } else {
                    self.error(name, "Type does not have properties or methods.");
                }
                Type::Void
            }

            Expr::Set { object, name, value } => {
                let obj_t = Self::unwrap_indirection(self.check_expr(object));
                let obj_t = self.resolve_type_vars(obj_t);

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

            Expr::StructInit { class_name, type_args, properties, brace } => {
                let mut actual_type_args = type_args.clone();
                if actual_type_args.is_empty() {
                    let num_params = if
                        let Some(template_stmt) = self.templates.get(&class_name.lexeme)
                    {
                        match template_stmt {
                            Stmt::Struct { type_params, .. } | Stmt::Enum { type_params, .. } =>
                                type_params.len(),
                            _ => 0,
                        }
                    } else {
                        0
                    };

                    for _ in 0..num_params {
                        actual_type_args.push(self.fresh_type_var());
                    }
                }

                let base_type = if actual_type_args.is_empty() {
                    Type::Custom(class_name.lexeme.clone())
                } else {
                    Type::Generic(class_name.lexeme.clone(), actual_type_args)
                };

                let resolved_type = self.resolve_type(&base_type);

                if let Type::Custom(real_name) = &resolved_type {
                    if let Some(info) = self.user_types.get(real_name).cloned() {
                        self.resolved_calls.insert(
                            class_name.clone(),
                            CallType::Static(real_name.clone())
                        );

                        let mut seen_fields = std::collections::HashSet::new();
                        for (name, expr) in properties.iter() {
                            if let Some((_, expected_ty, _)) = info.fields.get(&name.lexeme) {
                                seen_fields.insert(name.lexeme.clone());
                                self.check_expr_with_expectation(expr, expected_ty, name);
                            } else {
                                self.error(
                                    name,
                                    &format!(
                                        "Field '{}' not found on struct '{}'.",
                                        name.lexeme,
                                        real_name
                                    )
                                );
                                self.check_expr(expr);
                            }
                        }

                        if seen_fields.len() != info.fields.len() {
                            self.error(
                                brace,
                                &format!(
                                    "Expected {} fields for '{}', but got {}.",
                                    info.fields.len(),
                                    real_name,
                                    seen_fields.len()
                                )
                            );
                        }

                        let final_base_type = self.resolve_type_vars(base_type);
                        let mut final_resolved = self.resolve_type(&final_base_type);

                        if let Type::Generic(base_name, args) = &final_resolved {
                            let mut concrete_args = Vec::new();
                            for arg in args {
                                concrete_args.push(self.resolve_type_vars(arg.clone()));
                            }

                            let concrete_generic = Type::Generic(base_name.clone(), concrete_args);
                            final_resolved = self.resolve_type(&concrete_generic);

                            if let Type::Custom(mangled) = &final_resolved {
                                self.resolved_calls.insert(
                                    class_name.clone(),
                                    CallType::Static(mangled.clone())
                                );
                            }
                        }

                        return final_resolved;
                    }
                }

                self.error(class_name, &format!("Struct '{}' not found.", class_name.lexeme));
                Type::Void
            }

            Expr::Match { value, cases, keyword } => {
                let val_t = self.check_expr(value);
                let val_t = self.resolve_type_vars(val_t);

                let matched_type=Self::unwrap_indirection(val_t.clone());
                let exhaustive_bool=matched_type==Type::Bool && [false,true].iter().all(|wanted|cases.iter().any(|c| {
                    c.guard.is_none() && matches!(c.pattern.untyped(),Expr::Literal(Literal::Bool(actual)) if actual==wanted)
                }));
                if let Type::Custom(name) = &matched_type {
                    self.check_match_exhaustiveness(name, cases, keyword);
                } else if !exhaustive_bool && !matches!(matched_type,Type::Generic(..)) && !cases.iter().any(|c|c.guard.is_none() && matches!(c.pattern,Expr::WildcardPattern(_)|Expr::Variable(_))) {
                    self.error(keyword,"Non-exhaustive scalar match requires an unguarded fallback arm.");
                }

                let mut return_types = Vec::new();
                for case in cases {
                    self.begin_scope();
                    self.check_pattern(&case.pattern, &val_t);

                    if let Some(guard) = &case.guard {
                        let guard_t = self.check_expr(guard);
                        self.require_type(&guard_t, &Type::Bool, keyword);
                    }

                    let mut case_ret_type = Type::Void;

                    let actual_stmts = if case.body.len() == 1 {
                        if let Stmt::Block(inner) = &case.body[0] { inner } else { &case.body }
                    } else {
                        &case.body
                    };

                    for stmt in actual_stmts {
                        if let Stmt::Expression(e) = stmt {
                            case_ret_type = self.check_expr(e);
                        } else if let Stmt::Return { value: Some(e), .. } = stmt {
                            case_ret_type = self.check_expr(e);
                            self.check_stmt(stmt);
                        } else {
                            self.check_stmt(stmt);
                        }
                    }

                    return_types.push(case_ret_type);
                    self.end_scope();
                }

                if return_types.is_empty() {
                    Type::Void
                } else {
                    let first = self.resolve_type_vars(return_types[0].clone());
                    for t in &return_types[1..] {
                        if let Err(_) = self.unify(&first, t) {
                            self.error(keyword, "All match branches must return the same type.");
                        }
                    }
                    first
                }
            }

            | Expr::WildcardPattern(_)
            | Expr::UnionPattern { .. }
            | Expr::ListPattern { .. }
            | Expr::ObjectPattern { .. } => Type::Void,

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
                let rt = self.resolve_type_vars(rt);
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

            Expr::AddressOf { operand, operator } => {
                fn place(e:&Expr)->bool {match e.untyped() {
                    Expr::Variable(_)|Expr::Get {..}|Expr::SubscriptGet {..}|Expr::Dereference {..}=>true,
                    Expr::Grouping(e)=>place(e),_=>false,
                }}
                if !place(operand) {self.error(operator,"Address-of requires an assignable place.");}
                let inner_type = self.check_expr(operand);
                Type::Pointer(Box::new(inner_type))
            }

            Expr::Dereference { operator, operand } => {
                let operand_type = self.check_expr(operand);
                let operand_type = self.resolve_type_vars(operand_type);

                match operand_type {
                    Type::Pointer(inner) | Type::Reference(inner) | Type::MutReference(inner) =>
                        *inner,
                    _ => {
                        self.error(operator, "Cannot dereference a non-pointer/reference value.");
                        Type::Void
                    }
                }
            }

            Expr::DereferenceSet { operator, ptr, value } => {
                let ptr_type = self.check_expr(ptr);
                let ptr_type = self.resolve_type_vars(ptr_type);

                match ptr_type {
                    | Type::Pointer(inner_type)
                    | Type::Reference(inner_type)
                    | Type::MutReference(inner_type) => {
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

            Expr::Cast { value, target_type, operator } => {
                let source=self.check_expr(value);
                let source=self.final_type(source);
                let target=self.resolve_type(target_type);
                fn scalar(t:&Type)->bool {matches!(t,Type::I8|Type::I16|Type::I32|Type::I64|Type::U8|Type::U16|Type::U32|Type::U64|Type::F16|Type::F32|Type::F64|Type::Bool|Type::Pointer(_)|Type::String)}
                if !(scalar(&source)&&scalar(&target)) && source!=target {
                    self.error(operator,"Unsupported cast between these types.");
                }
                if target==Type::Void || source==Type::Void {self.error(operator,"Void is not a value and cannot be cast.");}
                target
            }

            Expr::Ternary { true_token, condition, then_branch, else_branch } => {
                let cond_t = self.check_expr(condition);
                self.require_type(&cond_t, &Type::Bool, true_token);
                let tt = self.check_expr(then_branch);
                let et = self.check_expr(else_branch);

                let tt = self.resolve_type_vars(tt);
                let et = self.resolve_type_vars(et);

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
                let inner = self.resolve_type_vars(inner);

                for el in elements.iter().skip(1) {
                    self.check_expr_with_expectation(el, &inner, bracket);
                }
                Type::Array(elements.len(), Box::new(inner))
            }

            Expr::SubscriptGet { indexee, bracket, index } => {
                let target_type = self.check_expr(indexee);
                let index_type = self.check_expr_with_expectation(index, &Type::U64, bracket);

                let index_type = self.resolve_type_vars(index_type);
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

                let target_type = self.resolve_type_vars(target_type);
                match target_type {
                    | Type::Array(_, inner_type)
                    | Type::Slice(inner_type)
                    | Type::Pointer(inner_type) => *inner_type,
                    Type::String => Type::U8,
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
                let index_type = self.check_expr_with_expectation(index, &Type::U64, bracket);

                let index_type = self.resolve_type_vars(index_type);
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

                let target_type = self.resolve_type_vars(target_type);
                match target_type {
                    | Type::Array(_, inner_type)
                    | Type::Slice(inner_type)
                    | Type::Pointer(inner_type) => {
                        self.check_expr_with_expectation(value, &inner_type, bracket);
                        *inner_type
                    }
                    Type::String => {
                        self.check_expr_with_expectation(value, &Type::U8, bracket);
                        Type::U8
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

                if ret_t!=Type::Void && !body.iter().any(|s|self.finalizes_execution(s)) {
                    self.error(&Token::synthetic(TokenType::Fn,"lambda"),"Lambda must return a value on every path.");
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
                let var_type = match (type_annotation, initializer) {
                    (Some(annot), Some(expr)) => {
                        let expected=self.resolve_type(annot);
                        self.check_expr_with_expectation(expr,&expected,name)
                    }
                    (Some(annot), None) => self.resolve_type(annot),
                    (None, Some(expr)) => self.check_expr(expr),
                    (None, None) => Type::Void,
                };
                if var_type == Type::Void {self.error(name,"A variable must have a non-void value type.");}

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
                let iter_t = self.resolve_type_vars(iter_t);
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

                self.declare_variable(key, if value.is_some() {k_type}else{v_type.clone()});
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

            Stmt::Function { name, params, return_type, body, is_async, type_params, .. } => {
                if self.current_return_type.is_some() {self.error(name,"Nested named functions are not supported; use a lambda.");return;}
                if !type_params.is_empty() {
                    return;
                }

                let ret_t = return_type
                    .as_ref()
                    .map(|t| self.resolve_type(t))
                    .unwrap_or(Type::Void);

                let p_types: Vec<Type> = params
                    .iter()
                    .map(|p| {
                        if p.name.lexeme == "self" {
                            if let Some((existing_ty, _)) = self.resolve_variable(&p.name) {
                                existing_ty.clone()
                            } else if let Some(s_name) = &self.current_struct {
                                let base_ty = match s_name.as_str() {
                                    "i8" => Type::I8,
                                    "i16" => Type::I16,
                                    "i32" => Type::I32,
                                    "i64" => Type::I64,
                                    "u8" => Type::U8,
                                    "u16" => Type::U16,
                                    "u32" => Type::U32,
                                    "u64" => Type::U64,
                                    "f16" => Type::F16,
                                    "f32" => Type::F32,
                                    "f64" => Type::F64,
                                    "bool" => Type::Bool,
                                    "string" => Type::String,
                                    _ => Type::Custom(s_name.clone()),
                                };

                                Type::MutReference(Box::new(base_ty))
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

                let signature=Type::Function(p_types.clone(),Box::new(ret_t.clone()),false);
                if !self.scopes.last().is_some_and(|s|s.contains_key(&name.lexeme)) {self.declare_variable(name,signature);}

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

            Stmt::Throw { keyword, thrown } => {
                self.error(keyword,"Exception handling is not implemented.");
                let _ = self.check_expr(thrown);
            }

            Stmt::TryCatch { try_body, catch_body, exception, exception_type } => {
                self.error(exception,"Exception handling is not implemented.");
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
                let init_t = self.resolve_type_vars(init_t);

                if let Type::Array(n,_) = &init_t {
                    if bindings.len()>*n {self.error(keyword,"Array destructuring exceeds the array length.");}
                }
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
                if let Some(struct_name) = self.type_name(&resolved_target) {
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
                                let self_ty = if
                                    self.is_number_type(&resolved_target) ||
                                    resolved_target == Type::Bool ||
                                    resolved_target == Type::String
                                {
                                    resolved_target.clone()
                                } else {
                                    Type::MutReference(Box::new(resolved_target.clone()))
                                };

                                self.declare_variable(
                                    &Token::synthetic(TokenType::This, "self"),
                                    self_ty
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
                    self.error(keyword, "Can only implement methods on valid types.");
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

                    for (n, t_opt) in &case.parameters {
                        let t = t_opt.as_ref().cloned().unwrap_or(Type::Custom(n.lexeme.clone()));
                        p_types.push(self.resolve_type(&t));
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

use std::collections::{ HashMap, HashSet };
use crate::ast::{ Expr, Stmt, Type, MatchCase, GenericParam, FunctionParam };
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

/// Typed call-site defaults. Parameter bindings are evaluated values, never
/// substituted source expressions; references to an earlier argument cannot
/// evaluate that argument a second time.
#[derive(Debug,Clone)]
pub struct DefaultCall {
    pub parameters: Vec<(Token,Type)>,
    pub defaults: Vec<Expr>,
    pub instance: bool,
}

pub struct TypeChecker {
    scopes: Vec<HashMap<String, (Type, VarState)>>,
    pub errors: Vec<TypeError>,
    pub module_symbols: HashSet<String>,
    pub external_names: HashMap<String,String>,
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
    integer_literals: Vec<(Type, i128)>,
    declared_structs: HashSet<String>,
    trait_implementations: HashSet<(String,String)>,
    specialized_types: HashMap<String,(String,Vec<Type>)>,
    enum_constructors: HashMap<String,Option<String>>,
    function_names: HashSet<String>,
    method_templates: HashMap<(String,String),Stmt>,
    pub specialized_methods: HashMap<Token,(String,bool)>,
    unification_table: HashMap<usize, Type>,
    default_declarations: HashMap<String,Stmt>,
    pub default_calls: HashMap<Token,DefaultCall>,
    default_stack: HashSet<String>,
    default_evaluators_needed: HashSet<String>,
    pub default_evaluators: HashMap<String,Stmt>,
    expected_types: HashMap<usize,Type>,
    forbidden_default_parameters: Option<(HashSet<String>,usize)>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            errors: Vec::new(),
            module_symbols: HashSet::new(),
            external_names: HashMap::new(),
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
            integer_literals: Vec::new(),
            declared_structs: HashSet::new(),
            trait_implementations: HashSet::new(),
            specialized_types: HashMap::new(),
            enum_constructors: HashMap::new(),
            function_names: HashSet::new(),
            method_templates: HashMap::new(),
            specialized_methods: HashMap::new(),
            unification_table: HashMap::new(),
            default_declarations: HashMap::new(),
            default_calls: HashMap::new(),
            default_stack: HashSet::new(),
            default_evaluators_needed: HashSet::new(),
            default_evaluators: HashMap::new(),
            expected_types: HashMap::new(),
            forbidden_default_parameters: None,
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

    fn validate_declarations(&mut self, statements: &[Stmt]) {
        let mut known = HashSet::new();
        let mut aliases = HashMap::new();
        let mut declarations: HashMap<String, (String, bool)> = HashMap::new();
        for stmt in statements {
            let (name, kind, defined) = match stmt {
                Stmt::Struct { name, .. } | Stmt::Enum { name, .. } | Stmt::Trait { name, .. } => {
                    known.insert(name.lexeme.clone());
                    (name, "type".to_string(), true)
                }
                Stmt::Alias { name, target } => {
                    known.insert(name.lexeme.clone());
                    aliases.insert(name.lexeme.clone(), target.clone());
                    (name, "alias".to_string(), true)
                }
                Stmt::Function {
                    name,
                    params,
                    return_type,
                    body,
                    ..
                } => (
                    name,
                    format!(
                        "fn {:?} {:?} false",
                        params
                            .iter()
                            .map(|p| &p.type_annotation)
                            .collect::<Vec<_>>(),
                        return_type
                    ),
                    body.is_some(),
                ),
                Stmt::ExternFunction {
                    name,
                    params,
                    return_type,
                    is_variadic,
                } => (
                    name,
                    format!(
                        "fn {:?} {:?} {is_variadic}",
                        params
                            .iter()
                            .map(|p| &p.type_annotation)
                            .collect::<Vec<_>>(),
                        return_type
                    ),
                    false,
                ),
                _ => continue,
            };
            if (name.lexeme.starts_with("__fishy_") || name.lexeme == "__global_init") && !self.module_symbols.contains(&name.lexeme) {
                self.error(name, "Name is reserved for the compiler runtime.");
            }
            if let Some((previous, has_body)) = declarations.get_mut(&name.lexeme) {
                if *previous != kind || (*has_body && defined) {
                    self.error(name, "Conflicting or duplicate declaration.");
                }
                *has_body |= defined;
            } else {
                declarations.insert(name.lexeme.clone(), (kind, defined));
            }
        }
        fn cycle(t: &Type, aliases: &HashMap<String, Type>, stack: &mut HashSet<String>) -> bool {
            match t {
                Type::Custom(n) if aliases.contains_key(n) => {
                    if !stack.insert(n.clone()) {
                        return true;
                    }
                    let result = cycle(&aliases[n], aliases, stack);
                    stack.remove(n);
                    result
                }
                Type::Pointer(t)
                | Type::Reference(t)
                | Type::MutReference(t)
                | Type::Slice(t)
                | Type::Array(_, t) => cycle(t, aliases, stack),
                Type::Tuple(ts) | Type::Union(ts) | Type::Generic(_, ts) => {
                    ts.iter().any(|t| cycle(t, aliases, stack))
                }
                Type::Function(ps, r, _) => {
                    ps.iter().any(|t| cycle(t, aliases, stack)) || cycle(r, aliases, stack)
                }
                _ => false,
            }
        }
        for (n, t) in &aliases {
            if cycle(t, &aliases, &mut HashSet::from([n.clone()])) {
                self.error(&Token::synthetic(TokenType::Alias, n), "Cyclic type alias.");
            }
        }
        struct Validate<'a> {
            checker: &'a mut TypeChecker,
            known: &'a HashSet<String>,
            params: HashSet<String>,
        }
        impl crate::ast_visit::Visitor for Validate<'_> {
            fn ty(&mut self, t: &mut Type) {
                fn walk(v: &mut Validate<'_>, t: &Type) {
                    match t {
                        Type::Custom(n) if !v.known.contains(n) && !v.params.contains(n) => {
                            v.checker.error(
                                &Token::synthetic(TokenType::Identifier, n),
                                "Unknown type name.",
                            )
                        }
                        Type::Union(_) => v.checker.error(
                            &Token::synthetic(TokenType::BitwiseOr, "|"),
                            "Anonymous union representation requires a language design decision.",
                        ),
                        Type::Pointer(t) | Type::Reference(t) | Type::MutReference(t) => walk(v, t),
                        Type::Array(_, t) | Type::Slice(t) => {
                            if **t == Type::Void {
                                v.checker.error(
                                    &Token::synthetic(TokenType::Identifier, "void"),
                                    "Aggregate element cannot be void.",
                                );
                            }
                            walk(v, t);
                        }
                        Type::Tuple(ts) => {
                            for t in ts {
                                if *t == Type::Void {
                                    v.checker.error(
                                        &Token::synthetic(TokenType::Identifier, "void"),
                                        "Tuple element cannot be void.",
                                    );
                                }
                                walk(v, t);
                            }
                        }
                        Type::Generic(n, ts) => {
                            if !v.known.contains(n) {
                                v.checker.error(
                                    &Token::synthetic(TokenType::Identifier, n),
                                    "Unknown generic type.",
                                );
                            }
                            for t in ts {
                                walk(v, t);
                            }
                        }
                        Type::Function(ps, r, _) => {
                            for t in ps {
                                if *t == Type::Void {
                                    v.checker.error(
                                        &Token::synthetic(TokenType::Identifier, "void"),
                                        "Parameter cannot be void.",
                                    );
                                }
                                walk(v, t);
                            }
                            walk(v, r);
                        }
                        _ => {}
                    }
                }
                walk(self, t);
            }
        }
        for stmt in statements {
            let mut members=HashSet::new();
            match stmt {
                Stmt::Struct {fields,..}=>for (name,_,_) in fields {if !members.insert(name.lexeme.clone()) {self.error(name,"Duplicate struct field.");}},
                Stmt::Enum {cases,..}=>for case in cases {if !members.insert(case.name.lexeme.clone()) {self.error(&case.name,"Duplicate enum case.");}},
                _=>{}
            }
            let mut params = HashSet::new();
            if let Stmt::Function { type_params, .. }
            | Stmt::Struct { type_params, .. }
            | Stmt::Enum { type_params, .. }
            | Stmt::Trait { type_params, .. }
            | Stmt::Impl { type_params, .. } = stmt
            {
                params.extend(type_params.iter().map(|p| p.name.lexeme.clone()));
            }
            if let Stmt::Trait {
                name,
                type_params,
                traits,
                ..
            } = stmt
            {
                if !type_params.is_empty() || !traits.is_empty() {
                    self.error(
                        name,
                        "Generic traits and trait inheritance require a language design decision.",
                    );
                }
            }
            if let Stmt::Impl { methods, .. } | Stmt::Trait { methods, .. } = stmt {
                for method in methods {
                    let mut method_params = params.clone();
                    if let Stmt::Function { type_params, .. } = method {
                        method_params.extend(type_params.iter().map(|p| p.name.lexeme.clone()));
                    }
                    let mut copy = method.clone();
                    crate::ast_visit::stmt(
                        &mut copy,
                        &mut Validate {
                            checker: self,
                            known: &known,
                            params: method_params,
                        },
                    );
                }
            }
            let mut copy = stmt.clone();
            if let Stmt::Impl { methods, .. } | Stmt::Trait { methods, .. } = &mut copy {
                methods.clear();
            }
            crate::ast_visit::stmt(
                &mut copy,
                &mut Validate {
                    checker: self,
                    known: &known,
                    params,
                },
            );
            match stmt {
                Stmt::Struct { fields, .. } => {
                    for (n, t, _) in fields {
                        if *t == Type::Void {
                            self.error(n, "Struct field cannot be void.");
                        }
                    }
                }
                Stmt::Function { params, .. } | Stmt::ExternFunction { params, .. } => {
                    for p in params {
                        if p.type_annotation == Some(Type::Void) {
                            self.error(&p.name, "Parameter cannot be void.");
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn valid_storage_type(ty: &Type) -> bool {
        match ty {
            Type::Void | Type::Union(_) => false,
            Type::Array(_, t) | Type::Slice(t) => Self::valid_storage_type(t),
            Type::Tuple(ts) => ts.iter().all(Self::valid_storage_type),
            Type::Function(ps, _, var) => !var && ps.iter().all(Self::valid_storage_type),
            _ => true,
        }
    }

    fn check_constraints(&mut self, params: &[GenericParam], args: &[Type], token: &Token) {
        for (p, arg) in params.iter().zip(args) {
            let arg = self.resolve_type_vars(arg.clone());
            for constraint in &p.constraints {
                match constraint {
                    Type::Custom(tr) if self.traits.contains(tr) => {
                        let valid = self.type_name(&arg).is_some_and(|n| {
                            n == *tr || self.trait_implementations.contains(&(n, tr.clone()))
                        });
                        if !valid {
                            self.error(
                                token,
                                &format!("Type '{arg}' does not satisfy trait constraint '{tr}'."),
                            );
                        }
                    }
                    _ => self.error(token, "Only declared traits can be generic constraints."),
                }
            }
        }
    }

    fn validate_conformance(&mut self, statements: &[Stmt]) {
        for stmt in statements {
            if let Stmt::Impl {
                keyword,
                trait_name: Some(tr),
                target_type,
                type_params,
                methods,
            } = stmt
            {
                if !type_params.is_empty() {
                    self.error(
                        keyword,
                        "Generic trait implementations require a language design decision.",
                    );
                    continue;
                }
                let resolved = self.resolve_type(target_type);
                let Some(target) = self.type_name(&resolved) else {
                    self.error(keyword, "Invalid trait implementation target.");
                    continue;
                };
                self.trait_implementations
                    .insert((target.clone(), tr.to_string()));
                let Type::Custom(tr) = tr else {
                    self.error(keyword, "Expected a trait name.");
                    continue;
                };
                if !self.traits.contains(tr) {
                    self.error(keyword, "Implemented type is not a trait.");
                    continue;
                }
                let required = self.user_types[tr].methods.clone();
                for (name, (ps, ret, _)) in required {
                    let actual = self
                        .user_types
                        .get(&target)
                        .and_then(|i| i.methods.get(&name))
                        .cloned();
                    if !methods.iter().any(
                        |s| matches!(s,Stmt::Function {name:n,body:Some(_),..} if n.lexeme==name),
                    ) {
                        self.error(keyword, &format!("Missing trait method '{name}'."));
                        continue;
                    }
                    if let Some((actual_ps, actual_ret, _)) = actual {
                        if ps.len() != actual_ps.len()
                            || ps.iter().skip(1).ne(actual_ps.iter().skip(1))
                            || ret != actual_ret
                        {
                            self.error(
                                keyword,
                                &format!("Signature does not match trait method '{name}'."),
                            );
                        }
                    } else {
                        self.error(
                            keyword,
                            &format!("Trait method '{name}' must have a self receiver."),
                        );
                    }
                }
            }
        }
    }

    pub fn pre_scan(&mut self, statements: &[Stmt]) {
        for stmt in statements {
            if let Stmt::Function {name,..}=stmt {
                // A prototype without defaults must not discard definition defaults.
                if matches!(stmt,Stmt::Function {params,..} if params.iter().any(|p|p.default_value.is_some())) || !self.default_declarations.contains_key(&name.lexeme) {
                    self.default_declarations.insert(name.lexeme.clone(),stmt.clone());
                }
            }
        }
        for stmt in statements {
            if let Stmt::Function {name,..}|Stmt::ExternFunction {name,..}=stmt {self.function_names.insert(name.lexeme.clone());}
        }
        // Register type names/templates before resolving any declaration signatures.
        for stmt in statements {
            match stmt {
                Stmt::Alias {name,target} => {self.aliases.insert(name.lexeme.clone(),target.clone());}
                Stmt::Struct {name,type_params,..}|Stmt::Enum {name,type_params,..}|
                Stmt::Function {name,type_params,..} if !type_params.is_empty() => {
                    self.templates.insert(name.lexeme.clone(),stmt.clone());
                }
                Stmt::Struct {name,..}|Stmt::Enum {name,..}|Stmt::Trait {name,..} => {
                    self.user_types.entry(name.lexeme.clone()).or_insert_with(|| StructInfo {
                        type_params:vec![],fields:HashMap::new(),methods:HashMap::new(),
                        static_methods:HashMap::new(),case_names:vec![],
                    });
                    if matches!(stmt,Stmt::Trait {..}) {self.traits.insert(name.lexeme.clone());}
                }
                Stmt::Impl {trait_name:Some(Type::Custom(tr)),target_type:Type::Custom(ty),..} => {
                    self.trait_implementations.insert((ty.clone(),tr.clone()));
                }
                Stmt::Impl {target_type:Type::Generic(..),..}=>{}
                _=>{}
            }
        }
        for stmt in statements {
            if let Stmt::Struct {name,..}=stmt {self.declared_structs.insert(name.lexeme.clone());}
        }
        for stmt in statements {
            if let Stmt::Impl {keyword,target_type:Type::Generic(owner,args),type_params,methods,..}=stmt {
                if type_params.is_empty() {continue;}
                let params=match self.templates.get(owner) {Some(Stmt::Struct {type_params,..}|Stmt::Enum {type_params,..})=>type_params.clone(),_=>continue};
                let mut substitutions=HashMap::new();
                if args.len()!=params.len() {self.error(keyword,"Generic implementation arity mismatch.");continue;}
                let mut valid=true;
                for (arg,param) in args.iter().zip(&params) {
                    if let Type::Custom(binding)=arg {
                        if !type_params.iter().any(|p|&p.name.lexeme==binding)||substitutions.insert(binding.clone(),Type::Custom(param.name.lexeme.clone())).is_some() {valid=false;}
                    }else{valid=false;}
                }
                if !valid||substitutions.len()!=type_params.len()||type_params.iter().any(|p|!p.constraints.is_empty()) {
                    self.error(keyword,"Conditional or patterned generic implementations require a language design decision.");continue;
                }
                for method in methods {
                    let mut bindings=substitutions.clone();
                    if let Stmt::Function {type_params,..}=method {for p in type_params {bindings.remove(&p.name.lexeme);}}
                    let method=self.monomorphize_stmt(method,&bindings);
                    let name=if let Stmt::Function {name,..}=&method {name.lexeme.clone()}else{continue;};
                    if matches!(self.templates.get(owner),Some(Stmt::Enum {cases,..}) if cases.iter().any(|c|c.name.lexeme==name)) {
                        self.error(keyword,"Duplicate implementation method or enum constructor.");continue;
                    }
                    let previous=self.impl_templates.get(owner).is_some_and(|ms|ms.iter().any(|m|matches!(m,Stmt::Function {name:n,..} if n.lexeme==name)));
                    if previous {self.error(keyword,"Duplicate implementation method.");}
                    else{self.impl_templates.entry(owner.clone()).or_default().push(method);}
                }
            }
        }

        for stmt in statements {
            if let Stmt::Impl {trait_name:Some(Type::Custom(tr)),target_type,type_params,..}=stmt {
                if type_params.is_empty() {
                    let target=self.resolve_type(target_type);
                    if let Some(name)=self.type_name(&target) {self.trait_implementations.insert((name,tr.clone()));}
                }
            }
        }
        // Implementations need the complete field and trait tables.
        let ordered=statements.iter().filter(|s|!matches!(s,Stmt::Impl {..}))
            .chain(statements.iter().filter(|s|matches!(s,Stmt::Impl {..})));
        for stmt in ordered {
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

                Stmt::Impl { keyword: _, target_type, methods, type_params, .. } => {
                    if !type_params.is_empty() {continue;}
                    let resolved_target = self.resolve_type(target_type);
                    if let Some(type_name) = self.type_name(&resolved_target) {
                        self.impl_targets.insert(type_name.clone());

                        self.user_types.entry(type_name.clone()).or_insert_with(|| StructInfo {
                            type_params: vec![],
                            fields: HashMap::new(),
                            methods: HashMap::new(),
                            static_methods: HashMap::new(),
                            case_names: vec![],
                        });

                        for method in methods {
                            if let Stmt::Function {name,..}=method {
                                self.default_declarations.insert(crate::symbols::member(&type_name,&name.lexeme),method.clone());
                            }
                            if let Stmt::Function {name,..}=method {
                                let duplicate=self.method_templates.contains_key(&(type_name.clone(),name.lexeme.clone()))||self.user_types.get(&type_name).is_some_and(|i|i.methods.contains_key(&name.lexeme)||i.static_methods.contains_key(&name.lexeme));
                                if duplicate {self.error(name,"Duplicate implementation method or enum constructor.");continue;}
                            }
                            if let Stmt::Function {name,type_params,..}=method {
                                if !type_params.is_empty() {self.method_templates.insert((type_name.clone(),name.lexeme.clone()),method.clone());continue;}
                            }
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
                                        let mangled_name = crate::symbols::member(&type_name,&m_name.lexeme);
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
                        for case in cases {
                            let qualified=crate::symbols::member(&name.lexeme,&case.name.lexeme);
                            self.enum_constructors.entry(case.name.lexeme.clone()).and_modify(|entry|*entry=None).or_insert(Some(qualified));
                            let ps=info.static_methods[&case.name.lexeme].0.clone();
                            self.scopes[0].entry(case.name.lexeme.clone()).or_insert((Type::Function(ps,Box::new(enum_type.clone()),false),VarState::Valid));
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

        if let (Type::Custom(n),Type::Generic(..))=(&t1_actual,&t2_actual) {
            if let Some((base,args))=self.specialized_types.get(n).cloned() {return self.unify(&Type::Generic(base,args),&t2_actual);}
        }
        if let (Type::Generic(..),Type::Custom(n))=(&t1_actual,&t2_actual) {
            if let Some((base,args))=self.specialized_types.get(n).cloned() {return self.unify(&t1_actual,&Type::Generic(base,args));}
        }
        match (t1_actual, t2_actual) {
            (Type::TypeVar(id), other) | (other, Type::TypeVar(id)) => {
                if self.occurs_in_type(id, &other) {
                    return Err(format!("Cannot construct recursive type for '?{}'.", id));
                }
                if self.integer_literals.iter().any(|(ty,_)|matches!(ty,Type::TypeVar(n) if *n==id)) && !self.is_number_type(&other) && !matches!(other,Type::TypeVar(_)) {
                    return Err(format!("Numeric literal cannot have type '{other}'."));
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

            (Type::Function(p1, r1, v1), Type::Function(p2, r2, v2)) => {
                if p1.len() != p2.len() || v1 != v2 {
                    return Err("Function arity mismatch".into());
                }

                for (arg1, arg2) in p1.iter().zip(p2.iter()) {
                    self.unify(arg1, arg2)?;
                }

                self.unify(&*r1, &*r2)
            }

            (Type::Pointer(i1), Type::Pointer(i2)) => self.unify(&i1, &i2),
            (Type::Pointer(i1),Type::Reference(i2)|Type::MutReference(i2)) if !matches!(i2.as_ref(),Type::Custom(_))=>self.unify(&i1,&i2),
            (Type::Reference(i1), Type::Reference(i2)) => self.unify(&i1, &i2),
            (Type::MutReference(i1), Type::MutReference(i2)) => self.unify(&i1, &i2),
            (Type::Range(i1), Type::Range(i2)) => self.unify(&i1, &i2),
            (Type::Slice(i1), Type::Slice(i2)) => self.unify(&i1, &i2),
            (Type::Array(s1, i1), Type::Array(s2, i2)) if s1 == s2 => self.unify(&i1, &i2),
            (Type::Tuple(a),Type::Tuple(b)) if a.len()==b.len() => {
                for (a,b) in a.iter().zip(&b) {self.unify(a,b)?;} Ok(())
            }

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

            Type::Pointer(t)=>Type::Pointer(Box::new(self.resolve_type_vars(*t))),
            Type::Reference(t)=>Type::Reference(Box::new(self.resolve_type_vars(*t))),
            Type::MutReference(t)=>Type::MutReference(Box::new(self.resolve_type_vars(*t))),
            Type::Range(t)=>Type::Range(Box::new(self.resolve_type_vars(*t))),
            Type::Slice(t)=>Type::Slice(Box::new(self.resolve_type_vars(*t))),
            Type::Array(n,t)=>Type::Array(n,Box::new(self.resolve_type_vars(*t))),
            Type::Tuple(ts)=>Type::Tuple(ts.into_iter().map(|t|self.resolve_type_vars(t)).collect()),
            Type::Union(ts)=>Type::Union(ts.into_iter().map(|t|self.resolve_type_vars(t)).collect()),
            _ => ty,
        }
    }

    fn occurs_in_type(&self, var_id: usize, ty: &Type) -> bool {
        match ty {
            Type::TypeVar(id) => *id == var_id,
            | Type::Range(inner)
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
        let defaults:HashMap<_,_>=statements.iter().filter_map(|s| {
            if let Stmt::Trait {name,methods,..}=s {Some((name.lexeme.clone(),methods.iter().filter(|s|matches!(s,Stmt::Function {body:Some(_),..})).cloned().collect::<Vec<_>>()))}else{None}
        }).collect();
        for stmt in statements.iter_mut() {
            if let Stmt::Impl {trait_name:Some(Type::Custom(tr)),methods,..}=stmt {
                if let Some(defaults)=defaults.get(tr) {
                    for default in defaults {
                        let Stmt::Function {name,..}=default else{continue;};
                        if !methods.iter().any(|m|matches!(m,Stmt::Function {name:n,..} if n.lexeme==name.lexeme)) {
                            let mut method=default.clone();crate::ast_visit::fresh_instance(&mut method);
                            if let Stmt::Function {is_public,..}=&mut method {*is_public=true;}
                            methods.push(method);
                        }
                    }
                }
            }
        }
        self.validate_declarations(statements);
        if !self.errors.is_empty() {return;}
        self.pre_scan(statements);
        let mut linked_signatures=HashMap::new();
        for stmt in statements.iter() {
            let name=match stmt {Stmt::ExternFunction {name,..}|Stmt::Function {name,..}=>name,_=>continue};
            if let Some((signature,_))=self.scopes[0].get(&name.lexeme).cloned() {
                let native=self.external_names.get(&name.lexeme).unwrap_or(&name.lexeme);
                if let Some(previous)=linked_signatures.insert(native.clone(),signature.clone()) {
                    if previous!=signature {self.error(name,&format!("Conflicting external signatures for '{native}'."));}
                }
            }
        }
        self.validate_conformance(statements);
        let invalid=self.user_types.iter().flat_map(|(n,i)|i.fields.values().filter(|(_,t,_)|!Self::valid_storage_type(t)).map(|_|n.clone())).collect::<Vec<_>>();
        for name in invalid {self.error(&Token::synthetic(TokenType::Struct,&name),"Struct field must have a supported non-void value type.");}
        // Initializers retain source order; function bodies see every established global.
        for stmt in statements.iter().filter(|s|!matches!(s,Stmt::Function {..}|Stmt::Impl {..}|Stmt::Trait {..})) {self.check_stmt(stmt);}
        for stmt in statements.iter().filter(|s|matches!(s,Stmt::Function {..}|Stmt::Impl {..}|Stmt::Trait {..})) {self.check_stmt(stmt);}
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
        for (ty,value) in std::mem::take(&mut self.integer_literals) {
            let ty=self.final_type(ty);
            let token=Token::synthetic(TokenType::Number,"integer literal");
            if !self.is_number_type(&ty) {self.error(&token,"Integer literal requires a numeric type.");}
            else{self.check_integer_bounds(value,&ty,&token);}
        }
    }

    fn missing_generic_information(&self, ty: &Type) -> bool {
        match self.resolve_type_vars(ty.clone()) {
            Type::TypeVar(id) => {
                // Only numeric literal variables have the established i64
                // fallback. An unused generic binder has no such evidence.
                !self.integer_literals.iter().any(|(literal, _)| {
                    self.resolve_type_vars(literal.clone()) == Type::TypeVar(id)
                })
            }
            Type::Pointer(t) | Type::Reference(t) | Type::MutReference(t)
            | Type::Array(_, t) | Type::Slice(t) | Type::Range(t) =>
                self.missing_generic_information(&t),
            Type::Generic(_, ts) | Type::Tuple(ts) | Type::Union(ts) =>
                ts.iter().any(|t| self.missing_generic_information(t)),
            Type::Function(ps, ret, _) => ps.iter().any(|t| self.missing_generic_information(t))
                || self.missing_generic_information(&ret),
            _ => false,
        }
    }

    fn infer_generic_arguments(&mut self, params: &[GenericParam], args: &[Type], token: &Token) -> Vec<Type> {
        for (param, arg) in params.iter().zip(args) {
            if self.missing_generic_information(arg) {
                self.error(token, &format!(
                    "Cannot infer generic parameter '{}'; supply an explicit argument or a contextual type.",
                    param.name.lexeme
                ));
            }
        }
        args.iter().map(|t| self.final_type(t.clone())).collect()
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
            Type::Range(t) => Type::Range(Box::new(self.final_type(*t))),
            Type::Slice(t) => Type::Slice(Box::new(self.final_type(*t))),
            Type::Function(ps,r,v) => Type::Function(ps.into_iter().map(|t|self.final_type(t)).collect(),Box::new(self.final_type(*r)),v),
            Type::Generic(ref n,ref args) => {
                let key=self.mangle_generic_name(n,args);
                if self.user_types.contains_key(&key) {Type::Custom(key)} else {ty}
            }
            Type::Tuple(ts)=>Type::Tuple(ts.into_iter().map(|t|self.final_type(t)).collect()),
            Type::Union(ts)=>Type::Union(ts.into_iter().map(|t|self.final_type(t)).collect()),
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
                if matches!(self.templates.get(name),Some(Stmt::Struct {..}|Stmt::Enum {..})) {
                    self.error(&Token::synthetic(TokenType::Identifier,name),"Generic type requires type arguments.");return Type::Void;
                }
                if !visited.insert(name.clone()) {
                    return ty.clone();
                }
                if let Some(real_type) = self.aliases.get(name).cloned() {
                    let resolved=self.resolve_type_inner(&real_type, visited);
                    visited.remove(name);
                    return resolved;
                }
                visited.remove(name);
                ty.clone()
            }
            Type::Pointer(inner) =>
                Type::Pointer(Box::new(self.resolve_type_inner(inner, visited))),
            Type::Reference(inner) =>
                Type::Reference(Box::new(self.resolve_type_inner(inner, visited))),
            Type::MutReference(inner) =>
                Type::MutReference(Box::new(self.resolve_type_inner(inner, visited))),
            Type::Range(inner) => Type::Range(Box::new(self.resolve_type_inner(inner, visited))),
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

                    self.check_constraints(type_params,&resolved_args,&Token::synthetic(TokenType::Identifier,name));
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

                    self.specialized_types.insert(mangled_name.clone(),(name.clone(),resolved_args.clone()));
                    self.user_types.insert(mangled_name.clone(),new_info.clone());
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

                    if let Some(methods)=self.impl_templates.get(name).cloned() {
                        for method in methods {
                            if matches!(&method,Stmt::Function {type_params,..} if !type_params.is_empty()) {continue;}
                            if let Stmt::Function {name:m,params,return_type,is_public,..}=method {
                                let instance=params.first().is_some_and(|p|p.name.lexeme=="self");
                                let mut ps=Vec::new();
                                for p in params {
                                    let ty=if p.name.lexeme=="self" {
                                        Type::MutReference(Box::new(Type::Custom(mangled_name.clone())))
                                    }else{self.substitute_type(p.type_annotation.as_ref().unwrap_or(&Type::Void),&substitutions)};
                                    ps.push(self.resolve_type_inner(&ty,visited));
                                }
                                let ret=self.substitute_type(return_type.as_ref().unwrap_or(&Type::Void),&substitutions);
                                let ret=self.resolve_type_inner(&ret,visited);
                                if instance {new_info.methods.insert(m.lexeme,(ps,ret,is_public));}
                                else {new_info.static_methods.insert(m.lexeme,(ps,ret,is_public));}
                            }
                        }
                    }

                    self.user_types.insert(mangled_name.clone(), new_info);
                    if matches!(template_stmt,Stmt::Struct {..}) {self.declared_structs.insert(mangled_name.clone());}

                    if !self.instantiated_types.contains(&mangled_name) {
                        self.instantiated_types.insert(mangled_name.clone());

                        let mut instantiated_decl = self.monomorphize_stmt(
                            &template_stmt,
                            &substitutions
                        );
                        if
                            let Stmt::Struct { name, type_params, .. } | Stmt::Enum { name, type_params, .. } =
                                &mut instantiated_decl
                        {
                            name.lexeme = mangled_name.clone();
                            type_params.clear();
                        }

                        self.instantiations.push(instantiated_decl);

                        if let Some(methods_ast) = self.impl_templates.get(name).cloned() {
                            let mut instantiated_methods_ast = Vec::new();

                            for method in methods_ast {
                                let mut outer_substitutions=substitutions.clone();
                                if let Stmt::Function {type_params,..}=&method {for p in type_params {outer_substitutions.remove(&p.name.lexeme);}}
                                let method=self.monomorphize_stmt(&method,&outer_substitutions);
                                if let Stmt::Function {name,type_params,..}=&method {
                                    self.default_declarations.insert(crate::symbols::member(&mangled_name,&name.lexeme),method.clone());
                                    if !type_params.is_empty() {self.method_templates.insert((mangled_name.clone(),name.lexeme.clone()),method.clone());}
                                }
                                instantiated_methods_ast.push(method);
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

    fn mangle_generic_name(&mut self,base_name:&str,type_args:&[Type])->String {
        crate::symbols::generic(base_name,type_args)
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
            Type::Range(inner) => Type::Range(Box::new(self.substitute_type(inner, substitutions))),
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
            | (Type::Custom(a_name), Type::Reference(e_inner)) => {
                self.is_assignable(&Type::Custom(a_name.clone()), e_inner)
            }

            (Type::Reference(inner)|Type::MutReference(inner),Type::Custom(n)) if matches!(inner.as_ref(),Type::Custom(_))=>self.is_assignable(inner,&Type::Custom(n.clone())),
            (Type::Pointer(a_inner),Type::Reference(e_inner)|Type::MutReference(e_inner)) if !matches!(e_inner.as_ref(),Type::Custom(_))=>self.is_assignable(a_inner,e_inner),
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
        // Grouping is transparent to every contextual rule, including calls and
        // recursive aggregates. Record the inner type as well as the wrapper.
        if let Expr::Grouping(inner) = expr.untyped() {
            return self.check_expr_with_expectation(inner, expected, token);
        }
        if let Expr::Match {value,cases,keyword}=expr.untyped() {return self.check_match(value,cases,keyword,Some(expected));}
        if let Expr::Ternary {condition,then_branch,else_branch,true_token}=expr.untyped() {
            let cond=self.check_expr(condition);self.require_type(&cond,&Type::Bool,true_token);
            self.check_expr_with_expectation(then_branch,expected,true_token);
            self.check_expr_with_expectation(else_branch,expected,true_token);
            return expected.clone();
        }
        if self.diverges(expr) {self.check_expr(expr);return expected.clone();}
        if let (Expr::AddressOf {operand,..},Type::Reference(t)|Type::MutReference(t))=(expr.untyped(),expected) {
            if matches!(t.as_ref(),Type::Custom(_)) {
                let actual=self.check_expr(operand);
                if matches!(expected, Type::MutReference(_)) {self.require_writable_place(operand, token);}
                if !self.is_place(operand) {self.error(token,"Address-of requires an assignable storage place.");}
                self.require_type(&actual,t,token);return expected.clone();
            }
        }
        if matches!(expr.untyped(),Expr::Literal(Literal::None))&&matches!(expected,Type::Pointer(_)|Type::String) {return expected.clone();}
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

        if let Expr::Call {callee,arguments,..}=target_expr {
            if let (Expr::Get {object,name},Type::Custom(expected_name))=(callee.untyped(),expected) {
                if let Expr::Variable(base)=object.untyped() {
                    if self.specialized_types.get(expected_name).is_some_and(|(name,_)|name==&base.lexeme) || expected_name==&base.lexeme {
                        if let Some((ps,ret,_))=self.user_types.get(expected_name).and_then(|i|i.static_methods.get(&name.lexeme)).cloned() {
                            if arguments.len()!=ps.len() {self.error(token,"Incorrect number of constructor arguments.");}
                            for (arg,ty) in arguments.iter().zip(&ps) {self.check_expr_with_expectation(arg,ty,token);}
                            self.resolved_calls.insert(name.clone(),CallType::Static(expected_name.clone()));
                            self.expression_types.insert(callee.as_ref() as *const Expr as usize,Type::Function(ps,Box::new(ret.clone()),false));
                            return ret;
                        }
                    }
                }
            }
        }
        match (target_expr,expected) {
            (Expr::Array {elements,bracket},Type::Array(n,inner)) => {
                let (length,_)=self.check_array_elements(elements,Some(inner),bracket);
                if length!=*n {self.error(bracket,"Array length does not match its annotation.");}
                return expected.clone();
            }
            (Expr::Array {elements,bracket},Type::Slice(inner)) => {
                let (length,_)=self.check_array_elements(elements,Some(inner),bracket);
                return Type::Array(length,inner.clone());
            }
            (Expr::TupleLiteral {elements,token},Type::Tuple(types)) => {
                if elements.len()!=types.len() {self.error(token,"Tuple arity does not match its annotation.");}
                for (e,t) in elements.iter().zip(types) {self.check_expr_with_expectation(e,t,token);}
                return expected.clone();
            }
            _=>{}
        }
        if matches!(expr.untyped(),Expr::Call {..}|Expr::StructInit {..}) {self.expected_types.insert(expr as *const Expr as usize,expected.clone());}
        let actual = self.check_expr(expr);
        if matches!(expected, Type::MutReference(_)) {
            if let Expr::AddressOf {operand,..} = expr.untyped() {
                self.require_writable_place(operand, token);
            }
        }
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

    fn check_array_elements(
        &mut self,
        elements: &[Expr],
        expected: Option<&Type>,
        token: &Token,
    ) -> (usize, Type) {
        let mut element = expected.cloned();
        let mut count = 0usize;
        for expr in elements {
            if let Expr::Spread { right, operator } = expr.untyped() {
                let ty = self.check_expr(right);
                let ty = self.resolve_type_vars(ty);
                let (length, types) = match &ty {
                    Type::Array(n, t) => (*n, vec![t.as_ref().clone()]),
                    Type::Tuple(ts) => (ts.len(), ts.clone()),
                    _ => {
                        self.error(operator, "Array spread requires a fixed array or tuple.");
                        (0, vec![])
                    }
                };
                self.expression_types
                    .insert(expr as *const Expr as usize, ty);
                count = count.checked_add(length).unwrap_or_else(|| {
                    self.error(operator, "Array length overflow.");
                    0
                });
                for ty in types {
                    if let Some(expected) = &element {
                        self.require_type(&ty, expected, operator);
                    } else {
                        element = Some(ty);
                    }
                }
            } else {
                let ty = if let Some(expected) = &element {
                    self.check_expr_with_expectation(expr, expected, token)
                } else {
                    self.check_expr(expr)
                };
                if element.is_none() {
                    element = Some(ty);
                }
                count = count.checked_add(1).unwrap_or_else(|| {
                    self.error(token, "Array length overflow.");
                    0
                });
            }
        }
        let ty = element.unwrap_or_else(|| {
            self.error(token, "Cannot infer type in empty array.");
            Type::Void
        });
        (count, ty)
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
            Type::U64 => if is_int && val >= 0.0 && val < 18446744073709551616.0 {
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

            Type::I64 => if is_int && val >= -9223372036854775808.0 && val < 9223372036854775808.0 {
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

    fn diverges(&self, expr: &Expr) -> bool {
        match expr.untyped() {
            Expr::Call { callee, .. } => {
                matches!(callee.untyped(),Expr::Variable(n) if matches!(n.lexeme.as_str(),"panic"|"exit")&&!self.scopes.iter().skip(1).any(|s|s.contains_key(&n.lexeme)))
            }
            Expr::Grouping(e) => self.diverges(e),
            Expr::Match { cases, .. } => {
                !cases.is_empty()
                    && cases
                        .iter()
                        .all(|c| c.body.iter().any(|s| self.finalizes_execution(s)))
            }
            _ => false,
        }
    }

    fn finalizes_execution(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Return { .. } | Stmt::Throw { .. } => true,
            Stmt::Expression(e)=>self.diverges(e),
            Stmt::For {condition:None,..}=>true,
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

    fn enum_info(&self, ty: &Type) -> Option<&StructInfo> {
        if let Type::Custom(n) = ty {
            self.user_types.get(n).filter(|i| !i.case_names.is_empty())
        } else {
            None
        }
    }

    fn check_match_exhaustiveness(
        &mut self,
        enum_name: &str,
        cases: &[MatchCase],
        keyword: &Token,
    ) {
        let Some(info) = self.user_types.get(enum_name).cloned() else {
            return;
        };
        if info.case_names.is_empty() {
            return;
        }
        let mut covered = HashSet::new();
        for case in cases.iter().filter(|c| c.guard.is_none()) {
            match case.pattern.untyped() {
                Expr::WildcardPattern(_) => return,
                Expr::Variable(n) if !info.case_names.contains(&n.lexeme) => return,
                Expr::Variable(n) => {
                    covered.insert(n.lexeme.clone());
                }
                Expr::UnionPattern { case_name, .. } => {
                    covered.insert(case_name.lexeme.clone());
                }
                _ => {}
            }
        }
        let missing = info
            .case_names
            .iter()
            .filter(|n| !covered.contains(*n))
            .cloned()
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            self.error(
                keyword,
                &format!(
                    "Non-exhaustive match: unhandled variants: {}.",
                    missing.join(", ")
                ),
            );
        }
    }

    fn check_pattern(&mut self, pattern: &Expr, target: &Type) {
        let target = self.resolve_type_vars(Self::unwrap_indirection(target.clone()));
        match pattern.untyped() {
            Expr::Variable(n) => {
                if let Some(info) = self.enum_info(&target) {
                    if info.case_names.contains(&n.lexeme) {
                        if !info.static_methods[&n.lexeme].0.is_empty() {
                            self.error(n, "Payload variant requires a binding pattern.");
                        }
                        return;
                    }
                }
                self.declare_variable(n, target.clone());
            }
            Expr::UnionPattern {
                case_name,
                bindings,
            } => {
                let ps = self
                    .enum_info(&target)
                    .and_then(|i| i.static_methods.get(&case_name.lexeme))
                    .map(|m| m.0.clone());
                if let Some(ps) = ps {
                    if ps.len() != bindings.len() {
                        self.error(
                            case_name,
                            "Incorrect number of variables extracted in pattern.",
                        );
                    }
                    for (n, t) in bindings.iter().zip(ps) {
                        self.declare_variable(n, t);
                    }
                } else {
                    self.error(case_name, "Enum variant not found on the matched type.");
                }
            }
            Expr::WildcardPattern(_) => {}
            Expr::Literal(_) => {
                self.check_expr_with_expectation(
                    pattern,
                    &target,
                    &Token::synthetic(TokenType::Match, "pattern"),
                );
            }
            Expr::ListPattern { elements, rest } => {
                match &target {
                    Type::Array(_, inner) | Type::Slice(inner) => {
                        for p in elements {
                            self.check_pattern(p, inner);
                        }
                        if let Some(rest) = rest {
                            let ty=match &target {
                                Type::Array(n,inner)=>Type::Array(n.saturating_sub(elements.len()),inner.clone()),
                                Type::Slice(inner)=>Type::Slice(inner.clone()),
                                _=>unreachable!(),
                            };
                            if let Expr::Variable(name)=rest.untyped() {
                                if self.scopes.last().is_some_and(|s|s.contains_key(&name.lexeme)) {self.error(name,"Duplicate remainder pattern binding.");}
                                self.declare_variable(name,ty.clone());
                                self.expression_types.insert(rest.as_ref() as *const Expr as usize,ty);
                            } else {
                                self.error(&Token::synthetic(TokenType::Spread,"..."),"List remainder must bind a variable.");
                            }
                        }
                    }
                    _ => self.error(
                        &Token::synthetic(TokenType::Match, "pattern"),
                        "List pattern requires an array or slice.",
                    ),
                }
            }
            Expr::ObjectPattern { properties, rest } => {
                if rest.is_some() {
                    self.error(
                        &Token::synthetic(TokenType::Spread, "..."),
                        "Rest object type requires a language design decision: nominal structs cannot represent an anonymous remainder record.",
                    );
                }
                if let Type::Custom(ref n) = target {
                    if let Some(info) = self.user_types.get(n).cloned() {
                        let mut seen = HashSet::new();
                        for (name, p) in properties {
                            if !seen.insert(&name.lexeme) {
                                self.error(name, "Duplicate pattern field.");
                            }
                            if let Some((index, ty, public)) = info.fields.get(&name.lexeme) {
                                if !public && self.current_struct.as_ref() != Some(n) {
                                    self.error(name, "Cannot access private member.");
                                }
                                self.property_indices.insert(name.clone(), *index);
                                self.check_pattern(p, ty);
                            } else {
                                self.error(name, "Pattern field not found.");
                            }
                        }
                    }
                } else {
                    self.error(
                        &Token::synthetic(TokenType::Match, "pattern"),
                        "Object pattern requires a struct.",
                    );
                }
            }
            _ => self.error(
                &Token::synthetic(TokenType::Match, "pattern"),
                "Unsupported pattern; expected a literal, binding, variant, list, or object.",
            ),
        }
    }

    fn check_match(
        &mut self,
        value: &Expr,
        cases: &[MatchCase],
        keyword: &Token,
        expected: Option<&Type>,
    ) -> Type {
        let val_t = self.check_expr(value);
        let val_t = self.resolve_type_vars(val_t);

        let matched_type = Self::unwrap_indirection(val_t.clone());
        let exhaustive_bool=matched_type==Type::Bool && [false,true].iter().all(|wanted|cases.iter().any(|c| {
                    c.guard.is_none() && matches!(c.pattern.untyped(),Expr::Literal(Literal::Bool(actual)) if actual==wanted)
                }));
        if let Type::Custom(name) = &matched_type {
            self.check_match_exhaustiveness(name, cases, keyword);
        } else if !exhaustive_bool
            && !matches!(matched_type, Type::Generic(..))
            && !cases.iter().any(|c| {
                c.guard.is_none()
                    && matches!(c.pattern, Expr::WildcardPattern(_) | Expr::Variable(_))
            })
        {
            self.error(
                keyword,
                "Non-exhaustive scalar match requires an unguarded fallback arm.",
            );
        }

        let mut return_types = Vec::new();
        for case in cases {
            self.begin_scope();
            self.check_pattern(&case.pattern, &val_t);

            if let Some(guard) = &case.guard {
                let guard_t = self.check_expr(guard);
                self.require_type(&guard_t, &Type::Bool, keyword);
            }

            let actual_stmts = if let [Stmt::Block(inner)] = case.body.as_slice() {
                inner.as_slice()
            } else {
                case.body.as_slice()
            };
            let mut terminated = false;
            let mut yielded = Type::Void;
            for (index, stmt) in actual_stmts.iter().enumerate() {
                if terminated {
                    break;
                }
                if let Stmt::Expression(e) = stmt {
                    let ty = if index + 1 == actual_stmts.len() {
                        if let Some(expected) = expected {
                            self.check_expr_with_expectation(e, expected, keyword)
                        } else {
                            self.check_expr(e)
                        }
                    } else {
                        self.check_expr(e)
                    };
                    if index + 1 == actual_stmts.len() {
                        yielded = ty;
                    }
                } else {
                    self.check_stmt(stmt);
                }
                terminated = self.finalizes_execution(stmt);
            }
            if !terminated {
                // A live arm without a final value still has type void. It
                // cannot acquire a storable result merely from its context.
                if let Some(expected) = expected {
                    self.require_type(&yielded, expected, keyword);
                }
                return_types.push(yielded);
            }
            self.end_scope();
        }

        if return_types.is_empty() {
            Type::Void
        } else if let Some(expected)=expected {
            // Each yielding arm has already been checked against this context.
            // A fixed array arm may coerce to a slice alongside a slice remainder.
            expected.clone()
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

    fn is_place(&self, expr: &Expr) -> bool {
        match expr.untyped() {
            Expr::Variable(n) => {
                self.resolve_variable(n).is_some()
                    && (!self.function_names.contains(&n.lexeme)
                        || self
                            .scopes
                            .iter()
                            .skip(1)
                            .any(|s| s.contains_key(&n.lexeme)))
            }
            Expr::This(_) => true,
            Expr::Get { name, .. } => self.property_indices.contains_key(name),
            Expr::SubscriptGet { .. } | Expr::Dereference { .. } => true,
            Expr::Grouping(e) => self.is_place(e),
            _ => false,
        }
    }

    fn immutable_reference_value(&self, expr: &Expr) -> bool {
        self.expression_types.get(&(expr as *const Expr as usize))
            .is_some_and(|t| matches!(self.resolve_type_vars(t.clone()), Type::Reference(_)))
    }

    fn immutable_place(&self, expr: &Expr) -> bool {
        match expr.untyped() {
            Expr::Grouping(e) => self.immutable_place(e),
            Expr::Dereference {operand,..} => self.immutable_reference_value(operand),
            Expr::Get {object,..} => self.immutable_reference_value(object) || self.immutable_place(object),
            Expr::SubscriptGet {indexee,..} => self.immutable_place(indexee),
            _ => false,
        }
    }

    fn require_writable_place(&mut self, expr: &Expr, token: &Token) {
        if self.immutable_place(expr) {
            self.error(token, "Cannot modify an immutable reference. Mutable access requires a mutable place.");
        }
    }

    fn check_default_expression(
        &mut self,
        expr: &Expr,
        ty: &Type,
        token: &Token,
        later: &[FunctionParam],
    ) {
        let forbidden = later.iter().map(|p| p.name.lexeme.clone()).collect();
        let previous = self
            .forbidden_default_parameters
            .replace((forbidden, self.scopes.len()));
        self.check_expr_with_expectation(expr, ty, token);
        self.forbidden_default_parameters = previous;
    }

    fn default_arity(&mut self, count: usize, params: &[FunctionParam], token: &Token) {
        let required = params
            .iter()
            .rposition(|p| p.default_value.is_none())
            .map_or(0, |i| i + 1);
        if count < required || count > params.len() {
            self.error(
                token,
                &format!(
                    "Expected {required}..{} arguments but got {count}.",
                    params.len()
                ),
            );
        }
    }

    fn elaborate_defaults(
        &mut self,
        params: &[FunctionParam],
        types: &[Type],
        count: usize,
        instance: bool,
        owner: Option<&str>,
        token: &Token,
    ) {
        let supplied = count + usize::from(instance);
        if supplied >= params.len() || params[supplied..].iter().any(|p| p.default_value.is_none())
        {
            return;
        }
        let globals = self.scopes[0].clone();
        let scopes = std::mem::replace(&mut self.scopes, vec![globals, HashMap::new()]);
        let old_owner = self.current_struct.clone();
        self.current_struct = owner.map(str::to_string);
        let old_ret = self.current_return_type.take();
        let old_static = self.is_in_static;
        self.is_in_static = owner.is_some() && !instance;
        let mut defaults = Vec::new();
        let mut parameters = Vec::new();
        for (i, (p, ty)) in params.iter().zip(types).enumerate() {
            let ty = self.resolve_type(ty);
            if i >= supplied {
                let signature = types
                    .iter()
                    .map(|t| self.final_type(t.clone()))
                    .collect::<Vec<_>>();
                let evaluator =
                    crate::symbols::generic(&format!("__fishy_default_{}", p.name.id), &signature);
                if !self.default_stack.insert(evaluator.clone()) {
                    // Recursive source substitution needs a finite representation.
                    // This helper evaluates only the default expression at runtime;
                    // it is not an overload of the called function or a new ABI.
                    self.default_evaluators_needed.insert(evaluator.clone());
                    let arguments = parameters
                        .iter()
                        .map(|(name, ty): &(Token, Type)| Expr::Typed {
                            expr: Box::new(if name.lexeme == "self" {
                                Expr::This(name.clone())
                            } else {
                                Expr::Variable(name.clone())
                            }),
                            ty: ty.clone(),
                        })
                        .collect();
                    let callee = Expr::Typed {
                        expr: Box::new(Expr::Variable(Token::synthetic(
                            TokenType::Identifier,
                            &evaluator,
                        ))),
                        ty: Type::Function(
                            parameters.iter().map(|(_, t)| t.clone()).collect(),
                            Box::new(ty.clone()),
                            false,
                        ),
                    };
                    defaults.push(Expr::Typed {
                        expr: Box::new(Expr::Call {
                            callee: Box::new(callee),
                            paren: Token::synthetic(TokenType::LeftParen, "("),
                            arguments,
                        }),
                        ty: ty.clone(),
                    });
                } else {
                    let mut stmt = Stmt::Expression(p.default_value.as_ref().unwrap().clone());
                    crate::ast_visit::fresh_instance(&mut stmt);
                    if let Stmt::Expression(e) = &stmt {
                        self.check_default_expression(e, &ty, &p.name, &params[i..]);
                    }
                    self.materialize(&mut stmt);
                    if let Stmt::Expression(e) = stmt {
                        if self.default_evaluators_needed.contains(&evaluator)
                            && !self.default_evaluators.contains_key(&evaluator)
                        {
                            let ps = parameters
                                .iter()
                                .map(|(name, ty)| FunctionParam {
                                    name: name.clone(),
                                    type_annotation: Some(ty.clone()),
                                    default_value: None,
                                })
                                .collect();
                            let helper = Stmt::Function {
                                name: Token::synthetic(TokenType::Identifier, &evaluator),
                                type_params: vec![],
                                params: ps,
                                return_type: Some(ty.clone()),
                                body: Some(vec![Stmt::Return {
                                    keyword: p.name.clone(),
                                    value: Some(e.clone()),
                                }]),
                                is_abstract: false,
                                is_async: false,
                                is_public: false,
                            };
                            self.default_evaluators.insert(evaluator.clone(), helper);
                        }
                        defaults.push(e);
                    }
                    self.default_stack.remove(&evaluator);
                }
            }
            self.declare_variable(&p.name, ty.clone());
            parameters.push((p.name.clone(), ty));
        }
        self.scopes = scopes;
        self.current_struct = old_owner;
        self.current_return_type = old_ret;
        self.is_in_static = old_static;
        self.default_calls.insert(
            token.clone(),
            DefaultCall {
                parameters,
                defaults,
                instance,
            },
        );
    }

    fn direct_default_declaration(&self, callee: &Expr) -> Option<(Stmt, Option<String>)> {
        match callee.untyped() {
            Expr::Grouping(inner)
                if matches!(inner.untyped(), Expr::Variable(_) | Expr::Grouping(_)) =>
            {
                self.direct_default_declaration(inner)
            }
            Expr::Variable(n)
                if !self
                    .scopes
                    .iter()
                    .skip(1)
                    .any(|s| s.contains_key(&n.lexeme)) =>
            {
                self.default_declarations
                    .get(&n.lexeme)
                    .cloned()
                    .map(|s| (s, None))
            }
            Expr::Get { name, .. } if !self.property_indices.contains_key(name) => {
                let owner = match self.resolved_calls.get(name)? {
                    CallType::Static(n) | CallType::Instance(n) => n,
                };
                self.default_declarations
                    .get(&crate::symbols::member(owner, &name.lexeme))
                    .cloned()
                    .map(|s| (s, Some(owner.clone())))
            }
            _ => None,
        }
    }

    fn direct_callee(mut expr: &Expr) -> &Expr {
        while let Expr::Grouping(inner) = expr.untyped() {
            expr = inner;
        }
        expr.untyped()
    }

    fn record_callee_type(&mut self, callee: &Expr, ty: &Type) {
        self.expression_types.insert(callee as *const Expr as usize, ty.clone());
        if let Expr::Grouping(inner) = callee.untyped() {
            self.record_callee_type(inner, ty);
        }
    }

    fn check_generic_method(
        &mut self,
        owner: &str,
        template: &Stmt,
        callee: &Expr,
        arguments: &[Expr],
        token: &Token,
        expected_return: Option<Type>,
    ) -> Type {
        let Stmt::Function {
            name,
            type_params,
            params,
            return_type,
            is_public,
            ..
        } = template
        else {
            unreachable!()
        };
        if !is_public && self.current_struct.as_deref() != Some(owner) {
            self.error(token, "Cannot access private member.");
        }
        let instance = params.first().is_some_and(|p| p.name.lexeme == "self");
        let mapping: HashMap<_, _> = type_params
            .iter()
            .map(|p| (p.name.lexeme.clone(), self.fresh_type_var()))
            .collect();
        let expected = params
            .iter()
            .skip(usize::from(instance))
            .map(|p| {
                self.substitute_type(p.type_annotation.as_ref().unwrap_or(&Type::Void), &mapping)
            })
            .collect::<Vec<_>>();
        self.default_arity(arguments.len(),&params[usize::from(instance)..],token);
        for (arg, expected) in arguments.iter().zip(&expected) {
            self.check_expr_with_expectation(arg, expected, token);
        }
        if let Some(expected_return)=expected_return {
            let ret=self.substitute_type(return_type.as_ref().unwrap_or(&Type::Void),&mapping);
            self.require_type(&ret,&expected_return,token);
        }
        let inferred = type_params.iter().map(|p| mapping[&p.name.lexeme].clone()).collect::<Vec<_>>();
        let concrete = self.infer_generic_arguments(type_params, &inferred, token);
        self.check_constraints(type_params, &concrete, token);
        let substitutions = type_params
            .iter()
            .zip(&concrete)
            .map(|(p, t)| (p.name.lexeme.clone(), t.clone()))
            .collect::<HashMap<_, _>>();
        let concrete_method=self.monomorphize_stmt(template,&substitutions);
        if let Stmt::Function {params,..}=&concrete_method {
            let mut types=expected.iter().map(|t|self.final_type(t.clone())).collect::<Vec<_>>();
            if instance {let self_ty=self.resolve_type(&Type::Custom(owner.into()));types.insert(0,if self.is_number_type(&self_ty)||matches!(self_ty,Type::Bool|Type::String) {self_ty}else{Type::MutReference(Box::new(self_ty))});}
            self.elaborate_defaults(params,&types,arguments.len(),instance,Some(owner),token);
        }
        let specialized =
            crate::symbols::generic(&crate::symbols::member(owner, &name.lexeme), &concrete);
        let symbol = crate::symbols::member(owner, &specialized);
        if self.instantiated_types.insert(symbol.clone()) {
            let mut method = self.monomorphize_stmt(template, &substitutions);
            if let Stmt::Function {
                name, type_params, ..
            } = &mut method
            {
                name.lexeme = specialized;
                type_params.clear();
            }
            self.instantiations.push(Stmt::Impl {
                keyword: token.clone(),
                target_type: Type::Custom(owner.into()),
                type_params: vec![],
                trait_name: None,
                methods: vec![method],
            });
        }
        if let Expr::Get { name, .. } = Self::direct_callee(callee) {
            self.specialized_methods
                .insert(name.clone(), (symbol, instance));
        }
        let ret = self.substitute_type(return_type.as_ref().unwrap_or(&Type::Void), &substitutions);
        let ret = self.resolve_type(&ret);
        let ps = expected
            .iter()
            .map(|t| self.substitute_type(&self.resolve_type_vars(t.clone()), &substitutions))
            .collect();
        self.record_callee_type(callee, &Type::Function(ps, Box::new(ret.clone()), false));
        ret
    }

    fn check_expr(&mut self, expr: &Expr) -> Type {
        let unsupported=match expr {
            Expr::Typeof(token)=>Some(token.clone()),
            Expr::Await {keyword,..}=>Some(keyword.clone()),

            Expr::Lazy {..}=>Some(Token::synthetic(TokenType::Lazy,"lazy")),
            _=>None,
        };
        if let Some(token)=unsupported {self.error(&token,"This expression is not implemented by the backend.");return Type::Void;}
        let ty=self.check_expr_inner(expr);
        self.expected_types.remove(&(expr as *const Expr as usize));
        self.expression_types.insert(expr as *const Expr as usize,ty.clone());
        ty
    }

    fn check_expr_inner(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Typed {ty,..} => ty.clone(),
            Expr::StorageSize(t) => {self.resolve_type(t);Type::U64}
            Expr::Interpolation {token,parts} => {
                for part in parts {
                    let ty=self.check_expr(part);
                    let ty=self.resolve_type_vars(ty);
                    if !matches!(ty,Type::String|Type::Bool|Type::TypeVar(_)) && !self.is_number_type(&ty) {
                        self.error(token,"Interpolation supports strings, booleans and numbers.");
                    }
                }
                Type::String
            }
            Expr::Literal(lit) =>
                match lit {
                    Literal::Number(_) => Type::F64,
                    Literal::Integer(n) => {
                        let ty=self.fresh_type_var();self.integer_literals.push((ty.clone(),*n));ty
                    },
                    Literal::String(_) => Type::String,
                    Literal::Bool(_) => Type::Bool,
                    Literal::None => Type::Void,
                }

            Expr::Variable(name) => {
                if let Some((forbidden,depth))=&self.forbidden_default_parameters {
                    if forbidden.contains(&name.lexeme) && !self.scopes.iter().skip(*depth).any(|s|s.contains_key(&name.lexeme)) {
                        self.error(name,"Default argument cannot reference its own or a later parameter.");return Type::Void;
                    }
                }
                if let Some((t, state)) = self.resolve_variable(name) {
                    if !self.scopes.iter().skip(1).any(|s|s.contains_key(&name.lexeme)) {
                        if let Some(constructor)=self.enum_constructors.get(&name.lexeme).cloned() {
                            if let Some(qualified)=constructor {self.resolved_calls.insert(name.clone(),CallType::Static(qualified));}
                            else{self.error(name,"Ambiguous enum constructor; qualify it with the enum type.");}
                        }
                    }
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
                    if matches!(self.templates.get(&name.lexeme), Some(Stmt::Function {..})) {
                        self.error(name, "Generic function values require a direct call site for inference.");
                        return Type::Void;
                    }
                    let mut real_name = name.lexeme.clone();
                    if let Some(Type::Custom(aliased)) = self.aliases.get(&name.lexeme) {
                        real_name = aliased.clone();
                    }

                    if let Some(info) = self.user_types.get(&real_name) {
                        if !self.declared_structs.contains(&real_name)&&!self.user_types.get(&real_name).is_some_and(|i|!i.case_names.is_empty()) {
                            self.error(name,"This type name is not a value constructor.");return Type::Void;
                        }
                        if info.methods.contains_key("init") {self.error(name,"Initializer method calling convention requires a language design decision.");return Type::Void;}
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
                if self.function_names.contains(&name.lexeme)
                    && !self.scopes.iter().skip(1).any(|s| s.contains_key(&name.lexeme))
                {
                    self.error(name, "Cannot assign to a function declaration.");
                }
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
                if matches!(left.untyped(),Expr::Literal(Literal::None))&&matches!(operator.token_type,TokenType::EqualEqual|TokenType::BangEqual|TokenType::QuestionQuestion) {
                    let rt=self.check_expr(right);
                    if !matches!(rt,Type::Void) {self.check_expr_with_expectation(left,&rt,operator);}
                    return if operator.token_type==TokenType::QuestionQuestion {rt}else{Type::Bool};
                }
                let lt = self.check_expr(left);
                let rt = self.check_expr_with_expectation(right, &lt, operator);

                let lt = self.resolve_type_vars(lt);
                let rt = self.resolve_type_vars(rt);

                match operator.token_type {
                    TokenType::DotDot => {
                        if !self.is_integer_type(&lt) && !matches!(lt,Type::TypeVar(_)) {
                            self.error(operator,"Range bounds must have compatible integer types.");
                        }
                        Type::Range(Box::new(lt))
                    }
                    TokenType::Plus => {
                        if lt==Type::String&&rt==Type::String {Type::String}
                        else {
                            if !self.is_number_type(&lt)&&!matches!(lt,Type::TypeVar(_)) {self.error(operator,"Addition requires numbers or two strings.");}
                            self.require_type(&rt,&lt,operator);lt
                        }
                    }
                    TokenType::Minus | TokenType::Star | TokenType::Slash | TokenType::Percent => {
                        if !self.is_number_type(&lt)&&!matches!(lt,Type::TypeVar(_)) {self.error(operator,"Arithmetic requires numeric operands.");}
                        self.require_type(&rt, &lt, operator);
                        lt
                    }
                    | TokenType::Greater
                    | TokenType::GreaterEqual
                    | TokenType::Less
                    | TokenType::LessEqual => {
                        if !self.is_number_type(&lt)&&!matches!(lt,Type::TypeVar(_)) {self.error(operator,"Ordering requires numeric operands.");}
                        self.require_type(&rt, &lt, operator);
                        Type::Bool
                    }
                    TokenType::EqualEqual | TokenType::BangEqual => {
                        if matches!(lt,Type::Range(_)) {self.error(operator,"Range value equality is not defined; compare integer bounds explicitly.");}
                        self.require_type(&rt, &lt, operator);
                        Type::Bool
                    }
                    TokenType::Exponentiation => {
                        if !self.is_number_type(&lt)&&!matches!(lt,Type::TypeVar(_)) {self.error(operator,"Exponentiation requires numeric operands.");}
                        self.require_type(&rt, &lt, operator);
                        lt
                    }
                    TokenType::QuestionQuestion => {
                        if !matches!(lt,Type::Pointer(_)|Type::String) {self.error(operator,"Null coalescing requires a pointer or string.");}
                        self.require_type(&rt,&lt,operator);lt
                    }
                    TokenType::BitwiseAnd | TokenType::BitwiseOr => {
                        if !self.is_integer_type(&lt)&&!matches!(lt,Type::TypeVar(_)) {self.error(operator,"Bitwise operators require integer operands.");}
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
                if operator.token_type==TokenType::Minus {
                    if let Expr::Literal(Literal::Integer(n))=right.untyped() {
                        let ty=self.fresh_type_var();self.integer_literals.push((ty.clone(),-*n));return ty;
                    }
                }
                let rt = self.check_expr(right);
                if matches!(operator.token_type,TokenType::PlusPlus|TokenType::MinusMinus) {
                    self.require_writable_place(right, operator);
                }
                match operator.token_type {
                    TokenType::Minus | TokenType::MinusMinus | TokenType::PlusPlus => {
                        let actual=self.resolve_type_vars(rt.clone());
                        if !self.is_number_type(&actual)&&!matches!(actual,Type::TypeVar(_)) {self.error(operator,"Numeric unary operator requires a number.");}
                        rt
                    }
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
                if let Expr::Get {object,name}=Self::direct_callee(callee) {
                    let owner=if let Expr::Variable(n)=object.untyped() {
                        if self.resolve_variable(n).is_none() {
                            if matches!(self.templates.get(&n.lexeme), Some(Stmt::Struct {..}|Stmt::Enum {..})) {
                                Some(n.lexeme.clone())
                            } else {
                                let resolved=self.resolve_type(&Type::Custom(n.lexeme.clone()));
                                self.type_name(&resolved).filter(|n|self.user_types.contains_key(n))
                            }
                        } else {None}
                    }else{None};
                    let owner=owner.or_else(||{
                        let ty=self.check_expr(object);let ty=Self::unwrap_indirection(self.resolve_type_vars(ty));self.type_name(&ty)
                    });
                    if let Some(owner)=owner {
                        if let Some(template)=self.method_templates.get(&(owner.clone(),name.lexeme.clone())).cloned() {
                            let expected=self.expected_types.get(&(expr as *const Expr as usize)).cloned();
                            return self.check_generic_method(&owner,&template,callee,arguments,paren,expected);
                        }
                    }
                }
                if let Expr::Variable(var_name) = Self::direct_callee(callee) {
                    if
                        let Some(Stmt::Function { type_params, params, return_type, .. }) =
                            self.templates.get(&var_name.lexeme).cloned().filter(|_|!self.scopes.iter().skip(1).any(|s|s.contains_key(&var_name.lexeme)))
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

                        self.default_arity(arguments.len(),&params,paren);

                        for (i, arg) in arguments.iter().enumerate() {
                            if i < expected_params.len() {
                                self.check_expr_with_expectation(arg, &expected_params[i], paren);
                            } else {
                                self.check_expr(arg);
                            }
                        }

                        if let Some(expected)=self.expected_types.get(&(expr as *const Expr as usize)).cloned() {
                            let ret=self.substitute_type(return_type.as_ref().unwrap_or(&Type::Void),&mapping);
                            self.require_type(&ret,&expected,paren);
                        }
                        let concrete_args = self.infer_generic_arguments(&type_params, &fresh_args, paren);
                        let concrete_mapping = type_params.iter().zip(&concrete_args)
                            .map(|(p, t)| (p.name.lexeme.clone(), t.clone())).collect::<HashMap<_, _>>();

                        let concrete_params=params.iter().map(|p|FunctionParam {name:p.name.clone(),type_annotation:p.type_annotation.as_ref().map(|t|self.substitute_type(t,&concrete_mapping)),default_value:p.default_value.as_ref().map(|e| {
                            let stmt=self.monomorphize_stmt(&Stmt::Expression(e.clone()),&concrete_mapping);
                            if let Stmt::Expression(e)=stmt {e}else{unreachable!()}
                        })}).collect::<Vec<_>>();
                        let types=expected_params.iter().map(|t|self.final_type(t.clone())).collect::<Vec<_>>();
                        self.elaborate_defaults(&concrete_params,&types,arguments.len(),false,None,paren);
                        self.check_constraints(&type_params,&concrete_args,var_name);
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
                        let final_ret = self.resolve_type(&final_ret);
                        let concrete_types = types.iter().map(|t|self.resolve_type(t)).collect();
                        self.record_callee_type(callee, &Type::Function(concrete_types, Box::new(final_ret.clone()), false));
                        return final_ret;
                    }
                }

                let callee_type = self.check_expr(callee);
                let callee_type = self.resolve_type_vars(callee_type);

                match callee_type {
                    Type::Function(param_types, ret_type, is_variadic) => {
                        let expected_param_count=param_types.len();
                        let declaration=self.direct_default_declaration(callee);
                        let has_defaults=declaration.as_ref().is_some_and(|(s,_)|matches!(s,Stmt::Function {params,..} if params.iter().any(|p|p.default_value.is_some())));
                        if has_defaults {
                            if let Some((Stmt::Function {params,..},_))=&declaration {
                                let instance=params.first().is_some_and(|p|p.name.lexeme=="self");
                                self.default_arity(arguments.len(),&params[usize::from(instance)..],paren);
                            }
                        } else if is_variadic {
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
                            if arguments.len()<expected_param_count && declaration.is_none() {
                                self.error(paren,"Calls through function values require the full parameter signature; defaults are available on direct calls.");
                            }
                            self.error(
                                paren,
                                &format!(
                                    "Expected {} arguments but got {}.",
                                    expected_param_count,
                                    arguments.len()
                                )
                            );
                        }

                        let param_offset = 0;

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

                        if let Some((Stmt::Function {params,..},owner))=declaration {
                            let instance=params.first().is_some_and(|p|p.name.lexeme=="self");
                            let mut types=param_types.clone();
                            if instance {
                                let owner_type=self.resolve_type(&Type::Custom(owner.clone().unwrap()));
                                let self_ty=if self.is_number_type(&owner_type)||matches!(owner_type,Type::String|Type::Bool) {owner_type}else{Type::MutReference(Box::new(owner_type))};
                                types.insert(0,self_ty);
                            }
                            self.elaborate_defaults(&params,&types,arguments.len(),instance,owner.as_deref(),paren);
                        }
                        let mut final_ret = self.resolve_type_vars(*ret_type);

                        if let Type::Generic(base_name, args) = &final_ret {
                            let type_params = match self.templates.get(base_name) {
                                Some(Stmt::Struct {type_params,..}|Stmt::Enum {type_params,..}) => type_params.clone(),
                                _ => vec![],
                            };
                            let concrete_args = self.infer_generic_arguments(&type_params, args, paren);
                            let concrete_generic = Type::Generic(base_name.clone(), concrete_args);

                            let mut overridden = false;
                            if let Some(Type::Custom(expected_mangled)) = &self.current_return_type {
                                if self.specialized_types.get(expected_mangled).is_some_and(|(name,_)|name==base_name) {
                                    final_ret = Type::Custom(expected_mangled.clone());
                                    overridden = true;
                                }
                            }

                            if !overridden {
                                final_ret = self.resolve_type(&concrete_generic);
                            }

                            if let Expr::Get { name, .. } = Self::direct_callee(callee) {
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
                // A lexical value binding takes precedence over a type with
                // the same spelling, including generic constructor owners.
                if let Some(var_name) = match object.untyped() {
                    Expr::Variable(n) if self.resolve_variable(n).is_none() => Some(n),
                    _ => None,
                } {
                    let mut real_name = var_name.lexeme.clone();
                    if let Some(Type::Custom(aliased)) = self.aliases.get(&var_name.lexeme) {
                        real_name = aliased.clone();
                    }

                    if let Some(template_stmt) = self.templates.get(&real_name).cloned() {
                        let type_params = match &template_stmt {
                            Stmt::Struct { type_params, .. } | Stmt::Enum { type_params, .. } =>
                                type_params,
                            _ => {self.error(name,"Function templates do not have static members.");return Type::Void;},
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

                if let Type::Tuple(types)=&obj_t {
                    if let Ok(index)=name.lexeme.parse::<usize>() {
                        if let Some(ty)=types.get(index) {
                            self.property_indices.insert(name.clone(),index);return ty.clone();
                        }
                    }
                    self.error(name,"Tuple field index is out of range.");return Type::Void;
                }
                if let Some(class_name) = self.type_name(&obj_t) {
                    if self.method_templates.contains_key(&(class_name.clone(),name.lexeme.clone())) {self.error(name,"Generic method values require a call site for inference.");return Type::Void;}
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
                        return if index_opt.is_none()&&is_instance {
                            if let Type::Function(ps,ret,var)=m_type {Type::Function(ps.into_iter().skip(1).collect(),ret,var)}else{m_type}
                        }else{m_type};
                    }
                    self.error(name, "Member not found on this type.");
                } else {
                    self.error(name, "Type does not have properties or methods.");
                }
                Type::Void
            }

            Expr::Set { object, name, value } => {
                let object_type=self.check_expr(object);
                if matches!(object_type,Type::Reference(_)) {self.error(name,"Cannot assign through an immutable reference.");}
                self.require_writable_place(object, name);
                let obj_t = Self::unwrap_indirection(object_type);
                let obj_t = self.resolve_type_vars(obj_t);

                if let Type::Tuple(types)=&obj_t {
                    if let Ok(index)=name.lexeme.parse::<usize>() {
                        if let Some(ty)=types.get(index) {
                            self.property_indices.insert(name.clone(),index);
                            return self.check_expr_with_expectation(value,ty,name);
                        }
                    }
                    self.error(name,"Tuple field index is out of range.");return Type::Void;
                }
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
                            }else{self.error(name,"Cannot assign to a method.");}
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
                    if let Some(expected) = self.expected_types.get(&(expr as *const Expr as usize)) {
                        let origin = match self.resolve_type_vars(expected.clone()) {
                            Type::Custom(ref n) => self.specialized_types.get(n).cloned(),
                            Type::Generic(n, args) => Some((n, args)),
                            _ => None,
                        };
                        if let Some((base, args)) = origin {
                            if base == class_name.lexeme {
                                actual_type_args = args;
                            }
                        }
                    }
                }
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

                if let Type::Generic(base,args)=&base_type {
                    if let Some(Stmt::Struct {type_params,fields,..})=self.templates.get(base).cloned() {
                        let subs:HashMap<_,_>=type_params.iter().zip(args).map(|(p,t)|(p.name.lexeme.clone(),t.clone())).collect();
                        for (name,expr) in properties {
                            if let Some((_,ty,_))=fields.iter().find(|(n,_,_)|n.lexeme==name.lexeme) {
                                let ty=self.substitute_type(ty,&subs);
                                self.check_expr_with_expectation(expr,&ty,name);
                            }
                        }
                    }
                }
                let base_type=match base_type {
                    Type::Generic(n,args)=> {
                        let params = match self.templates.get(&n) {
                            Some(Stmt::Struct {type_params,..}|Stmt::Enum {type_params,..}) => type_params.clone(),
                            _ => vec![],
                        };
                        let args = self.infer_generic_arguments(&params, &args, class_name);
                        Type::Generic(n,args)
                    }
                    t=>t,
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
                                if !seen_fields.insert(name.lexeme.clone()) {self.error(name,"Duplicate initializer field.");}
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
                                concrete_args.push(self.final_type(arg.clone()));
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

            Expr::Match {value,cases,keyword}=>self.check_match(value,cases,keyword,None),

            | Expr::WildcardPattern(_)
            | Expr::UnionPattern { .. }
            | Expr::ListPattern { .. }
            | Expr::ObjectPattern { .. } => {
                self.error(&Token::synthetic(TokenType::Match,"pattern"),"Pattern cannot be used as a value.");Type::Void
            },

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

            Expr::Spread {operator,..}=>{
                self.error(operator,"Spread is only valid inside an array literal.");Type::Void
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

            Expr::AddressOf {operand,operator}=>{
                let inner=self.check_expr(operand);
                if !self.is_place(operand) {self.error(operator,"Address-of requires an assignable storage place.");}
                Type::Pointer(Box::new(inner))
            }

            Expr::Dereference { operator, operand } => {
                let operand_type = self.check_expr(operand);
                let operand_type = self.resolve_type_vars(operand_type);

                match operand_type {
                    Type::Pointer(inner) | Type::Reference(inner) | Type::MutReference(inner) => {
                        if *inner==Type::Void {self.error(operator,"Cannot dereference a void pointer.");}
                        *inner
                    },
                    _ => {
                        self.error(operator, "Cannot dereference a non-pointer/reference value.");
                        Type::Void
                    }
                }
            }

            Expr::DereferenceSet { operator, ptr, value } => {
                let ptr_type = self.check_expr(ptr);
                let ptr_type = self.resolve_type_vars(ptr_type);

                if matches!(ptr_type,Type::Reference(_)) {self.error(operator,"Cannot assign through an immutable reference.");}

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
                let trait_cast=match (&source,&target) {
                    (Type::Custom(src),Type::Custom(tr)) if self.traits.contains(tr)=>{
                        if !self.trait_implementations.contains(&(src.clone(),tr.clone())) {self.error(operator,"Type does not implement this trait.");}
                        true
                    }
                    _=>false,
                };
                if !(scalar(&source)&&scalar(&target)) && source!=target && !trait_cast {
                    self.error(operator,"Unsupported cast between these types.");
                }
                if (matches!(source,Type::F16|Type::F32|Type::F64)&&matches!(target,Type::Pointer(_)|Type::String)) || (matches!(target,Type::F16|Type::F32|Type::F64)&&matches!(source,Type::Pointer(_)|Type::String)) {self.error(operator,"Casts between floating values and pointers require an integer intermediate.");}
                if target==Type::Void || source==Type::Void {self.error(operator,"Void is not a value and cannot be cast.");}
                target
            }

            Expr::Ternary { true_token, condition, then_branch, else_branch } => {
                let cond_t = self.check_expr(condition);
                self.require_type(&cond_t, &Type::Bool, true_token);
                if self.diverges(then_branch) {self.check_expr(then_branch);return self.check_expr(else_branch);}
                let tt = self.check_expr(then_branch);
                self.check_expr_with_expectation(else_branch,&tt,true_token);
                self.resolve_type_vars(tt)
            }

            Expr::TupleLiteral { elements, .. } => {
                Type::Tuple(
                    elements
                        .iter()
                        .map(|e| self.check_expr(e))
                        .collect()
                )
            }

            Expr::Array {elements,bracket}=>{
                let (length,inner)=self.check_array_elements(elements,None,bracket);
                Type::Array(length,Box::new(inner))
            }

            Expr::SubscriptGet { indexee, bracket, index } => {
                let target_type = self.check_expr(indexee);
                let index_type = self.check_expr(index);
                if matches!(self.resolve_type_vars(index_type.clone()),Type::TypeVar(_)) {self.require_type(&index_type,&Type::I64,bracket);}

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
                self.require_writable_place(indexee, bracket);
                let index_type = self.check_expr(index);
                if matches!(self.resolve_type_vars(index_type.clone()),Type::TypeVar(_)) {self.require_type(&index_type,&Type::I64,bracket);}

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

            Expr::Lambda { params, return_type, body, is_async } => {
                if *is_async {self.error(&Token::synthetic(TokenType::Async,"async"),"Async execution requires a language/runtime design decision.");}
                self.begin_scope();
                let mut p_types = Vec::new();
                for p in params {
                    if let Some(t) = &p.type_annotation {
                        let t=self.resolve_type(t);
                        if !Self::valid_storage_type(&t) {
                            self.error(&p.name, "Parameter must have a supported non-void value type.");
                        }
                        p_types.push(t.clone());
                        self.declare_variable(&p.name, t);
                    } else {
                        self.error(&p.name, "Lambda parameters require explicit typing.");
                        p_types.push(Type::Void);
                    }
                }

                let ret_t = self.resolve_type(return_type.as_ref().unwrap_or(&Type::Void));
                let old_ret = self.current_return_type.replace(ret_t.clone());

                for s in body {
                    self.check_stmt(s);
                }

                if ret_t!=Type::Void && ret_t!=Type::Tuple(vec![]) && !body.iter().any(|s|self.finalizes_execution(s)) {
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
        if self.scopes.len()>1 {
            let declaration=match stmt {
                Stmt::Alias {name,..}|Stmt::Struct {name,..}|Stmt::Enum {name,..}|Stmt::Trait {name,..}|Stmt::ExternFunction {name,..}=>Some(name),
                Stmt::Impl {keyword,..}=>Some(keyword),_=>None,
            };
            if let Some(token)=declaration {self.error(token,"Type and external declarations are only supported at module scope.");return;}
        }
        match stmt {
            Stmt::Export {keyword,..} => {self.error(keyword,"Export visibility is only valid at module scope.");}
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
                        self.check_expr_with_expectation(expr,&expected,name);
                        // The initializer may retain a concrete array allocation
                        // type, but the binding obeys its declared slice type.
                        expected
                    }
                    (Some(annot), None) => self.resolve_type(annot),
                    (None, Some(expr)) => self.check_expr(expr),
                    (None, None) => Type::Void,
                };
                if !Self::valid_storage_type(&var_type) {self.error(name,"A variable must have a supported non-void value type (variadic functions require direct calls).");}

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
                    Type::Array(_, inner) | Type::Slice(inner) | Type::Range(inner) => (Type::I64, *inner),
                    _ => {
                        self.error(
                            key,
                            "'for in' iterator can only be used with arrays, slices or integer ranges."
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
                if *is_async {self.error(name,"Async execution requires a language/runtime design decision.");}
                let mut has_default=false;
                for p in params {
                    if p.default_value.is_some() {has_default=true;}
                    else if has_default {self.error(&p.name,"Default parameters must be trailing.");}
                }
                if self.current_return_type.is_some() {self.error(name,"Nested named functions are not supported; use a lambda.");return;}
                if !type_params.is_empty() {
                    for p in params {if p.name.lexeme!="self"&&p.type_annotation.is_none() {self.error(&p.name,"Generic function parameters require explicit typing.");}}
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

                for (p,t) in params.iter().zip(&p_types) {if !Self::valid_storage_type(t) {self.error(&p.name,"Parameter must have a supported non-void value type.");}}
                let signature=Type::Function(p_types.clone(),Box::new(ret_t.clone()),false);
                if !self.scopes.last().is_some_and(|s|s.contains_key(&name.lexeme)) {self.declare_variable(name,signature);}

                self.begin_scope();
                let old_async = self.is_in_async;
                self.is_in_async = *is_async;
                let old_ret = self.current_return_type.replace(ret_t.clone());

                for (i, p) in params.iter().enumerate() {
                    if let Some(default)=&p.default_value {
                        self.check_default_expression(default,&p_types[i],&p.name,&params[i..]);
                    }
                    self.declare_variable(&p.name, p_types[i].clone());
                }

                if let Some(body_stmts) = body {
                    for s in body_stmts {
                        self.check_stmt(s);
                    }

                    if
                        ret_t != Type::Void && ret_t!=Type::Tuple(vec![]) &&
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
                        if expected==Type::Void && matches!(val_expr.untyped(),Expr::TupleLiteral {elements,..} if elements.is_empty()) {
                            self.check_expr(val_expr);
                            self.expression_types.insert(val_expr as *const Expr as usize,Type::Void);
                        } else {self.check_expr_with_expectation(val_expr, &expected, keyword);}
                    } else if expected != Type::Void && expected!=Type::Tuple(vec![]) {
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

                if !matches!(init_t,Type::Array(..)|Type::Slice(_)|Type::Tuple(_)) {
                    self.error(keyword,"Destructuring requires an array, slice, or tuple.");
                }
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

            Stmt::Using {keyword,..}=>self.error(keyword,"Imports are only supported at module scope."),

            Stmt::Trait { name, methods, .. } => {
                self.begin_scope();
                let old_struct = self.current_struct.replace(name.lexeme.clone());
                for m in methods {
                    self.check_stmt(m);
                }
                self.current_struct = old_struct;
                self.end_scope();
            }

            Stmt::Impl { keyword, target_type, methods, type_params, .. } => {
                if !type_params.is_empty() {return;}
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

            Stmt::Enum {..}=>{}

        }
    }
}

#[cfg(test)]
#[path = "semantic_tests.rs"]
mod tests;

//! Resolve module bindings before flattening compilation units. Only names, not
//! declarations, cross an import boundary. Locals, fields and generic binders
//! retain their lexical identities.
use crate::ast::*;
use crate::token::Token;
use std::collections::{ HashMap, HashSet };

pub struct Names<'a> {
    pub globals: &'a HashMap<String, String>,
    pub prelude: &'a HashSet<String>,
    pub constructors: HashMap<String, Option<(String, String)>>,
    locals: Vec<HashSet<String>>,
    generics: Vec<HashSet<String>>,
    module: usize,
}
impl<'a> Names<'a> {
    pub fn new(
        globals: &'a HashMap<String, String>,
        prelude: &'a HashSet<String>,
        module: usize
    ) -> Self {
        Self {
            globals,
            prelude,
            constructors: HashMap::new(),
            locals: vec![],
            generics: vec![],
            module,
        }
    }
    fn local(&self, n: &str) -> bool {
        self.locals
            .iter()
            .rev()
            .any(|s| s.contains(n))
    }
    fn bind(&mut self, n: &Token) {
        if let Some(scope) = self.locals.last_mut() {
            scope.insert(n.lexeme.clone());
        }
    }
    fn global(&self, n: &str) -> String {
        self.globals
            .get(n)
            .cloned()
            .unwrap_or_else(|| {
                if self.prelude.contains(n) {
                    n.into()
                } else {
                    format!("__fishy_hidden_{}_{}", self.module, n)
                }
            })
    }
    fn declaration(&self, n: &mut Token) {
        n.lexeme = self.global(&n.lexeme);
    }
    fn variable(&self, n: &mut Token) {
        if !self.local(&n.lexeme) {
            self.declaration(n);
        }
    }
    fn ty(&mut self, t: &mut Type) {
        match t {
            Type::Custom(n) | Type::Generic(n, _) => {
                if !self.generics.iter().any(|s| s.contains(n)) {
                    *n = self.global(n);
                }
            }
            _ => {}
        }
        match t {
            | Type::Pointer(t)
            | Type::Reference(t)
            | Type::MutReference(t)
            | Type::Slice(t)
            | Type::Range(t)
            | Type::Array(_, t) => self.ty(t),
            Type::Tuple(ts) | Type::Union(ts) | Type::Generic(_, ts) => {
                for t in ts {
                    self.ty(t);
                }
            }
            Type::Function(ps, r, _) => {
                for t in ps {
                    self.ty(t);
                }
                self.ty(r);
            }
            _ => {}
        }
    }
    fn generics(&mut self, ps: &mut [GenericParam]) {
        self.generics.push(
            ps
                .iter()
                .map(|p| p.name.lexeme.clone())
                .collect()
        );
        for p in ps {
            for t in &mut p.constraints {
                self.ty(t);
            }
        }
    }
    fn params(&mut self, ps: &mut [FunctionParam]) {
        // All binders shadow globals, including forbidden forward references in
        // defaults; the checker diagnoses those using the original names.
        for p in ps.iter() {
            self.bind(&p.name);
        }
        for p in ps {
            if let Some(t) = &mut p.type_annotation {
                self.ty(t);
            }
            if let Some(e) = &mut p.default_value {
                self.expr(e);
            }
        }
    }
    fn pattern(&mut self, e: &mut Expr) {
        match e {
            Expr::Variable(n) => {
                if !self.constructors.contains_key(&n.lexeme) {
                    self.bind(n);
                }
            }
            Expr::UnionPattern { bindings, .. } => {
                for n in bindings {
                    self.bind(n);
                }
            }
            Expr::ListPattern { elements, rest } => {
                for p in elements {
                    self.pattern(p);
                }
                if let Some(rest) = rest {
                    self.pattern(rest);
                }
            }
            Expr::ObjectPattern { properties, rest } => {
                for (_, p) in properties {
                    self.pattern(p);
                }
                if let Some(rest) = rest {
                    self.pattern(rest);
                }
            }
            Expr::Typed { expr, .. } => self.pattern(expr),
            _ => {}
        }
    }
    fn block(&mut self, b: &mut [Stmt]) {
        self.locals.push(HashSet::new());
        self.statements(b);
        self.locals.pop();
    }
    pub fn statements(&mut self, b: &mut [Stmt]) {
        for s in b {
            self.stmt(s);
        }
    }
    fn stmt(&mut self, s: &mut Stmt) {
        match s {
            Stmt::Export { declaration, .. } => self.stmt(declaration),
            Stmt::Expression(e) => self.expr(e),
            Stmt::Block(b) => self.block(b),
            Stmt::Var { name, type_annotation, initializer, .. } => {
                if let Some(t) = type_annotation {
                    self.ty(t);
                }
                if let Some(e) = initializer {
                    self.expr(e);
                }
                if self.locals.is_empty() {
                    self.declaration(name);
                } else {
                    self.bind(name);
                }
            }
            Stmt::ArrayDestructuring { bindings, initializer, .. } => {
                self.expr(initializer);
                for n in bindings {
                    self.bind(n);
                }
            }
            Stmt::Alias { name, target } => {
                self.declaration(name);
                self.ty(target);
            }
            Stmt::ExternFunction { name, params, return_type, .. } => {
                self.declaration(name);
                self.locals.push(HashSet::new());
                self.params(params);
                if let Some(t) = return_type {
                    self.ty(t);
                }
                self.locals.pop();
            }
            Stmt::Function { name, type_params, params, return_type, body, .. } => {
                // Method declarations are visited separately so their member
                // names never acquire a module prefix.
                self.declaration(name);
                self.function(type_params, params, return_type, body);
            }
            Stmt::Struct { name, type_params, fields } => {
                self.declaration(name);
                self.generics(type_params);
                for (_, t, _) in fields {
                    self.ty(t);
                }
                self.generics.pop();
            }
            Stmt::Enum { name, type_params, cases } => {
                self.declaration(name);
                self.generics(type_params);
                for c in cases {
                    for (_, t) in &mut c.parameters {
                        if let Some(t) = t {
                            self.ty(t);
                        }
                    }
                }
                self.generics.pop();
            }
            Stmt::Trait { name, type_params, traits, methods } => {
                self.declaration(name);
                self.generics(type_params);
                for e in traits {
                    self.expr(e);
                }
                self.methods(methods);
                self.generics.pop();
            }
            Stmt::Impl { type_params, trait_name, target_type, methods, .. } => {
                self.generics(type_params);
                if let Some(t) = trait_name {
                    self.ty(t);
                }
                self.ty(target_type);
                self.methods(methods);
                self.generics.pop();
            }
            Stmt::If { condition, then_branch, else_branch, .. } => {
                self.expr(condition);
                self.stmt(then_branch);
                if let Some(s) = else_branch {
                    self.stmt(s);
                }
            }
            Stmt::While { condition, body, .. } => {
                self.expr(condition);
                self.stmt(body);
            }
            Stmt::For { initializer, condition, increment, body, .. } => {
                self.locals.push(HashSet::new());
                if let Some(s) = initializer {
                    self.stmt(s);
                }
                if let Some(e) = condition {
                    self.expr(e);
                }
                if let Some(e) = increment {
                    self.expr(e);
                }
                self.block(body);
                self.locals.pop();
            }
            Stmt::ForIn { key, value, iterable, body, .. } => {
                self.expr(iterable);
                self.locals.push(HashSet::new());
                self.bind(key);
                if let Some(n) = value {
                    self.bind(n);
                }
                self.block(body);
                self.locals.pop();
            }
            Stmt::Return { value, .. } => {
                if let Some(e) = value {
                    self.expr(e);
                }
            }
            Stmt::Throw { thrown, .. } => self.expr(thrown),
            Stmt::TryCatch { try_body, catch_body, exception, exception_type } => {
                self.block(try_body);
                self.locals.push(HashSet::new());
                self.bind(exception);
                if let Some(t) = exception_type {
                    self.ty(t);
                }
                self.block(catch_body);
                self.locals.pop();
            }
            Stmt::Using { .. } => {} // Local imports remain checker diagnostics.
        }
    }
    fn methods(&mut self, methods: &mut [Stmt]) {
        for s in methods {
            if let Stmt::Function { type_params, params, return_type, body, .. } = s {
                self.function(type_params, params, return_type, body);
            }
        }
    }
    fn function(
        &mut self,
        gs: &mut [GenericParam],
        ps: &mut [FunctionParam],
        ret: &mut Option<Type>,
        body: &mut Option<Vec<Stmt>>
    ) {
        self.generics(gs);
        self.locals.push(HashSet::new());
        self.params(ps);
        if let Some(t) = ret {
            self.ty(t);
        }
        if let Some(b) = body {
            self.statements(b);
        }
        self.locals.pop();
        self.generics.pop();
    }
    fn expr(&mut self, e: &mut Expr) {
        match e {
            Expr::Memory { element, arguments, .. } => {
                self.ty(element);
                for argument in arguments {
                    self.expr(argument);
                }
            }
            Expr::StorageSize(t) => self.ty(t),
            Expr::Typed { expr, ty, .. } => {
                self.ty(ty);
                self.expr(expr);
            }
            Expr::Variable(n) => {
                if !self.local(&n.lexeme) && !self.globals.contains_key(&n.lexeme) {
                    if let Some(Some((owner, case))) = self.constructors.get(&n.lexeme) {
                        let mut token = n.clone();
                        token.lexeme = owner.clone();
                        let mut name = n.clone();
                        name.lexeme = case.clone();
                        name.id = Token::fresh_id();
                        *e = Expr::Get {
                            object: Box::new(Expr::Variable(token)),
                            name,
                        };
                        return;
                    }
                }
                self.variable(n);
            }
            Expr::This(_) | Expr::Literal(_) | Expr::Typeof(_) | Expr::WildcardPattern(_) => {}
            | Expr::Grouping(e)
            | Expr::Unary { right: e, .. }
            | Expr::Spread { right: e, .. }
            | Expr::Dereference { operand: e, .. }
            | Expr::AddressOf { operand: e, .. } => self.expr(e),
            | Expr::Interpolation { parts, .. }
            | Expr::Array { elements: parts, .. }
            | Expr::TupleLiteral { elements: parts, .. } => {
                for e in parts {
                    self.expr(e);
                }
            }
            Expr::Binary { left, right, .. } | Expr::Logical { left, right, .. } => {
                self.expr(left);
                self.expr(right);
            }
            Expr::Ternary { condition, then_branch, else_branch, .. } => {
                self.expr(condition);
                self.expr(then_branch);
                self.expr(else_branch);
            }
            Expr::Assign { name, value } => {
                self.variable(name);
                self.expr(value);
            }
            Expr::Get { object, .. } => self.expr(object),
            Expr::Set { object, value, .. } => {
                self.expr(object);
                self.expr(value);
            }
            Expr::SubscriptGet { indexee, index, .. } => {
                self.expr(indexee);
                self.expr(index);
            }
            Expr::SubscriptSet { indexee, index, value, .. } => {
                self.expr(indexee);
                self.expr(index);
                self.expr(value);
            }
            Expr::DereferenceSet { ptr, value, .. } => {
                self.expr(ptr);
                self.expr(value);
            }
            Expr::Call { callee, arguments, .. } => {
                self.expr(callee);
                for e in arguments {
                    self.expr(e);
                }
            }

            Expr::Lambda { params, return_type, body, .. } => {
                self.locals.push(HashSet::new());
                self.params(params);
                if let Some(t) = return_type {
                    self.ty(t);
                }
                self.statements(body);
                self.locals.pop();
            }
            Expr::StructInit { class_name, type_args, properties, .. } => {
                self.declaration(class_name);
                for t in type_args {
                    self.ty(t);
                }
                for (_, e) in properties {
                    self.expr(e);
                }
            }
            Expr::Match { value, cases, .. } => {
                self.expr(value);
                for c in cases {
                    self.locals.push(HashSet::new());
                    self.pattern(&mut c.pattern);
                    if let Some(e) = &mut c.guard {
                        self.expr(e);
                    }
                    self.statements(&mut c.body);
                    self.locals.pop();
                }
            }
            Expr::Lazy { expr, statements } => {
                if let Some(e) = expr {
                    self.expr(e);
                }
                if let Some(b) = statements {
                    self.block(b);
                }
            }
            Expr::Cast { value, target_type, .. } => {
                self.expr(value);
                self.ty(target_type);
            }
            Expr::UnionPattern { .. } | Expr::ListPattern { .. } | Expr::ObjectPattern { .. } => {
                self.pattern(e)
            }
        }
    }
}

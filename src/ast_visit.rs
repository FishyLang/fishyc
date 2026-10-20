//! Exhaustive mutable traversal shared by type materialization and specialization.
use crate::ast::*;
use crate::token::Token;

pub trait Visitor {
    fn token(&mut self, _token: &mut Token) {}
    fn ty(&mut self, _ty: &mut Type) {}
    fn expr(&mut self, _expr: &mut Expr) {}
}

pub fn expr(e: &mut Expr, v: &mut impl Visitor) {
    match e {
        Expr::Typed { expr: e, ty } => {
            expr(e, v);
            v.ty(ty);
        }
        Expr::Literal(_) => {}
        Expr::StorageSize(t) => v.ty(t),
        Expr::Variable(t) | Expr::This(t) | Expr::Typeof(t) | Expr::WildcardPattern(t) => {
            v.token(t)
        }
        Expr::Grouping(e) => expr(e, v),
        Expr::Interpolation {token,parts} => {
            v.token(token);
            for part in parts {expr(part,v);}
        }
        Expr::Unary { operator, right } | Expr::Spread { operator, right } => {
            v.token(operator);
            expr(right, v);
        }
        Expr::Binary {
            left,
            operator,
            right,
        }
        | Expr::Logical {
            left,
            operator,
            right,
        } => {
            expr(left, v);
            v.token(operator);
            expr(right, v);
        }
        Expr::Ternary {
            condition,
            then_branch,
            else_branch,
            true_token,
        } => {
            expr(condition, v);
            expr(then_branch, v);
            expr(else_branch, v);
            v.token(true_token);
        }
        Expr::Assign { name, value } => {
            v.token(name);
            expr(value, v);
        }
        Expr::Get { object, name } => {
            expr(object, v);
            v.token(name);
        }
        Expr::Set {
            object,
            name,
            value,
        } => {
            expr(object, v);
            v.token(name);
            expr(value, v);
        }
        Expr::Array { bracket, elements }
        | Expr::TupleLiteral {
            token: bracket,
            elements,
        } => {
            v.token(bracket);
            for e in elements {
                expr(e, v);
            }
        }
        Expr::SubscriptGet {
            indexee,
            bracket,
            index,
        } => {
            expr(indexee, v);
            v.token(bracket);
            expr(index, v);
        }
        Expr::SubscriptSet {
            indexee,
            bracket,
            index,
            value,
        } => {
            expr(indexee, v);
            v.token(bracket);
            expr(index, v);
            expr(value, v);
        }
        Expr::Call {
            callee,
            paren,
            arguments,
        } => {
            expr(callee, v);
            v.token(paren);
            for a in arguments {
                expr(a, v);
            }
        }
        Expr::Lambda {
            params,
            return_type,
            body,
            ..
        } => {
            parameters(params, v);
            if let Some(t) = return_type {
                v.ty(t);
            }
            statements(body, v);
        }
        Expr::StructInit {
            class_name,
            type_args,
            properties,
            brace,
        } => {
            v.token(class_name);
            for t in type_args {
                v.ty(t);
            }
            for (t, e) in properties {
                v.token(t);
                expr(e, v);
            }
            v.token(brace);
        }
        Expr::Match {
            keyword,
            value,
            cases,
        } => {
            v.token(keyword);
            expr(value, v);
            for c in cases {
                expr(&mut c.pattern, v);
                if let Some(g) = &mut c.guard {
                    expr(g, v);
                }
                statements(&mut c.body, v);
            }
        }
        Expr::Lazy {
            expr: e,
            statements: body,
        } => {
            if let Some(e) = e {
                expr(e, v);
            }
            if let Some(b) = body {
                statements(b, v);
            }
        }
        Expr::Await { keyword, value } => {
            v.token(keyword);
            expr(value, v);
        }
        Expr::UnionPattern {
            case_name,
            bindings,
        } => {
            v.token(case_name);
            for b in bindings {
                v.token(b);
            }
        }
        Expr::ListPattern { elements, rest } => {
            for e in elements {
                expr(e, v);
            }
            if let Some(e) = rest {
                expr(e, v);
            }
        }
        Expr::ObjectPattern { properties, rest } => {
            for (t, e) in properties {
                v.token(t);
                expr(e, v);
            }
            if let Some(e) = rest {
                expr(e, v);
            }
        }
        Expr::AddressOf { operator, operand } | Expr::Dereference { operator, operand } => {
            v.token(operator);
            expr(operand, v);
        }
        Expr::DereferenceSet {
            operator,
            ptr,
            value,
        } => {
            v.token(operator);
            expr(ptr, v);
            expr(value, v);
        }
        Expr::Cast {
            value,
            operator,
            target_type,
        } => {
            expr(value, v);
            v.token(operator);
            v.ty(target_type);
        }
    }
    v.expr(e);
}

fn parameters(params: &mut [FunctionParam], v: &mut impl Visitor) {
    for p in params {
        v.token(&mut p.name);
        if let Some(t) = &mut p.type_annotation {
            v.ty(t);
        }
        if let Some(e) = &mut p.default_value {
            expr(e, v);
        }
    }
}
fn generics(params: &mut [GenericParam], v: &mut impl Visitor) {
    for p in params {
        v.token(&mut p.name);
        for t in &mut p.constraints {
            v.ty(t);
        }
    }
}
pub fn statements(stmts: &mut [Stmt], v: &mut impl Visitor) {
    for s in stmts {
        stmt(s, v);
    }
}
pub fn stmt(s: &mut Stmt, v: &mut impl Visitor) {
    match s {
        Stmt::Export {keyword,declaration} => {v.token(keyword);stmt(declaration,v);}
        Stmt::Expression(e) => expr(e, v),
        Stmt::Block(b) => statements(b, v),
        Stmt::Alias { name, target } => {
            v.token(name);
            v.ty(target);
        }
        Stmt::ExternFunction {
            name,
            params,
            return_type,
            ..
        } => {
            v.token(name);
            parameters(params, v);
            if let Some(t) = return_type {
                v.ty(t);
            }
        }
        Stmt::Var {
            name,
            type_annotation,
            initializer,
        } => {
            v.token(name);
            if let Some(t) = type_annotation {
                v.ty(t);
            }
            if let Some(e) = initializer {
                expr(e, v);
            }
        }
        Stmt::ArrayDestructuring {
            keyword,
            bindings,
            initializer,
        } => {
            v.token(keyword);
            for b in bindings {
                v.token(b);
            }
            expr(initializer, v);
        }
        Stmt::If {
            condition,
            then_branch,
            else_branch,
            if_token,
        } => {
            expr(condition, v);
            stmt(then_branch, v);
            if let Some(s) = else_branch {
                stmt(s, v);
            }
            v.token(if_token);
        }
        Stmt::While {
            condition,
            body,
            while_token,
        } => {
            expr(condition, v);
            stmt(body, v);
            v.token(while_token);
        }
        Stmt::For {
            keyword,
            initializer,
            condition,
            increment,
            body,
        } => {
            v.token(keyword);
            if let Some(s) = initializer {
                stmt(s, v);
            }
            if let Some(e) = condition {
                expr(e, v);
            }
            if let Some(e) = increment {
                expr(e, v);
            }
            statements(body, v);
        }
        Stmt::ForIn {
            keyword,
            key,
            value,
            iterable,
            body,
        } => {
            v.token(keyword);
            v.token(key);
            if let Some(t) = value {
                v.token(t);
            }
            expr(iterable, v);
            statements(body, v);
        }
        Stmt::Return { keyword, value } => {
            v.token(keyword);
            if let Some(e) = value {
                expr(e, v);
            }
        }
        Stmt::Throw { keyword, thrown } => {
            v.token(keyword);
            expr(thrown, v);
        }
        Stmt::TryCatch {
            try_body,
            catch_body,
            exception,
            exception_type,
        } => {
            statements(try_body, v);
            statements(catch_body, v);
            v.token(exception);
            if let Some(t) = exception_type {
                v.ty(t);
            }
        }
        Stmt::Function {
            name,
            type_params,
            params,
            return_type,
            body,
            ..
        } => {
            v.token(name);
            generics(type_params, v);
            parameters(params, v);
            if let Some(t) = return_type {
                v.ty(t);
            }
            if let Some(b) = body {
                statements(b, v);
            }
        }
        Stmt::Struct {
            name,
            type_params,
            fields,
        } => {
            v.token(name);
            generics(type_params, v);
            for (n, t, _) in fields {
                v.token(n);
                v.ty(t);
            }
        }
        Stmt::Trait {
            name,
            type_params,
            traits,
            methods,
        } => {
            v.token(name);
            generics(type_params, v);
            for e in traits {
                expr(e, v);
            }
            statements(methods, v);
        }
        Stmt::Impl {
            keyword,
            type_params,
            trait_name,
            target_type,
            methods,
        } => {
            v.token(keyword);
            generics(type_params, v);
            if let Some(t) = trait_name {
                v.ty(t);
            }
            v.ty(target_type);
            statements(methods, v);
        }
        Stmt::Enum {
            name,
            type_params,
            cases,
        } => {
            v.token(name);
            generics(type_params, v);
            for c in cases {
                v.token(&mut c.name);
                for (n, t) in &mut c.parameters {
                    v.token(n);
                    if let Some(t) = t {
                        v.ty(t);
                    }
                }
            }
        }
        Stmt::Using {
            keyword,
            names,
            source,
        } => {
            v.token(keyword);
            for n in names {
                v.token(n);
            }
            expr(source, v);
        }
    }
}

pub fn fresh_instance(s: &mut Stmt) {
    struct Fresh;
    impl Visitor for Fresh {
        fn token(&mut self, t: &mut Token) {
            t.id = Token::fresh_id();
        }
    }
    stmt(s, &mut Fresh);
}

impl Expr {
    pub fn untyped(&self) -> &Expr {
        if let Expr::Typed { expr, .. } = self {
            expr.untyped()
        } else {
            self
        }
    }
    pub fn checked_type(&self) -> Option<&Type> {
        if let Expr::Typed { ty, .. } = self {
            Some(ty)
        } else {
            None
        }
    }
}

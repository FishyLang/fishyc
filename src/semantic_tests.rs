use super::TypeChecker;
use crate::ast::{Expr, Stmt, Type};
use crate::ast_visit::{self, Visitor};
use crate::module_loader::ModuleLoader;
use crate::parser::Parser;
use crate::scanner::Scanner;
use std::path::Path;

fn checked(source: &str) -> (Vec<Stmt>, TypeChecker) {
    let mut scanner = Scanner::new(source, "semantic-test.fih");
    scanner.scan_tokens();
    assert!(scanner.errors.is_empty(), "{:?}", scanner.errors);
    let mut parser = Parser::new(scanner.tokens);
    let ast = parser.parse();
    assert!(parser.errors.is_empty(), "{:?}", parser.errors);
    let mut loader = ModuleLoader::new();
    let mut ast = loader.resolve_root(ast, Path::new("."), vec![]);
    assert!(loader.parse_errors.is_empty(), "{:?}", loader.parse_errors);
    let mut checker = TypeChecker::new();
    checker.module_symbols = loader.symbols;
    checker.check(&mut ast);
    assert!(checker.errors.is_empty(), "{:?}", checker.errors);
    (ast, checker)
}

#[test]
fn grouping_materializes_context_on_inner_expressions() {
    let (ast, _) = checked(
        "fn make<T>(x:T=7)->T{return x;}fn main(){let a:[u8;2]=([1,2]);let n:u8=(make());}",
    );
    let Stmt::Function {
        body: Some(body), ..
    } = &ast[1]
    else {
        panic!("main");
    };
    let expected = [Type::Array(2, Box::new(Type::U8)), Type::U8];
    for (stmt, ty) in body.iter().zip(&expected) {
        let Stmt::Var {
            initializer: Some(expr),
            ..
        } = stmt
        else {
            panic!("variable");
        };
        assert_eq!(expr.checked_type(), Some(ty));
        let Expr::Grouping(inner) = expr.untyped() else {
            panic!("grouping");
        };
        assert_eq!(inner.checked_type(), Some(ty));
    }
}

#[test]
fn nested_generic_context_keeps_the_concrete_field_type() {
    let (_, checker) = checked(
        "struct Box<T>{pub v:T,}struct S{pub b:Box<Box<[u8;2]>>,}\
         fn main(){let s=S{b:Box{v:Box{v:[1,255]}}};}",
    );
    let outer = &checker.user_types["S"].fields["b"].1;
    let Type::Custom(outer) = outer else {
        panic!("outer box");
    };
    let Type::Custom(inner) = &checker.user_types[outer].fields["v"].1 else {
        panic!("inner box");
    };
    assert_eq!(
        checker.user_types[inner].fields["v"].1,
        Type::Array(2, Box::new(Type::U8))
    );
}

#[test]
fn specializations_have_distinct_names_and_concrete_signatures() {
    let (_, checker) = checked(
        "fn id<T>(x:T)->T{return x;}fn wrap<T>(x:T)->(T,){return (id(x),);}\
         fn main(){let a=wrap(7 as u8);let b=wrap(1.5 as f32);}",
    );
    let functions = checker
        .instantiations
        .iter()
        .filter_map(|stmt| {
            if let Stmt::Function {
                name,
                params,
                return_type,
                ..
            } = stmt
            {
                Some((
                    name.lexeme.clone(),
                    params[0].type_annotation.clone().unwrap(),
                    return_type.clone().unwrap(),
                ))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(functions.len(), 4);
    let names = functions
        .iter()
        .map(|(n, _, _)| n)
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(names.len(), 4);
    for ty in [Type::U8, Type::F32] {
        assert!(functions.iter().any(|(_, p, r)| p == &ty && r == &ty));
        assert!(functions
            .iter()
            .any(|(_, p, r)| p == &ty && r == &Type::Tuple(vec![ty.clone()])));
    }
}

#[test]
fn list_remainders_materialize_array_and_slice_binding_types() {
    let (mut ast, _) = checked(
        "fn main(){let a:[u8;2]=[1,2];let s:[u8;?]=[1,2];\
         match a {[x,...rest]->rest;*->[0 as u8];};\
         match s {[x,...rest]->rest;*->s;};}",
    );
    struct Remainders(Vec<Type>);
    impl Visitor for Remainders {
        fn expr(&mut self, expr: &mut Expr) {
            if let Expr::ListPattern {
                rest: Some(rest), ..
            } = expr
            {
                self.0
                    .push(rest.checked_type().expect("typed remainder").clone());
            }
        }
    }
    let mut remainders = Remainders(vec![]);
    ast_visit::statements(&mut ast, &mut remainders);
    assert_eq!(
        remainders.0,
        vec![
            Type::Array(1, Box::new(Type::U8)),
            Type::Slice(Box::new(Type::U8))
        ]
    );
}

#[test]
fn defaults_keep_full_parameter_signature_and_one_omitted_expression() {
    let (ast, checker) = checked("fn pick(a:u8,b:u8=a)->u8{return b;}fn main(){let n:u8=pick(1);}");
    let Stmt::Function { params, .. } = &ast[0] else {
        panic!("function");
    };
    assert_eq!(params.len(), 2);
    assert_eq!(checker.default_calls.len(), 1);
    let plan = checker.default_calls.values().next().unwrap();
    assert_eq!(plan.parameters.len(), 2);
    assert!(plan.parameters.iter().all(|(_, ty)| ty == &Type::U8));
    assert_eq!(plan.defaults.len(), 1);
}

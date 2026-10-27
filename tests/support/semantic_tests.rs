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

#[test]
fn recursive_generic_annotations_use_the_registered_nominal_identity() {
    let (ast, mut checker) = checked(
        "enum Link<T>{End,Next(T),}\
         struct Node<T>{pub value:[T;1],pub next:Link<Node<T>>,}\
         fn main(){let end:Link<Node<u8>>=Link.End();\
         let n=Node<u8>{value:[1],next:end};}",
    );
    let node = checker.resolve_type(&Type::Generic("Node".into(), vec![Type::U8]));
    let link = checker.resolve_type(&Type::Generic("Link".into(), vec![node.clone()]));
    let Stmt::Function {
        body: Some(body), ..
    } = &ast[2]
    else {
        panic!("main");
    };
    let Stmt::Var {
        type_annotation,
        initializer: Some(end),
        ..
    } = &body[0]
    else {
        panic!("end");
    };
    assert_eq!(end.checked_type(), Some(&link));
    assert_eq!(
        type_annotation.as_ref(),
        Some(&link),
        "binding and constructor must share one concrete identity"
    );
    let Type::Custom(node_name) = &node else {
        panic!("concrete node");
    };
    let Type::Custom(link_name) = &link else {
        panic!("concrete link");
    };
    assert_eq!(
        checker.user_types[node_name].fields["value"].1,
        Type::Array(1, Box::new(Type::U8))
    );
    assert_eq!(checker.user_types[node_name].fields["next"].1, link);
    assert_eq!(
        checker.user_types[link_name].static_methods["Next"].0,
        vec![node.clone()]
    );
    for name in [node_name, link_name] {
        assert_eq!(checker.instantiations.iter().filter(|stmt| {
            matches!(stmt, Stmt::Struct { name: token, .. } | Stmt::Enum { name: token, .. } if &token.lexeme == name)
        }).count(), 1, "each concrete recursive identity is declared once");
    }
}

#[test]
fn nested_generic_materialization_is_canonical_and_idempotent() {
    let (_, mut checker) = checked(
        "enum Link<T>{End,Next(T),}struct Node<T>{pub value:T,pub next:Link<Node<T>>,}\
         struct Leaf{pub value:u8,}\
         fn main(){let a:Link<Node<u8>>=Link.End();let b:Link<Node<Leaf>>=Link.End();}",
    );
    let mut identities = Vec::new();
    for payload in [Type::U8, Type::Custom("Leaf".into())] {
        let source_type = Type::Generic(
            "Link".into(),
            vec![Type::Generic("Node".into(), vec![payload])],
        );
        let concrete = checker.resolve_type(&source_type);
        let instantiations = checker.instantiations.len();
        assert_eq!(checker.final_type(source_type.clone()), concrete);
        assert_eq!(checker.final_type(concrete.clone()), concrete);
        assert_eq!(checker.resolve_type(&source_type), concrete);
        assert_eq!(
            checker.instantiations.len(),
            instantiations,
            "materialization must reuse the specialization"
        );
        identities.push(concrete);
    }
    assert_ne!(
        identities[0], identities[1],
        "different payloads keep different identities"
    );
}

#[test]
fn closure_capture_plans_resolve_free_binding_identities() {
    let (_, checker) = checked(
        "fn main(){let x:i64=1;let p=&x;let unused:i64=99;\
         let a=fn()->i64{return x;};\
         {let x:u8=2;let b=fn()->u8{return x;};}\
         let c=fn(x:i32)->i32{return x;};\
         let d=fn()->i64{let x:i64=3;return x;};\
         let e=fn()->i64{x=4;return x;};}",
    );
    let mut plans = checker.capture_plans.iter().collect::<Vec<_>>();
    plans.sort_by_key(|(id, _)| **id);
    assert_eq!(plans.iter().map(|(_, p)|p.len()).collect::<Vec<_>>(), vec![1,1,0,0,1]);
    let first = &plans[0].1[0];
    let shadow = &plans[1].1[0];
    assert_eq!(first.declaration.lexeme, "x");
    assert_eq!(first.ty, Type::I64);
    assert_eq!(shadow.ty, Type::U8);
    assert_ne!(first.declaration.id, shadow.declaration.id);
    assert_eq!(first.declaration.id, plans[4].1[0].declaration.id);
    assert!(checker.resolved_bindings.values().any(|id|*id==first.declaration.id));
}

#[test]
fn nested_closures_capture_transitively_but_not_inner_locals() {
    let (_, checker) = checked(
        "fn main(){let outer:i64=7;let unused:i64=0;\
         let a=fn(param:i64)->fn()->i64{let local:i64=2;\
         return fn()->i64{return outer+param+local;};};}",
    );
    let mut plans=checker.capture_plans.iter().collect::<Vec<_>>();
    plans.sort_by_key(|(id,_)|**id);
    assert_eq!(plans[0].1.iter().map(|c|c.declaration.lexeme.as_str()).collect::<Vec<_>>(),vec!["outer"]);
    assert_eq!(plans[1].1.iter().map(|c|c.declaration.lexeme.as_str()).collect::<Vec<_>>(),vec!["outer","param","local"]);
    assert_eq!(plans[0].1[0].declaration.id,plans[1].1[0].declaration.id);
}

#[test]
fn specialized_capture_plans_have_fresh_declarations_and_concrete_types() {
    let (_, checker)=checked(
        "fn make<T>(x:T)->fn()->T{return fn()->T{return x;};}\
         fn main(){let a=make(7 as u8);let b=make(9 as i64);}",
    );
    let captures=checker.capture_plans.values().flat_map(|p|p).collect::<Vec<_>>();
    assert_eq!(captures.len(),2);
    assert_ne!(captures[0].declaration.id,captures[1].declaration.id);
    assert!(captures.iter().any(|c|c.ty==Type::U8));
    assert!(captures.iter().any(|c|c.ty==Type::I64));
}

#[test]
fn default_closures_capture_formal_values_not_caller_bindings() {
    let (_, checker)=checked(
        "fn use(x:i64,f:fn()->i64=fn()->i64{return x;})->i64{return f();}\
         fn main(){let x:u8=99;let f=fn()->i64{return use(7);};}",
    );
    // The caller's closure references only a named function. Both the checked
    // declaration default and the elaborated default capture use's formal x.
    assert_eq!(checker.capture_plans.values().filter(|p|p.is_empty()).count(),1);
    let captures=checker.capture_plans.values().flat_map(|p|p).collect::<Vec<_>>();
    assert_eq!(captures.len(),2);
    assert!(captures.iter().all(|c|c.declaration.lexeme=="x" && c.ty==Type::I64));
    assert_eq!(captures[0].declaration.id,captures[1].declaration.id);
}


#[test]
fn retired_names_have_no_intrinsic_type_or_library_meaning() {
    // Ordinary user definitions may reuse names; the compiler supplies no runtime type.
    checked("struct Task<T>{pub value:T,}struct Mutex{pub n:i64,}fn main(){let p=Task{value:3};let q=Mutex{n:p.value};}");
    for source in ["fn main(){let missing:Task<i64>;}","fn main(){let missing:Awaitable<i64>;}","fn main(){Http.get();Json.object();}"] {
        let mut scanner=Scanner::new(source,"retirement.fih");scanner.scan_tokens();
        let mut parser=Parser::new(scanner.tokens);let mut ast=parser.parse();assert!(parser.errors.is_empty());
        let mut checker=TypeChecker::new();checker.check(&mut ast);assert!(!checker.errors.is_empty(),"{source}");
    }
}

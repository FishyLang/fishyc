use super::*;
use crate::scanner::Scanner;

fn parser(source: &str) -> Parser {
    let mut scanner = Scanner::new(source, "parser-test.fih");
    scanner.scan_tokens();
    assert!(scanner.errors.is_empty(), "{:?}", scanner.errors);
    Parser::new(scanner.tokens)
}

fn program(source: &str) -> Vec<Stmt> {
    let mut p = parser(source);
    let ast = p.parse();
    assert!(p.errors.is_empty(), "{source}\n{:?}", p.errors);
    ast
}

fn expression(source: &str) -> Expr {
    let mut ast = program(&format!("let result = {source};"));
    match ast.remove(0) {
        Stmt::Var {
            initializer: Some(expr),
            ..
        } => expr,
        other => panic!("expected initializer: {other:?}"),
    }
}

fn shape(expr: &Expr) -> String {
    match expr {
        Expr::Variable(t) => t.lexeme.clone(),
        Expr::Literal(l) => format!("{l:?}"),
        Expr::Grouping(e) => format!("(group {})", shape(e)),
        Expr::Binary {
            left,
            operator,
            right,
        }
        | Expr::Logical {
            left,
            operator,
            right,
        } => format!("({} {} {})", operator.lexeme, shape(left), shape(right)),
        Expr::Unary { operator, right } => format!("({} {})", operator.lexeme, shape(right)),
        Expr::AddressOf { operand, .. } => format!("(& {})", shape(operand)),
        Expr::Dereference { operand, .. } => format!("(^ {})", shape(operand)),
        Expr::Assign { name, value } => format!("(= {} {})", name.lexeme, shape(value)),
        Expr::Get { object, name } => format!("(. {} {})", shape(object), name.lexeme),
        Expr::SubscriptGet { indexee, index, .. } => {
            format!("([] {} {})", shape(indexee), shape(index))
        }
        Expr::Call {
            callee, arguments, ..
        } => format!(
            "(call {} [{}])",
            shape(callee),
            arguments.iter().map(shape).collect::<Vec<_>>().join(", ")
        ),
        Expr::Cast {
            value, target_type, ..
        } => format!("(as {} {})", shape(value), target_type),
        other => format!("{other:?}"),
    }
}

#[test]
fn arithmetic_precedence_and_associativity() {
    for (source, expected) in [
        ("a + b * c", "(+ a (* b c))"),
        ("a * b + c", "(+ (* a b) c)"),
        ("a - b - c", "(- (- a b) c)"),
        ("a / b * c", "(* (/ a b) c)"),
        ("(a + b) * c", "(* (group (+ a b)) c)"),
    ] {
        assert_eq!(shape(&expression(source)), expected, "{source}");
    }
}

#[test]
fn comparison_logical_and_assignment_shapes() {
    for (source, expected) in [
        ("a == b && c == d", "(&& (== a b) (== c d))"),
        ("a || b && c", "(|| a (&& b c))"),
        ("a < b == c", "(== (< a b) c)"),
        ("a = b = c", "(= a (= b c))"),
    ] {
        assert_eq!(shape(&expression(source)), expected, "{source}");
    }
}

#[test]
fn casts_and_prefix_operators_preserve_precedence() {
    for (source, expected) in [
        ("a + b as i32", "(+ a (as b i32))"),
        ("-a as i32 as i64", "(as (as (- a) i32) i64)"),
        ("^a + b", "(+ (^ a) b)"),
        ("&foo.bar[index]", "(& ([] (. foo bar) index))"),
        ("^ptr.field", "(^ (. ptr field))"),
        ("(a as i64).field", "(. (group (as a i64)) field)"),
    ] {
        assert_eq!(shape(&expression(source)), expected, "{source}");
    }
}

#[test]
fn postfix_chains_use_one_left_to_right_model() {
    for (source, expected) in [
        ("foo().bar[index]", "([] (. (call foo []) bar) index)"),
        ("foo.bar().baz", "(. (call (. foo bar) []) baz)"),
        (
            "foo.bar[index].baz()",
            "(call (. ([] (. foo bar) index) baz) [])",
        ),
        ("foo()[index](arg)", "(call ([] (call foo []) index) [arg])"),
    ] {
        assert_eq!(shape(&expression(source)), expected, "{source}");
    }
}

#[test]
fn recursive_types_and_nested_generic_closers() {
    let box_i64 = Type::Generic("Box".into(), vec![Type::I64]);
    let array = Type::Array(2, Box::new(box_i64));
    let pointer = Type::Pointer(Box::new(Type::MutReference(Box::new(array))));
    for (source, expected) in [
        ("^const &mut [Box<i64>; 0x2]", pointer),
        (
            "Box<Box<Box<i64>>>",
            Type::Generic(
                "Box".into(),
                vec![Type::Generic(
                    "Box".into(),
                    vec![Type::Generic("Box".into(), vec![Type::I64])],
                )],
            ),
        ),
        (
            "fn(^u8, ...) -> i32",
            Type::Function(
                vec![Type::Pointer(Box::new(Type::U8))],
                Box::new(Type::I32),
                true,
            ),
        ),
        (
            "(i64, &bool) | [u8; ?]",
            Type::Union(vec![
                Type::Tuple(vec![Type::I64, Type::Reference(Box::new(Type::Bool))]),
                Type::Slice(Box::new(Type::U8)),
            ]),
        ),
    ] {
        let mut p = parser(source);
        assert_eq!(p.parse_type().unwrap(), expected, "{source}");
        assert!(p.is_at_end(), "unconsumed type tokens: {source}");
    }
}

#[test]
fn generic_declarations_constraints_and_initializers() {
    let ast=program("fn id<T: First + Second<Box<T>>>(x: &T) -> &T { return x; } struct Box<T> { value:T, } let x=Box<Box<i64>> {value:Box<i64>{value:1}};");
    match &ast[0] {
        Stmt::Function {
            type_params,
            params,
            ..
        } => {
            assert_eq!(type_params[0].constraints.len(), 2);
            assert_eq!(
                params[0].type_annotation,
                Some(Type::Reference(Box::new(Type::Custom("T".into()))))
            );
        }
        other => panic!("{other:?}"),
    }
    assert!(
        matches!(&ast[2],Stmt::Var {initializer:Some(Expr::StructInit {type_args,..}),..} if type_args.len()==1)
    );
}

#[test]
fn declarations_control_flow_and_imports() {
    let ast=program("using {Thing, other} from \"module.fih\"; alias Bytes=[u8;?]; extern fn c(s:^u8,...) -> i32; trait Read with Base { fn read(self)->i64; } impl Read for Thing { pub fn read(self)->i64 {return 1;} } enum E { A(T), B(value:^i64), None, } fn main()->i64 {let xs=[1,2];let [a,b]=xs;for x in xs {if x>0 {return x;}}for(let i=0;i<2;i=i+1){}while false {}return 0;}");
    assert_eq!(ast.len(), 7);
}

#[test]
fn shorthand_initializers_and_arrow_bodies() {
    let ast=program("fn f()->i64 -> 1; let x=S{value,nested:S{value:2}}; let g=fn(x:i64)->i64 -> x; let m=match 1 {1 if true -> 2; * -> {return 3;}};");
    assert!(
        matches!(&ast[0],Stmt::Function {body:Some(b),..} if matches!(&b[0],Stmt::Return {..}))
    );
    assert!(
        matches!(&ast[3],Stmt::Var {initializer:Some(Expr::Match {cases,..}),..} if matches!(&cases[0].body[0],Stmt::Expression(_)))
    );
}

#[test]
fn initializer_context_is_scoped_to_undelimited_headers() {
    for source in [
        "if (S{x:1}.x==1) {}",
        "if check(S{x:1}) {}",
        "if [S{x:1}][0].x==1 {}",
        "while check((S{x:1})) {}",
        "for x in items(S{x:1}) {}",
        "if check(fn()->bool {let s=S{x:1};return true;}) {}",
        "if match 1 { * -> S{x:1}; } {}",
        "match match 1 { * -> S{x:1}; } { * -> 2; };",
    ] {
        program(source);
    }
    // The unavoidable outer ambiguity retains its existing rule: this is a
    // variable condition followed by a block, not an empty struct initializer.
    let ast = program("if flag {} while flag {} for x in items {} match value {* -> 1;};");
    assert!(matches!(
        &ast[0],
        Stmt::If {
            condition: Expr::Variable(_),
            ..
        }
    ));
}

#[test]
fn failed_headers_restore_expression_context() {
    for source in [
        "for x in ; let recovered=S{x:1};",
        "match ; let recovered=S{x:1};",
        "if ; let recovered=S{x:1};",
        "while ; let recovered=S{x:1};",
    ] {
        let mut p = parser(source);
        let ast = p.parse();
        assert!(!p.errors.is_empty());
        assert_eq!(p.expression_context, ExpressionContext::Normal);
        assert!(ast.iter().any(|s|matches!(s,Stmt::Var {name,initializer:Some(Expr::StructInit {..}),..} if name.lexeme=="recovered")),"{source}: {ast:?}");
    }
}

#[test]
fn generic_initializer_probe_respects_grammar_boundaries() {
    program("let small=a<b; if c>d {} let next=7;");
    for source in [
        "let s=Holder<fn()->i64>{value:f};",
        "let s=Holder<(i64,bool)>{value:x};",
        "let s=Holder<fn(^Box<i64>,...)->(i64,bool)>{value:f};",
        "let s=Box<^&mut [Box<i64>;2]>{value:x};",
    ] {
        assert!(matches!(
            &program(source)[0],
            Stmt::Var {
                initializer: Some(Expr::StructInit { .. }),
                ..
            }
        ));
    }
    assert_eq!(shape(&expression("a < b > c")), "(> (< a b) c)");
    assert_eq!(shape(&expression("a < T >= b")), "(>= (< a T) b)");
    // Explicit generic function calls are not in the Call AST/grammar today.
    assert_eq!(shape(&expression("f<T>(x)")), "(> (< f T) (group x))");
}

#[test]
fn generic_closer_fission_preserves_assignment_and_rollback() {
    for source in [
        "let x:Box<i64>=value;",
        "let x:Box<Box<i64>>=value;",
        "let x:Box<Box<Box<i64>>>=value;",
        "let x:^Box<fn()->Box<i64>>=value;",
    ] {
        let ast = program(source);
        assert!(
            matches!(&ast[0],Stmt::Var {initializer:Some(Expr::Variable(t)),..} if t.lexeme=="value")
        );
    }
    let mut p = parser("<T>=value;");
    let before = p.tokens.clone();
    assert!(p.generic_initializer_arguments().is_none());
    assert_eq!(p.current, 0);
    assert_eq!(p.tokens, before);
    assert!(p.type_token_edits.is_empty());
    assert!(p.errors.is_empty());
}

#[test]
fn syntax_errors_stay_inside_the_owning_block() {
    let mut p = parser("fn f(){let bad=;let good=7;let bad2=;return;} fn next()->i64{return 9;}");
    let ast = p.parse();
    assert_eq!(p.errors.len(), 2);
    assert_eq!(ast.len(), 2, "{ast:?}");
    match &ast[0] {
        Stmt::Function {
            body: Some(body), ..
        } => {
            assert_eq!(body.len(), 2);
            assert!(matches!(&body[0],Stmt::Var {name,..} if name.lexeme=="good"));
            assert!(matches!(&body[1], Stmt::Return { value: None, .. }));
        }
        other => panic!("{other:?}"),
    }
    assert!(matches!(&ast[1],Stmt::Function {name,..} if name.lexeme=="next"));
}

#[test]
fn recovery_recognizes_all_declaration_starters() {
    let mut p=parser("let bad=;alias A=i64;let bad=;extern fn f(i64);let bad=;using X from \"x.fih\";let bad=;try {} catch e {}let bad=;throw 1;");
    let ast = p.parse();
    assert_eq!(p.errors.len(), 5);
    assert_eq!(ast.len(), 5);
    assert!(matches!(&ast[0], Stmt::Alias { .. }));
    assert!(matches!(&ast[1], Stmt::ExternFunction { .. }));
    assert!(matches!(&ast[2], Stmt::Using { .. }));
    assert!(matches!(&ast[3], Stmt::TryCatch { .. }));
    assert!(matches!(&ast[4], Stmt::Throw { .. }));
}

#[test]
fn eof_and_invalid_delimiters_do_not_panic() {
    assert!(Parser::new(vec![]).parse().is_empty());
    let mut p = parser("let x=1;");
    p.tokens.pop();
    let mut p = Parser::new(p.tokens);
    assert_eq!(p.parse().len(), 1);
    assert!(p.errors.is_empty());
    let mut p = Parser::new(vec![]);
    assert_eq!(p.advance().token_type, TokenType::Eof);
    assert_eq!(p.previous().token_type, TokenType::Eof);
    for source in [
        "let x=(;",
        "let x=a[;",
        "let x=S{field:};",
        "fn f(",
        "fn f(){",
        "alias X=Box<;",
        "let x=1 as;",
        ") ] } let valid=7;",
        "if {",
        "for x in",
        "match 1{* ->",
        "let x=Box<fn(> {} ;",
    ] {
        let mut p = parser(source);
        p.parse();
        assert!(!p.errors.is_empty(), "{source}");
        assert!(p.is_at_end());
    }
}

#[test]
fn every_truncated_prefix_of_a_compound_program_terminates() {
    let source = "fn f<T>(x:&Box<T>)->i64 {if check(Box<T>{value:x}) {return 1;}return 0;}";
    for end in 0..=source.len() {
        let mut scanner = Scanner::new(&source[..end], "prefix.fih");
        scanner.scan_tokens();
        let mut p = Parser::new(scanner.tokens);
        p.parse();
        assert!(p.is_at_end());
    }
}

#[test]
fn arrow_and_compound_sugar_keep_real_source_locations() {
    let ast = program("fn f()->i64 -> 7; let g=fn()->i64 -> 8; let x=1; x+=2;");
    let Stmt::Function {
        body: Some(body), ..
    } = &ast[0]
    else {
        panic!()
    };
    let Stmt::Return { keyword, .. } = &body[0] else {
        panic!()
    };
    assert_eq!(keyword.file_path.as_str(), "parser-test.fih");
    assert_eq!(keyword.line, 1);
    let Stmt::Expression(Expr::Assign { value, .. }) = &ast[3] else {
        panic!()
    };
    let Expr::Binary { operator, .. } = value.as_ref() else {
        panic!()
    };
    assert_eq!(operator.token_type, TokenType::Plus);
    assert_eq!(operator.file_path.as_str(), "parser-test.fih");
    assert_eq!(operator.line, 1);
}

#[test]
fn malformed_generic_lists_and_initializers_fail_gracefully() {
    for source in [
        "alias X=Box<i64,,bool>;",
        "alias X=Box<(i64,bool>;",
        "alias X=Box<fn(i64)->>;",
        "let x=Box<,>{value:1};",
        "let x=Box<i64>{field 1};",
        "let x=S{field:1 field:2};",
        "let x=Box<i64>{field:};",
    ] {
        let mut p = parser(source);
        p.parse();
        assert!(!p.errors.is_empty(), "{source}");
    }
}

#[test]
fn positional_enum_payloads_use_the_full_type_grammar() {
    let ast=program("enum E<T>{Value(T), Byte(u8), Pointer(^T), Array([T;2]), Nested(Box<Box<T>>), Callback(fn(^T)->bool), Named(value:&mut T),}");
    let Stmt::Enum { cases, .. } = &ast[0] else {
        panic!()
    };
    assert_eq!(cases[0].parameters[0].1, Some(Type::Custom("T".into())));
    assert_eq!(cases[1].parameters[0].1, Some(Type::U8));
    assert_eq!(
        cases[2].parameters[0].1,
        Some(Type::Pointer(Box::new(Type::Custom("T".into()))))
    );
    assert!(matches!(&cases[3].parameters[0].1, Some(Type::Array(2, _))));
    assert!(matches!(
        &cases[5].parameters[0].1,
        Some(Type::Function(..))
    ));
    assert_eq!(cases[6].parameters[0].0.lexeme, "value");
    assert!(matches!(
        &cases[6].parameters[0].1,
        Some(Type::MutReference(_))
    ));
}

#[test]
fn numeric_members_require_integer_indices() {
    assert!(
        matches!(expression("value.0"),Expr::Get {name,..} if matches!(name.literal,Literal::Integer(0)))
    );
    let mut p = parser("let x=value.1.5;");
    p.parse();
    assert!(p
        .errors
        .iter()
        .any(|e| e.message == "Tuple field index must be an integer."));
}

#[test]
fn recovered_errors_explain_expected_and_encountered_input() {
    let mut p = parser("let x=call(1; let y=2 as;");
    p.parse();
    assert!(p
        .errors
        .iter()
        .any(|e| e.message.contains("Expected ')'") && e.message.contains("Found `;`")));
    assert!(p
        .errors
        .iter()
        .any(|e| e.message.contains("type annotation") && e.message.contains("Found `;`")));
    let mut p = parser("fn f(");
    p.parse();
    assert!(p.errors.iter().any(|e| e.message.contains("end of input")));
}

#[test]
fn malformed_token_combinations_make_progress() {
    let choices = [
        "fn", "let", "if", "while", "for", "in", "match", "return", "alias", "extern", "using",
        "try", "catch", "x", "1", "(", ")", "[", "]", "{", "}", ":", ";", ",", "<", ">", ">=", "=",
        "as", "^", "&", "->",
    ];
    let mut seed = 0x51a7_u64;
    for _ in 0..2000 {
        let mut source = String::new();
        for _ in 0..48 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            source.push_str(choices[(seed >> 32) as usize % choices.len()]);
            source.push(' ');
        }
        let mut p = parser(&source);
        p.parse();
        assert!(p.is_at_end(), "{source}");
    }
}

#[test]
fn unchanged_reserved_operator_grammar_is_explicit() {
    assert_eq!(shape(&expression("a ** b ** c")), "(** (** a b) c)");
    assert_eq!(shape(&expression("a .. b + c")), "(.. a (+ b c))");
    assert_eq!(shape(&expression("*a + b")), "(+ (* a) b)");
    // These operators lack complete semantic/backend support. Do not silently
    // change their relative precedence while repairing supported constructs.
    let Expr::Ternary {
        condition,
        else_branch,
        ..
    } = expression("a = b ? c : d ? e : f")
    else {
        panic!()
    };
    assert!(matches!(*condition, Expr::Assign { .. }));
    assert!(matches!(*else_branch, Expr::Ternary { .. }));
    assert_eq!(shape(&expression("a = b ?? c")), "(?? (= a b) c)");
}

#[test]
fn committed_type_fission_does_not_change_later_comparisons() {
    let ast = program("let x:Box<i64>=value;let y=a<T>=b;let z:Box<>=value;");
    let Stmt::Var {
        initializer: Some(expr),
        ..
    } = &ast[1]
    else {
        panic!()
    };
    assert_eq!(shape(expr), "(>= (< a T) b)");
    assert!(
        matches!(&ast[2],Stmt::Var {type_annotation:Some(Type::Generic(_,args)),initializer:Some(_),..} if args.is_empty())
    );
}

#[test]
fn adjacent_reference_prefixes_are_types_without_changing_logical_and() {
    for source in ["&&T", "& &T", "&&&T", "&&&&mut T"] {
        let mut p = parser(source);
        let ty = p.parse_type().unwrap();
        assert!(p.is_at_end());
        fn depth(ty: &Type) -> usize {
            match ty {
                Type::Reference(t) | Type::MutReference(t) => 1 + depth(t),
                _ => 0,
            }
        }
        assert_eq!(depth(&ty), source.chars().filter(|c| *c == '&').count());
    }
    let mut p = parser("<&&T>=value;");
    let original = p.tokens.clone();
    assert!(p.generic_initializer_arguments().is_none());
    assert_eq!(p.tokens, original);
    assert_eq!(p.current, 0);
    assert_eq!(shape(&expression("a && b")), "(&& a b)");
    assert_eq!(
        shape(&expression("a < b && c >= d")),
        "(&& (< a b) (>= c d))"
    );
    assert!(
        matches!(&program("let s=Box<&&T>{value:x};")[0],Stmt::Var {initializer:Some(Expr::StructInit {type_args,..}),..} if matches!(&type_args[0],Type::Reference(_)))
    );
}

#[test]
fn initializer_braces_do_not_close_the_enclosing_function_during_recovery() {
    for bad in [
        "let bad=S{field:};",
        "let bad=S{field:Nested{value:}};",
        "let bad=call(S{field:});",
    ] {
        let source = format!("fn f(){{{bad}let good=7;return;}} fn next()->i64{{return 9;}}");
        let mut p = parser(&source);
        let ast = p.parse();
        assert!(!p.errors.is_empty());
        assert_eq!(ast.len(), 2, "{source}: {ast:?}");
        let Stmt::Function {
            body: Some(body), ..
        } = &ast[0]
        else {
            panic!()
        };
        assert!(
            body.iter()
                .any(|s| matches!(s,Stmt::Var {name,..} if name.lexeme=="good")),
            "{source}: {ast:?}"
        );
        assert!(matches!(&ast[1],Stmt::Function {name,..} if name.lexeme=="next"));
    }
}

#[test]
fn comparisons_after_casts_do_not_start_generic_types() {
    for (source, expected) in [
        ("x as i64 < limit", "(< (as x i64) limit)"),
        ("x as Int < limit", "(< (as x Int) limit)"),
        ("x as i64 < y > z", "(> (< (as x i64) y) z)"),
        ("x as ^Int < limit", "(< (as x ^Int) limit)"),
        ("x as Box<i64> < limit", "(< (as x Box<i64>) limit)"),
        ("x as fn()->i64 < limit", "(< (as x fn() -> i64) limit)"),
    ] {
        assert_eq!(shape(&expression(source)), expected, "{source}");
    }
}

#[test]
fn cast_type_disambiguation_does_not_relax_required_type_lists() {
    for source in ["Box<i64", "Box<fn()->Int", "^Box<Int", "[Box<Int;2]"] {
        let mut p = parser(source);
        assert!(p.parse_type().is_err(), "{source}");
    }
    let mut p = parser("Box<Int;");
    assert!(p.parse_type().unwrap_err().message.contains("Expected '>'"));
    let Expr::Cast { target_type, .. } = expression("value as Box<fn(^T)->T>") else {
        panic!()
    };
    assert!(matches!(target_type,Type::Generic(_,args) if matches!(&args[0],Type::Function(..))));
}

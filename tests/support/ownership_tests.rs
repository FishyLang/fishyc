//! Current acyclic ARC invariants at the checked source/custom-IR boundary.
use super::IrBuilder;
use crate::ast::{Stmt, Type};
use crate::ast_visit::{self};
use crate::ir::*;
use crate::ir_optimizer::IrOptimizer;
use crate::{
    ir_verify, module_loader::ModuleLoader, parser::Parser, scanner::Scanner,
    type_checker::TypeChecker,
};
use std::path::Path;

fn parse(source: &str) -> Vec<Stmt> {
    let mut scanner = Scanner::new(source, "ir-contract.fih");
    scanner.scan_tokens();
    assert!(scanner.errors.is_empty(), "{:?}", scanner.errors);
    let mut parser = Parser::new(scanner.tokens);
    let ast = parser.parse();
    assert!(parser.errors.is_empty(), "{:?}", parser.errors);
    ast
}

fn checked(source: &str) -> ModuleIr {
    let mut prelude = parse(include_str!("../../src/stdlib.fih"));
    ast_visit::inject_prelude_intrinsics(&mut prelude);
    let mut loader = ModuleLoader::new();
    let mut ast = loader.resolve_root(parse(source), Path::new("."), prelude);
    assert!(loader.parse_errors.is_empty(), "{:?}", loader.parse_errors);
    let mut checker = TypeChecker::new();
    checker.module_symbols = loader.symbols;
    checker.external_names = loader.external_names;
    checker.check(&mut ast);
    assert!(checker.errors.is_empty(), "{:?}", checker.errors);
    ast.extend(checker.instantiations.clone());
    let mut evaluators = checker.default_evaluators.iter().collect::<Vec<_>>();
    evaluators.sort_by_key(|(name, _)| *name);
    ast.extend(evaluators.into_iter().map(|(_, stmt)| stmt.clone()));
    let plan=crate::ownership::Plan::checked_program(&ast,&checker);
    assert!(plan.errors.is_empty(),"{:?}",plan.errors);
    let module = IrBuilder::new(checker.capture_plans,
        checker.property_indices,
        checker.resolved_calls,
        checker.specialized_methods,
        checker.default_calls,
        checker.traits,
        checker.trait_vtable_layout,
        checker.user_types,
    )
    .with_ownership(plan).build(&ast);
    ir_verify::verify(&module).unwrap();
    module
}

fn rejected(source:&str,diagnostic:&str) {
    let mut ast=parse(source);let mut checker=TypeChecker::new();checker.check(&mut ast);
    assert!(checker.errors.is_empty(),"{:?}",checker.errors);
    ast.extend(checker.instantiations.clone());
    let plan=crate::ownership::Plan::checked_program(&ast,&checker);
    assert!(plan.errors.iter().any(|e|e.message.contains(diagnostic)),"{:?}",plan.errors);
}

fn instructions<'a>(module: &'a ModuleIr, name: &str) -> Vec<&'a Instruction> {
    module
        .functions
        .iter()
        .find(|f| f.name == name)
        .unwrap()
        .blocks
        .iter()
        .flat_map(|b| &b.instructions)
        .collect()
}
fn call_result(module: &ModuleIr, function: &str, target: &str) -> VReg {
    instructions(module, function)
        .into_iter()
        .find_map(|i| match i {
            Instruction::Call {
                dest, func_name, ..
            } if func_name == target => Some(*dest),
            _ => None,
        })
        .unwrap()
}
fn verify_optimized(mut module: ModuleIr) {
    IrOptimizer::optimize(&mut module);
    ir_verify::verify(&module).unwrap();
}

#[test]
fn program_exit_clears_and_releases_globals_in_reverse_order() {
    let module=checked("struct P{pub n:u8,}let first=P{n:1};let second=P{n:2};fn main()->i32{if first.n==1{return 0;}return 1;}");
    let mut initialized=Vec::new();
    for instruction in instructions(&module,"__global_init") {
        if let Instruction::Store {ptr,..}=instruction {
            if module.globals.get(ptr).is_some_and(|t|matches!(t,IrType::Struct(..)|IrType::Array(..)|IrType::Closure(..)|IrType::FatPtr|IrType::Ptr(_)))
                && initialized.last()!=Some(ptr) {initialized.push(*ptr);}
        }
    }
    assert!(initialized.len()>=2);
    initialized.reverse();
    let cleanup=instructions(&module,"__fishy_global_cleanup");
    let mut cleared=Vec::new();
    for (index,instruction) in cleanup.iter().enumerate() {
        if let Instruction::Load {src_ptr,dest,..}=instruction {
            cleared.push(*src_ptr);
            assert!(matches!(cleanup[index+3],Instruction::Store {ptr,..} if ptr==src_ptr));
            assert!(matches!(cleanup[index+4],Instruction::Release {ptr} if ptr==dest));
        }
    }
    assert_eq!(cleared,initialized);
    let exits=instructions(&module,"__fishy_program_exit").into_iter().filter_map(|i| {
        if let Instruction::Call {func_name,..}=i {Some(func_name.as_str())}else{None}
    }).collect::<Vec<_>>();
    assert_eq!(exits,["__fishy_global_cleanup"]);
    verify_optimized(module);
}

#[test]
fn noncopy_temporary_field_extraction_requires_consuming_pattern() {
    rejected("struct P{pub n:u8,}struct O{pub p:P,}fn make()->O{return O{p:P{n:7}};}fn main(){let p=make().p;}","cannot move a field");
}

#[test]
fn argument_pins_are_in_return_cleanup_scopes() {
    let module=checked("struct P{pub n:u8,}fn consume(a:P,b:P){}fn run()->u8{consume(P{n:1},match true{true->{return 7;}false->P{n:0};});return 0;}fn main(){run();}");
    let inst = instructions(&module, "run");
    let pins = inst
        .iter()
        .filter_map(|i| match i {
            Instruction::Alloca { dest, name, .. } if name == "owner.pin" => Some(*dest),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(!pins.is_empty());
    assert!(inst
        .iter()
        .any(|i| matches!(i,Instruction::Load{src_ptr,..}if pins.contains(src_ptr))));
    assert!(inst
        .iter()
        .any(|i| matches!(i, Instruction::Release { .. })));
    verify_optimized(module);
}

#[test]
fn indexing_rejects_effects_that_invalidate_the_evaluated_owner() {
    rejected("struct P{pub n:u8,}let mut a=[P{n:7}];fn index()->i64{a=[P{n:9}];return 0;}fn main(){let n=a[index()].n;}","call cannot access global 'a'");
}

#[test]
fn managed_slot_assignment_transfers_incoming_and_releases_outgoing() {
    let module=checked("struct P{pub n:u8,}struct O{pub p:P,}fn main(){let mut o=O{p:P{n:7}};o.p=P{n:8};}");
    let code=instructions(&module,"main");
    assert!(!code.iter().any(|i|matches!(i,Instruction::Retain {..})));
    assert!(code.iter().enumerate().any(|(index,i)|matches!(i,Instruction::Release {..})&&code[index+1..].iter().any(|i|matches!(i,Instruction::Store {..}))));
    verify_optimized(module);
    rejected("struct P{pub n:u8,}struct O{pub p:P,}fn main(){let mut o=O{p:P{n:7}};o.p=o.p;}","cannot move field");
}

#[test]
fn managed_references_cannot_extend_local_storage_lifetimes() {
    rejected("struct P{pub n:u8,}fn reference()->&P{let p=P{n:7};return &p;}","returned reference points to local storage");
    rejected("struct P{pub n:u8,}fn reference()->&mut P{let mut p=P{n:7};return &p;}","returned reference points to local storage");
    let module=checked("struct P{pub n:u8,}fn reference(p:&P)->&P{return p;}fn main(){let p=P{n:7};let r=reference(p);}");
    assert!(!instructions(&module,"main").iter().any(|i|matches!(i,Instruction::Retain {..})));
    verify_optimized(module);
}

#[test]
fn ignored_for_increment_consumes_its_managed_result() {
    let module=checked("struct P{pub n:u8,}let mut n:i64=0;fn step()->P{n=n+1;return P{n:7};}fn main(){for(;n<3;step()){}}");
    let result = call_result(&module, "main", "step");
    assert_eq!(
        instructions(&module, "main")
            .iter()
            .filter(|i| matches!(i,Instruction::Release{ptr}if *ptr==result))
            .count(),
        1
    );
    verify_optimized(module);
}

#[test]
fn unit_raw_resources_and_primitive_specializations_have_no_arc_edges() {
    let module=checked("struct P{pub n:u8,}fn id<T>(x:T)->T{return x;}fn main(){let u=id(());let n=id(7 as u8);let p=id(P{n:7});let raw=malloc(8);free(raw);}");
    for ty in [Type::U8, Type::Tuple(vec![])] {
        let name = crate::symbols::generic("id", &[ty]);
        assert!(!instructions(&module, &name)
            .iter()
            .any(|i| matches!(i, Instruction::Retain { .. } | Instruction::Release { .. })));
    }
    let managed = crate::symbols::generic("id", &[Type::Custom("P".into())]);
    assert!(!instructions(&module, &managed)
        .iter()
        .any(|i| matches!(i, Instruction::Retain { .. })));
    assert!(instructions(&module, &managed)
        .iter()
        .any(|i| matches!(i, Instruction::Release { .. })));
    verify_optimized(module);
}

#[test]
fn exclusive_replacement_transfers_a_whole_value_without_payload_cloning() {
    let module=checked("struct P{pub n:u8,}struct O{pub p:P,pub q:P,}fn main(){let mut o=O{p:P{n:7},q:P{n:8}};let r:&mut O=&o;^r=O{p:P{n:9},q:P{n:10}};}");
    let code=instructions(&module,"main");
    assert!(!code.iter().any(|i|matches!(i,Instruction::Retain {..}|Instruction::IndirectCall {..})));
    assert!(code.iter().any(|i|matches!(i,Instruction::Release {..})));
    verify_optimized(module);
    rejected("struct P{pub n:u8,}struct O{pub p:P,pub q:P,}fn main(){let p=P{n:7};let o=O{p:p,q:p};}","was moved");
}

#[test]
fn compiler_evaluation_pins_do_not_become_source_captures() {
    let module=checked("struct P{pub n:u8,}fn consume(p:P,mut f:fn()->u8)->u8{return f();}fn main(){consume(P{n:7},fn()->u8{return 9;});}");
    let envs = module
        .structs
        .keys()
        .filter(|n| n.starts_with("__fishy_lambda_") && n.ends_with(".env"))
        .collect::<Vec<_>>();
    assert!(
        envs.is_empty(),
        "an argument pin is not a source-visible capture"
    );
    verify_optimized(module);
}

#[test]
fn complete_place_updates_use_an_explicit_mutable_owner() {
    let module = checked("struct P{pub n:u8,}fn make()->P{return P{n:7};}fn main(){let mut p=make();++p.n;}");
    let owner = call_result(&module, "main", "make");
    let inst = instructions(&module, "main");
    let pin = inst
        .iter()
        .find_map(|i| match i {
            Instruction::Store { value, ptr, .. } if *value == owner => Some(*ptr),
            _ => None,
        })
        .unwrap();
    assert!(inst
        .iter()
        .any(|i| matches!(i,Instruction::Load{src_ptr,..}if *src_ptr==pin)));
    assert!(inst
        .iter()
        .any(|i| matches!(i, Instruction::Release { .. })));
    verify_optimized(module);
}

#[test]
fn managed_identity_operands_have_scoped_owners() {
    let module = checked(
        "struct P{pub n:u8,}fn make()->P{return P{n:7};}fn main(){let same=make()==make();}",
    );
    let inst = instructions(&module, "main");
    let owners = inst
        .iter()
        .filter_map(|i| match i {
            Instruction::Call {
                dest, func_name, ..
            } if func_name == "make" => Some(*dest),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(owners.len(), 2);
    for owner in owners {
        let pin = inst
            .iter()
            .find_map(|i| match i {
                Instruction::Store { value, ptr, .. } if *value == owner => Some(*ptr),
                _ => None,
            })
            .unwrap();
        assert!(inst
            .iter()
            .any(|i| matches!(i,Instruction::Load{src_ptr,..}if *src_ptr==pin)));
    }
    assert_eq!(
        inst.iter()
            .filter(|i| matches!(i, Instruction::Release { .. }))
            .count(),
        2
    );
    verify_optimized(module);
}

#[test]
fn recursive_generic_binding_keeps_its_managed_slot_and_scope_release() {
    let module = checked(
        "struct Node<T>{pub value:[T;1],pub next:Option<Node<T>>,}\
         fn main(){let end:Option<Node<u8>>=Option.None();\
         let n=Node<u8>{value:[1],next:end};}",
    );
    let inst = instructions(&module, "main");
    let node = inst
        .iter()
        .find_map(|i| match i {
            Instruction::AllocStruct { class_name, .. } => Some(class_name),
            _ => None,
        })
        .unwrap();
    let next_type = &module.structs[node][1];
    let (slot, ty) = inst
        .iter()
        .find_map(|i| match i {
            Instruction::Alloca { dest, name, ty } if name == "end" => Some((*dest, ty)),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        ty, next_type,
        "the local and recursive field must use the same managed handle type"
    );
    assert!(matches!(ty, IrType::Ptr(inner) if matches!(inner.as_ref(), IrType::Struct(_, _))));
    assert!(
        inst.iter().any(|i| match i {
            Instruction::Load { dest, src_ptr, .. } if *src_ptr == slot => {
                inst.iter()
                    .any(|i| matches!(i, Instruction::Release { ptr } if ptr == dest))
            }
            _ => false,
        }),
        "the local owns a scope edge independently of the node's field edge"
    );
    verify_optimized(module);
}

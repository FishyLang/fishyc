//! Current acyclic ARC invariants at the checked source/custom-IR boundary.
use super::IrBuilder;
use crate::ast::{Expr, Stmt, Type};
use crate::ast_visit::{self, Visitor};
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
    let mut prelude = parse(include_str!("stdlib.fih"));
    struct RuntimeSize;
    impl Visitor for RuntimeSize {
        fn expr(&mut self, expr: &mut Expr) {
            if matches!(expr, Expr::Variable(n) if n.lexeme == "__fishy_storage_size") {
                *expr = Expr::StorageSize(Type::Custom("T".into()));
            }
        }
    }
    ast_visit::statements(&mut prelude, &mut RuntimeSize);
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
    let module = IrBuilder::new(
        checker.property_indices,
        checker.resolved_calls,
        checker.specialized_methods,
        checker.default_calls,
        checker.traits,
        checker.trait_vtable_layout,
        checker.user_types,
    )
    .build(&ast);
    ir_verify::verify(&module).unwrap();
    module
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
fn temporary_field_extraction_releases_parent_exactly_once() {
    let module=checked("struct P{pub n:u8,}struct O{pub p:P,}fn make()->O{return O{p:P{n:7}};}fn main(){let p=make().p;}");
    let parent = call_result(&module, "main", "make");
    let inst = instructions(&module, "main");
    assert_eq!(
        inst.iter()
            .filter(|i| matches!(i,Instruction::Release{ptr}if *ptr==parent))
            .count(),
        1
    );
    let release = inst
        .iter()
        .position(|i| matches!(i,Instruction::Release{ptr}if *ptr==parent))
        .unwrap();
    assert!(inst[..release]
        .iter()
        .any(|i| matches!(i, Instruction::Retain { .. })));
    verify_optimized(module);
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
fn indexing_pins_the_evaluated_array_before_index_effects() {
    let module=checked("struct P{pub n:u8,}let a=[P{n:7}];fn index()->i64{a=[P{n:9}];return 0;}fn main(){let p=a[index()];}");
    let inst = instructions(&module, "main");
    let effect = inst
        .iter()
        .position(|i| matches!(i,Instruction::Call{func_name,..}if func_name=="index"))
        .unwrap();
    let pin = inst[..effect]
        .iter()
        .find_map(|i| match i {
            Instruction::Alloca { dest, name, .. } if name == "owner.pin" => Some(*dest),
            _ => None,
        })
        .unwrap();
    let base = inst[..effect]
        .iter()
        .find_map(|i| match i {
            Instruction::Store { ptr, value, .. } if *ptr == pin => Some(*value),
            _ => None,
        })
        .unwrap();
    assert!(inst[..effect]
        .iter()
        .any(|i| matches!(i,Instruction::Retain{ptr}if *ptr==base)));
    assert!(inst[effect..]
        .iter()
        .any(|i| matches!(i,Instruction::Load{src_ptr,..}if *src_ptr==pin)));
    verify_optimized(module);
}

#[test]
fn managed_slot_assignment_retains_incoming_before_releasing_outgoing() {
    let module =
        checked("struct P{pub n:u8,}struct O{pub p:P,}fn main(){let o=O{p:P{n:7}};o.p=o.p;}");
    let inst = instructions(&module, "main");
    let mut ordered = false;
    for (store_index, i) in inst.iter().enumerate() {
        if let Instruction::Store { value, ptr, .. } = i {
            for (load_index, j) in inst[..store_index].iter().enumerate() {
                if let Instruction::Load { dest, src_ptr, .. } = j {
                    if src_ptr == ptr {
                        let release = inst[load_index + 1..store_index]
                            .iter()
                            .position(|k| matches!(k,Instruction::Release{ptr}if ptr==dest));
                        if let Some(release) = release {
                            let release = release + load_index + 1;
                            ordered |= inst[..release]
                                .iter()
                                .any(|k| matches!(k,Instruction::Retain{ptr}if ptr==value));
                        }
                    }
                }
            }
        }
    }
    assert!(ordered);
    verify_optimized(module);
}

#[test]
fn owned_managed_reference_reads_and_replacements_transfer_their_pin() {
    for source in ["struct P{pub n:u8,}fn reference()->&P{let p=P{n:7};return &p;}fn main(){let p:P=^reference();}","struct P{pub n:u8,}fn reference()->&mut P{let p=P{n:7};return &p;}fn main(){let p=^reference()=P{n:9};}"]{
        let module=checked(source);let reference=call_result(&module,"main","reference");
        assert!(!instructions(&module,"main").iter().any(|i|matches!(i,Instruction::Retain{ptr}if *ptr==reference)));
        verify_optimized(module);
    }
}

#[test]
fn ignored_for_increment_consumes_its_managed_result() {
    let module=checked("struct P{pub n:u8,}let n:i64=0;fn step()->P{n=n+1;return P{n:7};}fn main(){for(;n<3;step()){}}");
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
    assert!(instructions(&module, &managed)
        .iter()
        .any(|i| matches!(i, Instruction::Retain { .. })));
    assert!(instructions(&module, &managed)
        .iter()
        .any(|i| matches!(i, Instruction::Release { .. })));
    verify_optimized(module);
}

#[test]
fn replacement_retains_incoming_children_before_invoking_old_destructor() {
    let module=checked("struct P{pub n:u8,}struct O{pub p:P,pub q:P,}fn main(){let p=P{n:7};let o=O{p:p,q:p};let r:&mut O=&o;^r=O{p:o.q,q:o.p};}");
    let inst = instructions(&module, "main");
    let destructor = inst
        .iter()
        .position(|i| {
            matches!(
                i,
                Instruction::IndirectCall {
                    ret_type: IrType::Void,
                    ..
                }
            )
        })
        .unwrap();
    let incoming = inst
        .iter()
        .filter_map(|i| match i {
            Instruction::AllocStruct { dest, class_name } if class_name == "O" => Some(*dest),
            _ => None,
        })
        .last()
        .unwrap();
    let fields = inst[..destructor]
        .iter()
        .filter_map(|i| match i {
            Instruction::GetElementPtr { dest, base_ptr, .. } if *base_ptr == incoming => {
                Some(*dest)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let children = inst[..destructor]
        .iter()
        .filter_map(|i| match i {
            Instruction::Load {
                dest,
                src_ptr,
                ty: IrType::Ptr(inner),
            } if fields.contains(src_ptr)
                && matches!(inner.as_ref(),IrType::Struct(name,_) if name == "P") =>
            {
                Some(*dest)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(children.len(), 2);
    for child in children {
        assert!(inst[..destructor]
            .iter()
            .any(|i| matches!(i,Instruction::Retain{ptr} if *ptr == child)));
    }
    assert!(inst[destructor + 1..]
        .iter()
        .any(|i| matches!(i, Instruction::Store { .. })));
    verify_optimized(module);
}

#[test]
fn compiler_evaluation_pins_do_not_become_source_captures() {
    let module=checked("struct P{pub n:u8,}fn consume(p:P,f:fn()->u8)->u8{return f();}fn main(){consume(P{n:7},fn()->u8{return 9;});}");
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
fn complete_place_updates_keep_temporary_owners_in_scope() {
    let module = checked("struct P{pub n:u8,}fn make()->P{return P{n:7};}fn main(){++make().n;}");
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

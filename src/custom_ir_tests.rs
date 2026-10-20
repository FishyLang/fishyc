//! Hand-authored checked-AST/IR contracts, without LLVM or native execution.
use super::IrBuilder;
use crate::ast::{Expr, Stmt, Type};
use crate::ast_visit::{self, Visitor};
use crate::ir::*;
use crate::ir_optimizer::IrOptimizer;
use crate::{
    ir_verify, module_loader::ModuleLoader, parser::Parser, scanner::Scanner,
    type_checker::TypeChecker,
};
use std::collections::HashMap;
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

fn checked(source: &str) -> (ModuleIr, Vec<String>) {
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
    let selected_methods = checker
        .specialized_methods
        .values()
        .map(|(name, _)| name.clone())
        .collect();
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
    (module, selected_methods)
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

fn verify_both(mut module: ModuleIr) {
    ir_verify::verify(&module).unwrap();
    IrOptimizer::optimize(&mut module);
    ir_verify::verify(&module).unwrap();
}

#[test]
fn terminal_first_merges_use_the_checked_result_type() {
    let (module, _) = checked("struct P{pub n:u8,}fn main(){let a=match false {true->panic(\"unused\");false->P{n:7};};let b=false?(panic(\"unused\")):P{n:8};}");
    let slots = instructions(&module, "main")
        .into_iter()
        .filter_map(|i| match i {
            Instruction::Alloca { name, ty, .. }
                if name == "match.result" || name == "ternary.result" =>
            {
                Some(ty)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        slots,
        vec![&IrType::Ptr(Box::new(IrType::Struct("P".into(), vec![]))); 2]
    );
    verify_both(module);
}

#[test]
fn slice_bindings_and_merge_slots_keep_the_slice_type() {
    let (module, _) = checked("fn main(){let s:[u8;?]=[1,2];let a:[u8;?]=true?[3,4]:s;let b:[u8;?]=match true {true->[5,6];false->s;};}");
    let mut slots = 0;
    for i in instructions(&module, "main") {
        if let Instruction::Alloca { name, ty, .. } = i {
            if matches!(
                name.as_str(),
                "s" | "a" | "b" | "match.result" | "ternary.result"
            ) {
                assert_eq!(*ty, IrType::Array(0, Box::new(IrType::U8)), "{name}");
                slots += 1;
            }
        }
    }
    assert_eq!(slots, 5);
    assert!(instructions(&module, "main")
        .iter()
        .any(|i| matches!(i, Instruction::AllocArray { ty: IrType::U8, .. })));
    verify_both(module);
}

#[test]
fn grouped_calls_consume_exact_specialization_targets() {
    let (module, selected) = checked("struct P{}impl P{pub fn id<T>(self,x:T)->T{return x;}}fn id<T>(x:T)->T{return x;}fn main(){let p=P{};let a:u8=(id)(7);let b:f32=(p.id)(1.5 as f32);}");
    let calls = instructions(&module, "main")
        .into_iter()
        .filter_map(|i| match i {
            Instruction::Call { func_name, .. } => Some(func_name),
            _ => None,
        })
        .collect::<Vec<_>>();
    for target in selected {
        assert!(calls.contains(&&target), "lost method target {target}");
    }
    let id = crate::symbols::generic("id", &[Type::U8]);
    assert!(calls.contains(&&id));
    let target = module.functions.iter().find(|f| f.name == id).unwrap();
    assert_eq!(target.args[0].1, IrType::U8);
    assert_eq!(target.ret_type, IrType::U8);
    verify_both(module);
}

#[test]
fn default_plans_emit_full_arity_for_grouped_generic_calls() {
    let (module, _) =
        checked("fn pick<T>(a:T,b:T=a,c:T=b)->T{return c;}fn main(){let n:u8=(pick)(7);}");
    let name = crate::symbols::generic("pick", &[Type::U8]);
    let calls = instructions(&module, "main")
        .into_iter()
        .filter_map(|i| match i {
            Instruction::Call {
                func_name, args, ..
            } if func_name == &name => Some(args.len()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(calls, vec![3]);
    verify_both(module);
}

#[test]
fn scalar_storage_types_survive_materialization() {
    let (module, _) = checked("fn main(){let a:i8=-1;let b:u8=1;let c:i16=-1;let d:u16=1;let e:i32=-1;let f:u32=1;let g:i64=-1;let h:u64=1;let i:f16=1.5;let j:f32=2.5;let k:f64=3.5;let l:bool=true;let m:string=\"fish\";let n:()=();}");
    let expected = [
        IrType::I8,
        IrType::U8,
        IrType::I16,
        IrType::U16,
        IrType::I32,
        IrType::U32,
        IrType::I64,
        IrType::U64,
        IrType::F16,
        IrType::F32,
        IrType::F64,
        IrType::Bool,
        IrType::Ptr(Box::new(IrType::I8)),
        IrType::I8,
    ];
    let slots = instructions(&module, "main")
        .into_iter()
        .filter_map(|i| match i {
            Instruction::Alloca { name, ty, .. } if name.len() == 1 => Some(ty.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(slots, expected);
    verify_both(module);
}

#[test]
fn sizeof_uses_concrete_types_from_generic_storage() {
    let (module, _) = checked("struct P{pub n:u8,}fn sizes(a:Vec<u8>,b:Vec<i64>,c:Vec<f16>,d:Vec<f32>,e:Vec<(u8,)>,f:Vec<P>,g:Vec<[u8;2]>,h:Vec<((u8,),[u16;2])>){}fn main(){}");
    let sizes = module
        .functions
        .iter()
        .flat_map(|f| &f.blocks)
        .flat_map(|b| &b.instructions)
        .filter_map(|i| match i {
            Instruction::SizeOf { ty, .. } => Some(ty),
            _ => None,
        })
        .collect::<Vec<_>>();
    for ty in [
        IrType::U8,
        IrType::I64,
        IrType::F16,
        IrType::F32,
        IrType::Array(2, Box::new(IrType::U8)),
        IrType::Ptr(Box::new(IrType::Struct("P".into(), vec![]))),
    ] {
        assert!(sizes.contains(&&ty), "missing storage type {ty:?}");
    }
    assert!(sizes.iter().any(|t|matches!(t,IrType::Ptr(inner) if matches!(inner.as_ref(),IrType::Struct(n,fields) if n.starts_with("__tuple_") && !fields.is_empty()))));
    ir_verify::verify(&module).unwrap();
}

fn function(name: &str, ret_type: IrType, instructions: Vec<Instruction>) -> FunctionIr {
    FunctionIr {
        name: name.into(),
        args: vec![],
        ret_type,
        is_variadic: false,
        blocks: vec![BasicBlock {
            id: BlockId(0),
            name: "entry".into(),
            instructions,
        }],
    }
}
fn module(functions: Vec<FunctionIr>) -> ModuleIr {
    ModuleIr {
        name: "contract".into(),
        globals: HashMap::new(),
        structs: HashMap::new(),
        vtables: HashMap::new(),
        functions,
    }
}

#[test]
fn inlining_remaps_values_independent_of_block_listing_order() {
    let mut callee = function(
        "select",
        IrType::I64,
        vec![Instruction::Br { target: BlockId(2) }],
    );
    callee.blocks.extend([
        BasicBlock {
            id: BlockId(1),
            name: "exit".into(),
            instructions: vec![Instruction::Ret {
                value: Some(VReg(8)),
            }],
        },
        BasicBlock {
            id: BlockId(2),
            name: "definition".into(),
            instructions: vec![
                Instruction::ConstInt {
                    dest: VReg(8),
                    value: 7,
                },
                Instruction::Br { target: BlockId(1) },
            ],
        },
    ]);
    let caller = function(
        "main",
        IrType::I64,
        vec![
            Instruction::Call {
                dest: VReg(9),
                func_name: "select".into(),
                args: vec![],
            },
            Instruction::Ret {
                value: Some(VReg(9)),
            },
        ],
    );
    verify_both(module(vec![caller, callee]));
}

#[test]
fn verifier_rejects_wrong_call_and_return_value_types() {
    let mut callee = function("float", IrType::F32, vec![]);
    callee.blocks.clear();
    callee.args = vec![(VReg(5), IrType::F32)];
    let caller = function(
        "main",
        IrType::Void,
        vec![
            Instruction::ConstString {
                dest: VReg(0),
                value: "wrong".into(),
            },
            Instruction::Call {
                dest: VReg(1),
                func_name: "float".into(),
                args: vec![VReg(0)],
            },
            Instruction::Ret { value: None },
        ],
    );
    assert!(
        ir_verify::verify(&module(vec![caller, callee])).is_err(),
        "pointer argument accepted for f32"
    );
    let f = function(
        "main",
        IrType::F32,
        vec![
            Instruction::ConstString {
                dest: VReg(0),
                value: "wrong".into(),
            },
            Instruction::Ret {
                value: Some(VReg(0)),
            },
        ],
    );
    assert!(
        ir_verify::verify(&module(vec![f])).is_err(),
        "pointer return accepted for f32"
    );
    let mut array = function("array", IrType::Void, vec![]);
    array.blocks.clear();
    array.args = vec![(VReg(5), IrType::Array(2, Box::new(IrType::U64)))];
    let caller = function(
        "main",
        IrType::Void,
        vec![
            Instruction::ConstInt {
                dest: VReg(0),
                value: 2,
            },
            Instruction::AllocArray {
                dest: VReg(1),
                size: VReg(0),
                ty: IrType::U8,
            },
            Instruction::Call {
                dest: VReg(2),
                func_name: "array".into(),
                args: vec![VReg(1)],
            },
            Instruction::Ret { value: None },
        ],
    );
    assert!(
        ir_verify::verify(&module(vec![caller, array])).is_err(),
        "register normalization must not change array element widths"
    );
    let mut target = function(
        "closure",
        IrType::U8,
        vec![Instruction::Ret {
            value: Some(VReg(6)),
        }],
    );
    target.args = vec![
        (VReg(5), IrType::Ptr(Box::new(IrType::Any))),
        (VReg(6), IrType::U8),
    ];
    let caller = function(
        "main",
        IrType::Void,
        vec![
            Instruction::ConstInt {
                dest: VReg(0),
                value: 0,
            },
            Instruction::MakeClosure {
                dest: VReg(1),
                fn_name: "closure".into(),
                env_ptr: VReg(0),
            },
            Instruction::ConstInt {
                dest: VReg(2),
                value: 7,
            },
            Instruction::CallClosure {
                dest: VReg(3),
                closure_ptr: VReg(1),
                args: vec![VReg(2)],
                arg_types: vec![IrType::I64],
                ret_type: IrType::I64,
            },
            Instruction::Ret { value: None },
        ],
    );
    assert!(
        ir_verify::verify(&module(vec![caller, target])).is_err(),
        "callable declarations must preserve integer widths"
    );
}

#[test]
fn verifier_rejects_non_address_load_and_mismatched_store() {
    let f = function(
        "main",
        IrType::Void,
        vec![
            Instruction::ConstFloat {
                dest: VReg(0),
                value: 1.5,
                ty: IrType::F32,
            },
            Instruction::Load {
                dest: VReg(1),
                ty: IrType::I64,
                src_ptr: VReg(0),
            },
            Instruction::Ret { value: None },
        ],
    );
    assert!(
        ir_verify::verify(&module(vec![f])).is_err(),
        "floating value used as load address"
    );
    let f = function(
        "main",
        IrType::Void,
        vec![
            Instruction::Alloca {
                dest: VReg(0),
                name: "n".into(),
                ty: IrType::I64,
            },
            Instruction::ConstString {
                dest: VReg(1),
                value: "wrong".into(),
            },
            Instruction::Store {
                ty: IrType::I64,
                value: VReg(1),
                ptr: VReg(0),
            },
            Instruction::Ret { value: None },
        ],
    );
    assert!(
        ir_verify::verify(&module(vec![f])).is_err(),
        "pointer stored into integer storage without a cast"
    );
    let f = function(
        "main",
        IrType::Void,
        vec![
            Instruction::Alloca {
                dest: VReg(0),
                name: "n".into(),
                ty: IrType::I64,
            },
            Instruction::Load {
                dest: VReg(1),
                ty: IrType::F32,
                src_ptr: VReg(0),
            },
            Instruction::Ret { value: None },
        ],
    );
    assert!(
        ir_verify::verify(&module(vec![f])).is_err(),
        "load disagrees with typed storage"
    );
    let f = function(
        "main",
        IrType::Void,
        vec![
            Instruction::Alloca {
                dest: VReg(0),
                name: "n".into(),
                ty: IrType::I64,
            },
            Instruction::ConstFloat {
                dest: VReg(1),
                ty: IrType::F32,
                value: 1.5,
            },
            Instruction::Store {
                ty: IrType::F32,
                value: VReg(1),
                ptr: VReg(0),
            },
            Instruction::Ret { value: None },
        ],
    );
    assert!(
        ir_verify::verify(&module(vec![f])).is_err(),
        "store disagrees with typed storage"
    );
}

#[test]
fn verifier_rejects_invalid_condition_and_indirect_arity() {
    let f = function(
        "main",
        IrType::Void,
        vec![
            Instruction::ConstFloat {
                dest: VReg(0),
                value: 1.5,
                ty: IrType::F32,
            },
            Instruction::CondBr {
                cond: VReg(0),
                if_true: BlockId(0),
                if_false: BlockId(0),
            },
        ],
    );
    assert!(
        ir_verify::verify(&module(vec![f])).is_err(),
        "floating branch condition"
    );
    let f = function(
        "main",
        IrType::Void,
        vec![
            Instruction::ConstInt {
                dest: VReg(0),
                value: 0,
            },
            Instruction::IndirectCall {
                dest: VReg(1),
                fn_ptr: VReg(0),
                args: vec![],
                arg_types: vec![IrType::I64],
                ret_type: IrType::Void,
            },
            Instruction::Ret { value: None },
        ],
    );
    assert!(
        ir_verify::verify(&module(vec![f])).is_err(),
        "indirect signature arity mismatch"
    );
}

#[test]
fn verifier_rejects_duplicate_parameter_definitions_and_missing_function_values() {
    let mut f = function("main", IrType::Void, vec![Instruction::Ret { value: None }]);
    f.args = vec![(VReg(0), IrType::I64), (VReg(0), IrType::I64)];
    assert!(
        ir_verify::verify(&module(vec![f])).is_err(),
        "duplicate parameters"
    );
    let f = function(
        "main",
        IrType::Void,
        vec![
            Instruction::ConstInt {
                dest: VReg(0),
                value: 0,
            },
            Instruction::MakeClosure {
                dest: VReg(1),
                fn_name: "missing".into(),
                env_ptr: VReg(0),
            },
            Instruction::Ret { value: None },
        ],
    );
    assert!(
        ir_verify::verify(&module(vec![f])).is_err(),
        "missing closure function"
    );
}

#[test]
fn optimizer_preserves_entry_with_a_backedge() {
    let mut f = function(
        "main",
        IrType::I64,
        vec![Instruction::CondBr {
            cond: VReg(0),
            if_true: BlockId(2),
            if_false: BlockId(1),
        }],
    );
    f.args = vec![(VReg(0), IrType::Bool)];
    f.blocks.extend([
        BasicBlock {
            id: BlockId(1),
            name: "exit".into(),
            instructions: vec![
                Instruction::ConstInt {
                    dest: VReg(1),
                    value: 7,
                },
                Instruction::Ret {
                    value: Some(VReg(1)),
                },
            ],
        },
        BasicBlock {
            id: BlockId(2),
            name: "backedge".into(),
            instructions: vec![Instruction::Br { target: BlockId(0) }],
        },
    ]);
    let mut m = module(vec![f]);
    ir_verify::verify(&m).unwrap();
    IrOptimizer::optimize(&mut m);
    ir_verify::verify(&m).unwrap();
    assert_eq!(
        m.functions[0].blocks[0].id,
        BlockId(0),
        "entry was replaced by an exit block"
    );
    assert!(m.functions[0]
        .blocks
        .iter()
        .flat_map(|b| &b.instructions)
        .any(|i| matches!(i, Instruction::CondBr { .. })));
}

#[test]
fn verifier_rejects_out_of_bounds_field_and_void_size() {
    let f = function(
        "main",
        IrType::Void,
        vec![
            Instruction::AllocStruct {
                dest: VReg(0),
                class_name: "P".into(),
            },
            Instruction::ConstInt {
                dest: VReg(1),
                value: 0,
            },
            Instruction::ConstInt {
                dest: VReg(2),
                value: 2,
            },
            Instruction::GetElementPtr {
                dest: VReg(3),
                base_ty: IrType::Struct("P".into(), vec![]),
                base_ptr: VReg(0),
                indices: vec![VReg(1), VReg(2)],
            },
            Instruction::Ret { value: None },
        ],
    );
    let mut m = module(vec![f]);
    m.structs.insert("P".into(), vec![IrType::U8]);
    assert!(ir_verify::verify(&m).unwrap_err().contains("out of bounds"));
    m.functions[0].blocks[0].instructions[2] = Instruction::ConstInt {
        dest: VReg(2),
        value: 0,
    };
    if let Instruction::GetElementPtr { base_ty, .. } =
        &mut m.functions[0].blocks[0].instructions[3]
    {
        *base_ty = IrType::Struct("P".into(), vec![IrType::F32]);
    }
    assert!(ir_verify::verify(&m)
        .unwrap_err()
        .contains("Conflicting field layout"));
    let f = function(
        "main",
        IrType::Void,
        vec![
            Instruction::SizeOf {
                dest: VReg(0),
                ty: IrType::Void,
            },
            Instruction::Ret { value: None },
        ],
    );
    assert!(ir_verify::verify(&module(vec![f])).is_err());
}

#[test]
fn sizeof_identity_and_effects_survive_optimization() {
    let layouts = vec![
        IrType::U8,
        IrType::I64,
        IrType::F16,
        IrType::F32,
        IrType::Array(2, Box::new(IrType::U8)),
        IrType::Ptr(Box::new(IrType::Struct("P".into(), vec![]))),
        IrType::Struct("Pair".into(), vec![IrType::U8, IrType::F64]),
    ];
    let mut body = layouts
        .iter()
        .enumerate()
        .map(|(n, ty)| Instruction::SizeOf {
            dest: VReg(n),
            ty: ty.clone(),
        })
        .collect::<Vec<_>>();
    body.push(Instruction::Call {
        dest: VReg(20),
        func_name: "observe".into(),
        args: (0..layouts.len()).map(VReg).collect(),
    });
    body.extend([
        Instruction::ConstInt {
            dest: VReg(21),
            value: 1,
        },
        Instruction::Alloca {
            dest: VReg(22),
            name: "live".into(),
            ty: IrType::I64,
        },
        Instruction::Store {
            ty: IrType::I64,
            value: VReg(21),
            ptr: VReg(22),
        },
        Instruction::AllocArray {
            dest: VReg(23),
            size: VReg(21),
            ty: IrType::U8,
        },
        Instruction::Retain { ptr: VReg(23) },
        Instruction::Release { ptr: VReg(23) },
        Instruction::Ret { value: None },
    ]);
    let main = function("main", IrType::Void, body);
    let mut observer = function("observe", IrType::Void, vec![]);
    observer.blocks.clear();
    observer.args = (0..layouts.len())
        .map(|n| (VReg(100 + n), IrType::U64))
        .collect();
    let mut m = module(vec![main, observer]);
    ir_verify::verify(&m).unwrap();
    IrOptimizer::optimize(&mut m);
    ir_verify::verify(&m).unwrap();
    let code = instructions(&m, "main");
    let sizes = code
        .iter()
        .filter_map(|i| match i {
            Instruction::SizeOf { ty, .. } => Some(ty.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(sizes, layouts);
    assert!(code
        .iter()
        .any(|i| matches!(i,Instruction::Call {func_name,..} if func_name=="observe")));
    assert!(code.iter().any(|i| matches!(i, Instruction::Store { .. })));
    assert!(code
        .iter()
        .any(|i| matches!(i, Instruction::AllocArray { .. })));
    assert!(code.iter().any(|i| matches!(i, Instruction::Retain { .. })));
    assert!(code
        .iter()
        .any(|i| matches!(i, Instruction::Release { .. })));
}

#[test]
fn grouped_generic_enum_keeps_the_selected_constructor_owner() {
    let (module, _) = checked("enum E<T>{One(T),}fn main(){let e:E<u8>=(E.One)(255 as u8);}");
    let owner = crate::symbols::generic("E", &[Type::U8]);
    let target = crate::symbols::member(&owner, "One");
    assert!(instructions(&module, "main")
        .iter()
        .any(|i| matches!(i,Instruction::Call {func_name,..} if func_name==&target)));
    let constructor = module.functions.iter().find(|f| f.name == target).unwrap();
    assert_eq!(constructor.args[0].1, IrType::U8);
    assert_eq!(
        constructor.ret_type,
        IrType::Ptr(Box::new(IrType::Struct(owner, vec![])))
    );
    verify_both(module);
}

#[test]
fn managed_replacement_uses_pointer_typed_destructor_views() {
    let (module, _) =
        checked("struct P{pub n:u8,}fn main(){let p=P{n:1};let r:&mut P=&p;^r=P{n:2};}");
    let code = instructions(&module, "main");
    let offsets = code
        .iter()
        .filter_map(|i| match i {
            Instruction::ConstInt { dest, value: -3 } => Some(*dest),
            _ => None,
        })
        .collect::<Vec<_>>();
    let mut views = 0;
    for i in &code {
        if let Instruction::GetElementPtr {
            base_ty, indices, ..
        } = i
        {
            if indices.len() == 1 && offsets.contains(&indices[0]) {
                assert_eq!(*base_ty, IrType::Ptr(Box::new(IrType::U8)));
                views += 1;
            }
        }
    }
    assert!(views >= 2);
    verify_both(module);
    let (trait_object, _)=checked("trait Read{fn read(self)->u8;}struct P{pub n:u8,}impl Read for P{pub fn read(self)->u8{return self.n;}}fn main(){let t=P{n:1} as Read;let r:&mut Read=&t;^r=P{n:2} as Read;}");
    verify_both(trait_object);
}

#[test]
fn inlining_fresh_registers_do_not_alias_module_globals() {
    let callee = function(
        "value",
        IrType::I64,
        vec![
            Instruction::ConstInt {
                dest: VReg(1),
                value: 7,
            },
            Instruction::Ret {
                value: Some(VReg(1)),
            },
        ],
    );
    let main = function(
        "main",
        IrType::I64,
        vec![
            Instruction::Call {
                dest: VReg(0),
                func_name: "value".into(),
                args: vec![],
            },
            Instruction::Ret {
                value: Some(VReg(0)),
            },
        ],
    );
    let mut m = module(vec![main, callee]);
    m.globals.insert(VReg(2), IrType::I64);
    verify_both(m);
}

#[test]
fn pointer_poisoning_uses_the_underlying_place_type() {
    let (module, _) = checked("fn main(){let p:^i64=malloc(8) as ^i64;free(((p) as ^u8));}");
    let code = instructions(&module, "main");
    let slot = code
        .iter()
        .find_map(|i| match i {
            Instruction::Alloca { dest, name, .. } if name == "p" => Some(*dest),
            _ => None,
        })
        .unwrap();
    let stores = code
        .iter()
        .filter_map(|i| match i {
            Instruction::Store { ty, ptr, .. } if *ptr == slot => Some(ty),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(stores.len(), 3);
    assert!(stores
        .iter()
        .all(|ty| **ty == IrType::Ptr(Box::new(IrType::I64))));
    verify_both(module);
}

//! Focused valid custom-IR -> LLVM representation and ABI contracts.
use super::*;
use crate::ir::{BasicBlock, BlockId, FunctionIr};
use crate::ir_verify;
use inkwell::values::InstructionOpcode;

fn function(
    name: &str,
    args: Vec<(VReg, IrType)>,
    ret_type: IrType,
    instructions: Vec<Instruction>,
) -> FunctionIr {
    FunctionIr {
        name: name.into(),
        args,
        ret_type,
        is_variadic: false,
        blocks: if instructions.is_empty() {
            vec![]
        } else {
            vec![BasicBlock {
                name: "test".into(),
                id: BlockId(0),
                instructions,
            }]
        },
    }
}
fn module(functions: Vec<FunctionIr>) -> ModuleIr {
    let mut functions = functions;
    functions.insert(
        0,
        function(
            "free",
            vec![(VReg(1000), IrType::Ptr(Box::new(IrType::U8)))],
            IrType::Void,
            vec![],
        ),
    );
    ModuleIr {
        name: "abi-test".into(),
        functions,
        globals: HashMap::new(),
        structs: HashMap::new(),
        vtables: HashMap::new(),
    }
}
fn compile<'ctx>(emitter: &mut LlvmEmitter<'ctx>, module: &ModuleIr) {
    ir_verify::verify(module).expect("test must enter the backend with valid custom IR");
    emitter.compile(module, &HashMap::new()).unwrap();
    emitter.module.verify().unwrap();
}

#[test]
fn scalar_memory_types_and_target_sizes_are_exact() {
    let context = Context::create();
    let e = LlvmEmitter::new(&context, "scalar-layout");
    for (ty, bits, bytes) in [
        (IrType::I8, 8, 1),
        (IrType::U8, 8, 1),
        (IrType::I16, 16, 2),
        (IrType::U16, 16, 2),
        (IrType::I32, 32, 4),
        (IrType::U32, 32, 4),
        (IrType::I64, 64, 8),
        (IrType::U64, 64, 8),
        (IrType::Bool, 1, 1),
    ] {
        let llvm = e.get_llvm_type(&ty);
        assert_eq!(llvm.into_int_type().get_bit_width(), bits);
        assert_eq!(
            e.target_machine.get_target_data().get_abi_size(&llvm),
            bytes
        );
    }
    for (ty, llvm, bytes) in [
        (IrType::F16, context.f16_type(), 2),
        (IrType::F32, context.f32_type(), 4),
        (IrType::F64, context.f64_type(), 8),
    ] {
        assert_eq!(e.get_llvm_type(&ty), llvm.into());
        assert_eq!(
            e.target_machine.get_target_data().get_abi_size(&llvm),
            bytes
        );
    }
}

#[test]
fn handles_are_pointers_and_ranges_are_inline() {
    let c = Context::create();
    let e = LlvmEmitter::new(&c, "handle-layout");
    let pointer_bytes = e
        .target_machine
        .get_target_data()
        .get_pointer_byte_size(None) as u64;
    for ty in [
        IrType::Ptr(Box::new(IrType::U8)),
        IrType::Ptr(Box::new(IrType::Struct("P".into(), vec![IrType::U8]))),
        IrType::Array(3, Box::new(IrType::U16)),
        IrType::Array(0, Box::new(IrType::U16)),
        IrType::Closure(vec![IrType::U8], Box::new(IrType::F32)),
        IrType::FatPtr,
    ] {
        let llvm = e.get_llvm_type(&ty);
        assert!(llvm.is_pointer_type());
        assert_eq!(
            e.target_machine.get_target_data().get_abi_size(&llvm),
            pointer_bytes
        );
    }
    let r = e
        .get_llvm_type(&IrType::Struct(
            "__fishy_range_u16".into(),
            vec![IrType::U16, IrType::U16],
        ))
        .into_struct_type();
    assert_eq!(r.get_field_types(), vec![c.i16_type().into(); 2]);
    assert_eq!(e.target_machine.get_target_data().get_abi_size(&r), 4);
}

#[test]
fn mixed_field_layout_uses_target_padding() {
    let c = Context::create();
    let e = LlvmEmitter::new(&c, "mixed-layout");
    let ty = e
        .get_llvm_type(&IrType::Struct(
            "Mixed".into(),
            vec![IrType::U8, IrType::U64, IrType::U16, IrType::Bool],
        ))
        .into_struct_type();
    let td = e.target_machine.get_target_data();
    assert!(!ty.is_packed());
    assert_eq!(
        ty.get_field_types(),
        vec![
            c.i8_type().into(),
            c.i64_type().into(),
            c.i16_type().into(),
            c.bool_type().into()
        ]
    );
    let wide = td.get_abi_alignment(&c.i64_type()) as u64;
    assert_eq!(td.offset_of_element(&ty, 1), Some(wide));
    assert_eq!(td.offset_of_element(&ty, 2), Some(wide + 8));
    assert_eq!(td.get_abi_size(&ty) % wide, 0);
    assert!(td.get_abi_size(&ty) > 12);
}

#[test]
fn sizeof_instructions_use_the_storage_representation() {
    let c = Context::create();
    let mut e = LlvmEmitter::new(&c, "sizeof");
    let types = [
        IrType::U8,
        IrType::U16,
        IrType::U32,
        IrType::U64,
        IrType::F16,
        IrType::F32,
        IrType::F64,
        IrType::Bool,
        IrType::Ptr(Box::new(IrType::U8)),
        IrType::Array(4, Box::new(IrType::U16)),
        IrType::Struct("__fishy_range_u16".into(), vec![IrType::U16, IrType::U16]),
        IrType::Struct("Mixed".into(), vec![IrType::U8, IrType::U64, IrType::U16]),
    ];
    let functions = types
        .iter()
        .enumerate()
        .map(|(n, ty)| {
            function(
                &format!("size{n}"),
                vec![],
                IrType::U64,
                vec![
                    Instruction::SizeOf {
                        dest: VReg(0),
                        ty: ty.clone(),
                    },
                    Instruction::Ret {
                        value: Some(VReg(0)),
                    },
                ],
            )
        })
        .collect();
    compile(&mut e, &module(functions));
    for (n, ty) in types.iter().enumerate() {
        let f = e.module.get_function(&format!("size{n}")).unwrap();
        let ret = f
            .get_first_basic_block()
            .unwrap()
            .get_last_instruction()
            .unwrap();
        let value = ret
            .get_operand(0)
            .unwrap()
            .left()
            .unwrap()
            .into_int_value()
            .get_zero_extended_constant()
            .unwrap();
        assert_eq!(
            value,
            e.target_machine
                .get_target_data()
                .get_abi_size(&e.get_llvm_type(ty))
        );
    }
}

#[test]
fn abi_signatures_preserve_narrow_float_and_inline_types() {
    let c = Context::create();
    let mut e = LlvmEmitter::new(&c, "signatures");
    let types = [
        IrType::U8,
        IrType::I16,
        IrType::U32,
        IrType::I64,
        IrType::F16,
        IrType::F32,
        IrType::F64,
        IrType::Bool,
        IrType::Struct("__fishy_range_u16".into(), vec![IrType::U16, IrType::U16]),
    ];
    let functions = types
        .iter()
        .enumerate()
        .map(|(n, ty)| {
            function(
                &format!("identity{n}"),
                vec![(VReg(0), ty.clone())],
                ty.clone(),
                vec![Instruction::Ret {
                    value: Some(VReg(0)),
                }],
            )
        })
        .collect();
    compile(&mut e, &module(functions));
    for (n, ty) in types.iter().enumerate() {
        let sig = e
            .module
            .get_function(&format!("identity{n}"))
            .unwrap()
            .get_type();
        assert_eq!(sig.get_return_type(), Some(e.get_llvm_type(ty)));
        assert_eq!(sig.get_param_types(), vec![e.get_llvm_type(ty).into()]);
    }
}

#[test]
fn backend_accepts_dominating_definition_in_a_later_listed_block() {
    let c = Context::create();
    let mut e = LlvmEmitter::new(&c, "block-order");
    let mut f = function("select", vec![], IrType::I64, vec![]);
    f.blocks = vec![
        BasicBlock {
            name: "test".into(),
            id: BlockId(0),
            instructions: vec![Instruction::Br { target: BlockId(2) }],
        },
        BasicBlock {
            name: "test".into(),
            id: BlockId(1),
            instructions: vec![Instruction::Ret {
                value: Some(VReg(8)),
            }],
        },
        BasicBlock {
            name: "test".into(),
            id: BlockId(2),
            instructions: vec![
                Instruction::ConstInt {
                    dest: VReg(8),
                    value: 7,
                },
                Instruction::Br { target: BlockId(1) },
            ],
        },
    ];
    compile(&mut e, &module(vec![f]));
}

#[test]
fn equality_uses_the_declared_unsigned_operand_view() {
    let c = Context::create();
    let mut e = LlvmEmitter::new(&c, "equality");
    let f = function(
        "equals",
        vec![(VReg(0), IrType::U8)],
        IrType::Bool,
        vec![
            Instruction::ConstInt {
                dest: VReg(1),
                value: 255,
            },
            Instruction::CmpEq {
                dest: VReg(2),
                left: VReg(0),
                right: VReg(1),
            },
            Instruction::Ret {
                value: Some(VReg(2)),
            },
        ],
    );
    compile(&mut e, &module(vec![f]));
    let f = e.module.get_function("equals").unwrap();
    let mut inst = f.get_first_basic_block().unwrap().get_first_instruction();
    let mut ext = None;
    while let Some(i) = inst {
        if matches!(
            i.get_opcode(),
            InstructionOpcode::SExt | InstructionOpcode::ZExt
        ) {
            ext = Some(i.get_opcode());
            break;
        }
        inst = i.get_next_instruction();
    }
    assert_eq!(ext, Some(InstructionOpcode::ZExt));
    assert!(jit_bool(&e, "equals", 255));
    assert!(!jit_bool(&e, "equals", 254));
    e.module
        .run_passes(
            "default<O2>",
            &e.target_machine,
            inkwell::passes::PassBuilderOptions::create(),
        )
        .unwrap();
    e.module.verify().unwrap();
    assert!(jit_bool(&e, "equals", 255));
}

fn jit_bool(emitter: &LlvmEmitter<'_>, name: &str, value: u8) -> bool {
    let copy = emitter.module.clone();
    let engine = copy
        .create_jit_execution_engine(inkwell::OptimizationLevel::None)
        .unwrap();
    unsafe {
        engine
            .get_function::<unsafe extern "C" fn(u8) -> bool>(name)
            .unwrap()
            .call(value)
    }
}

#[test]
fn every_narrow_integer_boundary_enters_word_registers() {
    let c = Context::create();
    let mut e = LlvmEmitter::new(&c, "word-boundaries");
    let byte = IrType::U8;
    let ptr = IrType::Ptr(Box::new(byte.clone()));
    let f = function("external_byte", vec![], byte.clone(), vec![]);
    let target = function(
        "callback",
        vec![(VReg(0), ptr.clone())],
        byte.clone(),
        vec![
            Instruction::ConstInt {
                dest: VReg(1),
                value: 255,
            },
            Instruction::Ret {
                value: Some(VReg(1)),
            },
        ],
    );
    let main = function(
        "check",
        vec![
            (VReg(0), ptr.clone()),
            (VReg(1), IrType::Closure(vec![], Box::new(byte.clone()))),
            (VReg(2), IrType::FatPtr),
        ],
        IrType::Void,
        vec![
            Instruction::ConstInt {
                dest: VReg(3),
                value: 255,
            },
            Instruction::Cast {
                dest: VReg(4),
                value: VReg(3),
                source_ty: IrType::I64,
                target_ty: byte.clone(),
            },
            Instruction::Load {
                dest: VReg(5),
                src_ptr: VReg(0),
                ty: byte.clone(),
            },
            Instruction::Call {
                dest: VReg(6),
                func_name: "external_byte".into(),
                args: vec![],
            },
            Instruction::LoadFnPtr {
                dest: VReg(7),
                fn_name: "callback".into(),
            },
            Instruction::IndirectCall {
                dest: VReg(8),
                fn_ptr: VReg(7),
                args: vec![VReg(0)],
                arg_types: vec![ptr],
                ret_type: byte.clone(),
            },
            Instruction::CallClosure {
                dest: VReg(9),
                closure_ptr: VReg(1),
                args: vec![],
                arg_types: vec![],
                ret_type: byte.clone(),
            },
            Instruction::DynamicCall {
                dest: VReg(10),
                fat_ptr: VReg(2),
                vtable_index: 0,
                args: vec![],
                arg_types: vec![],
                ret_type: byte,
            },
            Instruction::Ret { value: None },
        ],
    );
    compile(&mut e, &module(vec![f, target, main]));
    for reg in [VReg(4), VReg(5), VReg(6), VReg(8), VReg(9), VReg(10)] {
        assert_eq!(
            e.registers[&reg]
                .into_int_value()
                .get_type()
                .get_bit_width(),
            64,
            "{reg:?}"
        );
    }
    assert_eq!(
        e.module
            .get_function("external_byte")
            .unwrap()
            .get_type()
            .get_return_type(),
        Some(c.i8_type().into())
    );
}

#[test]
fn external_function_addresses_use_the_resolved_link_identity() {
    let c = Context::create();
    let mut e = LlvmEmitter::new(&c, "external-identity");
    let pointer = IrType::Ptr(Box::new(IrType::Any));
    let f = function("module_native", vec![], IrType::U8, vec![]);
    let address = function(
        "address",
        vec![],
        pointer,
        vec![
            Instruction::LoadFnPtr {
                dest: VReg(0),
                fn_name: "module_native".into(),
            },
            Instruction::Ret {
                value: Some(VReg(0)),
            },
        ],
    );
    let ir = module(vec![f, address]);
    ir_verify::verify(&ir).unwrap();
    e.compile(
        &ir,
        &HashMap::from([("module_native".into(), "native_byte".into())]),
    )
    .unwrap();
    assert!(e.module.get_function("native_byte").is_some());
    assert!(e.module.get_function("module_native").is_none());
    e.module.verify().unwrap();
}

#[test]
fn enum_allocation_reserves_the_largest_typed_payload() {
    let c = Context::create();
    let mut e = LlvmEmitter::new(&c, "enum-capacity");
    let pointer = IrType::Ptr(Box::new(IrType::U8));
    let f = function(
        "construct",
        vec![],
        IrType::Void,
        vec![
            Instruction::AllocStruct {
                dest: VReg(0),
                class_name: "E#Small".into(),
            },
            Instruction::Release { ptr: VReg(0) },
            Instruction::Ret { value: None },
        ],
    );
    let mut ir = module(vec![
        function(
            "malloc",
            vec![(VReg(0), IrType::U64)],
            pointer.clone(),
            vec![],
        ),
        function("panic", vec![(VReg(0), pointer)], IrType::Void, vec![]),
        f,
    ]);
    ir.structs
        .insert("E#Small".into(), vec![IrType::I64, IrType::U8]);
    ir.structs.insert(
        "E#Large".into(),
        vec![
            IrType::I64,
            IrType::U8,
            IrType::U64,
            IrType::U16,
            IrType::F64,
        ],
    );
    compile(&mut e, &ir);
    let td = e.target_machine.get_target_data();
    let layout = e.get_llvm_type(&IrType::Struct("E#Large".into(), vec![]));
    let expected = td.get_abi_size(&layout) + 3 * td.get_abi_size(&c.i64_type());
    let f = e.module.get_function("construct").unwrap();
    let mut actual = None;
    for b in f.get_basic_blocks() {
        let mut inst = b.get_first_instruction();
        while let Some(i) = inst {
            if i.get_opcode() == InstructionOpcode::Call {
                let callee = i
                    .get_operand(i.get_num_operands() - 1)
                    .unwrap()
                    .left()
                    .unwrap()
                    .into_pointer_value();
                if callee.get_name().to_bytes() == b"malloc" {
                    actual = Some(
                        i.get_operand(0)
                            .unwrap()
                            .left()
                            .unwrap()
                            .into_int_value()
                            .get_zero_extended_constant()
                            .unwrap(),
                    );
                }
            }
            inst = i.get_next_instruction();
        }
    }
    assert_eq!(actual, Some(expected));
    assert_eq!(
        (3 * td.get_abi_size(&c.i64_type())) % td.get_abi_alignment(&layout) as u64,
        0
    );
}

#[test]
fn closure_code_addresses_share_the_resolved_link_identity() {
    let c = Context::create();
    let mut e = LlvmEmitter::new(&c, "external-closure");
    let pointer = IrType::Ptr(Box::new(IrType::U8));
    let closure = IrType::Closure(vec![IrType::U8], Box::new(IrType::U16));
    let callback = function(
        "module_callback",
        vec![(VReg(0), pointer.clone()), (VReg(1), IrType::U8)],
        IrType::U16,
        vec![],
    );
    let make = function(
        "make",
        vec![(VReg(0), pointer.clone())],
        closure,
        vec![
            Instruction::MakeClosure {
                dest: VReg(1),
                fn_name: "module_callback".into(),
                env_ptr: VReg(0),
            },
            Instruction::Ret {
                value: Some(VReg(1)),
            },
        ],
    );
    let ir = module(vec![
        function(
            "malloc",
            vec![(VReg(0), IrType::U64)],
            pointer.clone(),
            vec![],
        ),
        function("panic", vec![(VReg(0), pointer)], IrType::Void, vec![]),
        callback,
        make,
    ]);
    ir_verify::verify(&ir).unwrap();
    e.compile(
        &ir,
        &HashMap::from([("module_callback".into(), "native_callback".into())]),
    )
    .unwrap();
    e.module.verify().unwrap();
    let sig = e.module.get_function("native_callback").unwrap().get_type();
    assert_eq!(sig.get_param_types().len(), 2);
    assert_eq!(sig.get_param_types()[1], c.i8_type().into());
    assert_eq!(sig.get_return_type(), Some(c.i16_type().into()));
}

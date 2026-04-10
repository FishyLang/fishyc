use std::{ collections::HashMap, fmt };

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VReg(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub usize);

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum IrType {
    Void,

    I8,
    I16,
    I32,
    I64,

    F16,
    F32,
    F64,
    Bool,
    Ptr(Box<IrType>),
    Array(usize, Box<IrType>),
    Struct(String, Vec<IrType>),
    Any,
    FatPtr,
}

impl std::fmt::Display for IrType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IrType::Void => write!(f, "void"),
            IrType::I8 => write!(f, "i8"),
            IrType::I16 => write!(f, "i16"),
            IrType::I32 => write!(f, "i32"),
            IrType::I64 | IrType::Any => write!(f, "i64"),
            IrType::F16 => write!(f, "f16"),
            IrType::F32 => write!(f, "f32"),
            IrType::F64 => write!(f, "f64"),
            IrType::Bool => write!(f, "i1"),
            IrType::Ptr(_) => write!(f, "ptr"),
            IrType::Array(size, inner_ty) => write!(f, "[{} x {}]", size, inner_ty),
            IrType::Struct(name, _) => write!(f, "%{}", name),
            IrType::FatPtr => write!(f, "fat_ptr"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Alloca {
        dest: VReg,
        name: String,
        ty: IrType,
    },

    AllocArray {
        dest: VReg,
        size: VReg,
        ty: IrType,
    },

    GetElementPtr {
        dest: VReg,
        base_ty: IrType,
        base_ptr: VReg,
        indices: Vec<VReg>,
    },

    Load {
        dest: VReg,
        ty: IrType,
        src_ptr: VReg,
    },

    Store {
        ty: IrType,
        value: VReg,
        ptr: VReg,
    },

    ConstInt {
        dest: VReg,
        value: i64,
    },

    ConstFloat {
        dest: VReg,
        value: f64,
        ty: IrType,
    },

    ConstBool {
        dest: VReg,
        value: bool,
    },

    ConstString {
        dest: VReg,
        value: String,
    },

    Add {
        dest: VReg,
        left: VReg,
        right: VReg,
    },

    Sub {
        dest: VReg,
        left: VReg,
        right: VReg,
    },

    Mul {
        dest: VReg,
        left: VReg,
        right: VReg,
    },

    Div {
        dest: VReg,
        left: VReg,
        right: VReg,
    },

    Mod {
        dest: VReg,
        left: VReg,
        right: VReg,
    },

    CmpEq {
        dest: VReg,
        left: VReg,
        right: VReg,
    },

    CmpLt {
        dest: VReg,
        left: VReg,
        right: VReg,
    },

    CmpGt {
        dest: VReg,
        left: VReg,
        right: VReg,
    },

    CmpNeq {
        dest: VReg,
        left: VReg,
        right: VReg,
    },

    CmpLe {
        dest: VReg,
        left: VReg,
        right: VReg,
    },

    CmpGe {
        dest: VReg,
        left: VReg,
        right: VReg,
    },

    Br {
        target: BlockId,
    },

    CondBr {
        cond: VReg,
        if_true: BlockId,
        if_false: BlockId,
    },

    Ret {
        value: Option<VReg>,
    },

    Call {
        dest: VReg,
        func_name: String,
        args: Vec<VReg>,
    },

    AllocStruct {
        dest: VReg,
        class_name: String,
        size: usize,
    },

    Cast {
        dest: VReg,
        value: VReg,
        target_ty: IrType,
    },

    MakeFatPtr {
        dest: VReg,
        data_ptr: VReg,
        vtable_name: String,
    },

    DynamicCall {
        dest: VReg,
        vtable_index: usize,
        fat_ptr: VReg,
        args: Vec<VReg>,
        arg_types: Vec<IrType>,
        ret_type: IrType,
    },

    LoadFnPtr {
        dest: VReg,
        fn_name: String,
    },

    IndirectCall {
        dest: VReg,
        fn_ptr: VReg,
        args: Vec<VReg>,
        arg_types: Vec<IrType>,
        ret_type: IrType,
    },

    MakeClosure {
        dest: VReg,
        fn_name: String,
        env_ptr: VReg,
    },

    CallClosure {
        dest: VReg,
        closure_ptr: VReg,
        args: Vec<VReg>,
        arg_types: Vec<IrType>,
        ret_type: IrType,
    },

    /// retain a block of memory
    Retain {
        ptr: VReg,
    },

    /// release a block of memory
    /// in fishy, we use ARC instead of a borrow-checker model to prevent memory leaks, specially in lambdas and closures
    /// we insert an invisible header in objects that are heap-allocated to track the number of times that that object was called
    /// our ir silently inserts these instructions at the end of blocks to free objects that are no longer being used, reducing
    /// memory usage and preventing memory leaks
    Release {
        ptr: VReg,
    },

    Unreachable,
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: BlockId,
    pub name: String,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone)]
pub struct FunctionIr {
    pub name: String,
    pub ret_type: IrType,
    pub args: Vec<(VReg, IrType)>,
    pub blocks: Vec<BasicBlock>,
    pub is_variadic: bool,
}

#[derive(Debug)]
pub struct ModuleIr {
    pub name: String,
    pub functions: Vec<FunctionIr>,
    pub vtables: HashMap<String, Vec<String>>,
}

impl fmt::Display for VReg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.0)
    }
}

impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "label %bb{}", self.0)
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Alloca { dest, name, ty } =>
                write!(f, "  {} = alloca {}, align 8 ; var {}", dest, ty, name),

            Instruction::AllocArray { dest, size, ty } =>
                write!(f, "  {} = alloc_array {} size {}", dest, ty, size),

            Instruction::GetElementPtr { dest, base_ty, base_ptr, indices } => {
                write!(f, "  {} = getelementptr {}, ptr {}", dest, base_ty, base_ptr)?;
                for idx in indices {
                    write!(f, ", i64 {}", idx)?;
                }
                Ok(())
            }

            Instruction::Load { dest, ty, src_ptr } =>
                write!(f, "  {} = load {}, ptr {}, align 8", dest, ty, src_ptr),

            Instruction::Store { ty, value, ptr } =>
                write!(f, "  store {} {}, ptr {}, align 8", ty, value, ptr),

            Instruction::ConstInt { dest, value } => write!(f, "  {} = const int {}", dest, value),

            Instruction::ConstFloat { dest, value, ty } =>
                write!(f, "  {} = const {} {}", dest, ty, value),

            Instruction::ConstBool { dest, value } =>
                write!(f, "  {} = const bool {}", dest, value),

            Instruction::ConstString { dest, value } =>
                write!(f, "  {} = const string \"{}\"", dest, value.replace("\n", "\\0A\\00")),

            Instruction::Add { dest, left, right } =>
                write!(f, "  {} = add {}, {}", dest, left, right),

            Instruction::Sub { dest, left, right } =>
                write!(f, "  {} = sub {}, {}", dest, left, right),

            Instruction::Mul { dest, left, right } =>
                write!(f, "  {} = mul {}, {}", dest, left, right),

            Instruction::Div { dest, left, right } =>
                write!(f, "  {} = div {}, {}", dest, left, right),

            Instruction::Mod { dest, left, right } =>
                write!(f, "  {} = mod {}, {}", dest, left, right),

            Instruction::CmpEq { dest, left, right } =>
                write!(f, "  {} = cmp eq {}, {}", dest, left, right),

            Instruction::CmpLt { dest, left, right } =>
                write!(f, "  {} = cmp lt {}, {}", dest, left, right),

            Instruction::CmpGt { dest, left, right } =>
                write!(f, "  {} = cmp gt {}, {}", dest, left, right),

            Instruction::CmpNeq { dest, left, right } =>
                write!(f, "  {} = cmp neq {}, {}", dest, left, right),

            Instruction::CmpLe { dest, left, right } =>
                write!(f, "  {} = cmp le {}, {}", dest, left, right),

            Instruction::CmpGe { dest, left, right } =>
                write!(f, "  {} = cmp ge {}, {}", dest, left, right),

            Instruction::Br { target } => write!(f, "  br {}", target),

            Instruction::CondBr { cond, if_true, if_false } =>
                write!(f, "  br cond {}, {}, {}", cond, if_true, if_false),

            Instruction::Ret { value } => {
                if let Some(v) = value { write!(f, "  ret {}", v) } else { write!(f, "  ret void") }
            }

            Instruction::Call { dest, func_name, args } => {
                let args_str: Vec<String> = args
                    .iter()
                    .map(|a| a.to_string())
                    .collect();
                write!(f, "  {} = call @{}({})", dest, func_name, args_str.join(", "))
            }

            Instruction::AllocStruct { dest, class_name, size } =>
                write!(f, "  {} = alloca {}, size {}, align 8", dest, class_name, size),

            Instruction::Cast { dest, value, target_ty } =>
                write!(f, "  {} = cast {} as {}", dest, value, target_ty),

            Instruction::MakeFatPtr { dest, data_ptr, vtable_name } =>
                write!(f, "  {} = make_fat_ptr {}, @{}", dest, data_ptr, vtable_name),

            Instruction::DynamicCall { dest, vtable_index, fat_ptr, args, .. } => {
                let args_str: Vec<String> = args
                    .iter()
                    .map(|a| a.to_string())
                    .collect();
                write!(
                    f,
                    "  {} = dyn_call vtable[{}] {}({})",
                    dest,
                    vtable_index,
                    fat_ptr,
                    args_str.join(", ")
                )
            }

            Instruction::LoadFnPtr { dest, fn_name } =>
                write!(f, "  {} = load_fn_ptr {}", dest, fn_name),

            Instruction::IndirectCall { dest, fn_ptr, args, .. } =>
                write!(f, "  {} = indirect_call {}({:?})", dest, fn_ptr, args),

            Instruction::MakeClosure { dest, fn_name, env_ptr } =>
                write!(f, "  {} = make_closure {}, env: {}", dest, fn_name, env_ptr),

            Instruction::CallClosure { dest, closure_ptr, args, .. } =>
                write!(f, "  {} = call_closure {}({:?})", dest, closure_ptr, args),

            Instruction::Retain { ptr } => write!(f, "  retain {}", ptr),

            Instruction::Release { ptr } => write!(f, "  release {}", ptr),

            Instruction::Unreachable => write!(f, "  unreachable"),
        }
    }
}

impl fmt::Display for BasicBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}: ; {}", self.id, self.name)?;
        for inst in &self.instructions {
            writeln!(f, "{}", inst)?;
        }
        Ok(())
    }
}

impl fmt::Display for FunctionIr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let args_str: Vec<String> = self.args
            .iter()
            .map(|(v, _)| v.to_string())
            .collect();
        writeln!(f, "define @{}({}) {{", self.name, args_str.join(", "))?;
        for block in &self.blocks {
            write!(f, "{}", block)?;
        }
        writeln!(f, "}}")
    }
}

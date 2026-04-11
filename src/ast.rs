use crate::token::{Literal, Token};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Bool,
    String,
    Void,

    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F16,
    F32,
    F64,

    Pointer(Box<Type>),
    Reference(Box<Type>),
    MutReference(Box<Type>),
    Slice(Box<Type>),
    Array(usize, Box<Type>),
    Custom(String),
    Tuple(Vec<Type>),
    Function(Vec<Type>, Box<Type>, bool),
    Generic(String, Vec<Type>),
    Union(Vec<Type>),

    // inference variable for Hindley-Milner type inference
    TypeVar(usize),
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Bool => write!(f, "bool"),
            Type::String => write!(f, "string"),
            Type::Void => write!(f, "void"),

            Type::U8 => write!(f, "u8"),
            Type::U16 => write!(f, "u16"),
            Type::U32 => write!(f, "u32"),
            Type::U64 => write!(f, "u64"),

            Type::I8 => write!(f, "i8"),
            Type::I16 => write!(f, "i16"),
            Type::I32 => write!(f, "i32"),
            Type::I64 => write!(f, "i64"),

            Type::F16 => write!(f, "f16"),
            Type::F32 => write!(f, "f32"),
            Type::F64 => write!(f, "f64"),

            Type::Pointer(inner) => write!(f, "^{}", inner),
            Type::Reference(inner) => write!(f, "&{}", inner),
            Type::MutReference(inner) => write!(f, "&mut {}", inner),
            Type::Slice(inner) => write!(f, "[{}; ?]", inner),
            Type::Array(size, ty) => write!(f, "[{}; {}]", size, ty),
            Type::Custom(name) => write!(f, "{}", name),

            Type::Tuple(items) => {
                write!(f, "(")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, ")")
            }

            Type::Function(params, ret_ty, _) => {
                write!(f, "fn(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", ret_ty)
            }

            Type::Generic(name, args) => {
                write!(f, "{}<", name)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ">")
            }

            Type::Union(items) => {
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "{}", item)?;
                }
                Ok(())
            }

            Type::TypeVar(id) => write!(f, "?{}", id),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MatchCase {
    pub pattern: Expr,
    pub guard: Option<Expr>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Variable(Token),
    This(Token),

    Grouping(Box<Expr>),

    Unary {
        operator: Token,
        right: Box<Expr>,
    },

    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },

    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },

    Ternary {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
        true_token: Token,
    },

    Assign {
        name: Token,
        value: Box<Expr>,
    },

    Get {
        object: Box<Expr>,
        name: Token,
    },

    Set {
        object: Box<Expr>,
        name: Token,
        value: Box<Expr>,
    },

    Array {
        bracket: Token,
        elements: Vec<Expr>,
    },

    TupleLiteral {
        elements: Vec<Expr>,
        token: Token,
    },

    SubscriptGet {
        indexee: Box<Expr>,
        bracket: Token,
        index: Box<Expr>,
    },

    SubscriptSet {
        indexee: Box<Expr>,
        bracket: Token,
        index: Box<Expr>,
        value: Box<Expr>,
    },

    Spread {
        operator: Token,
        right: Box<Expr>,
    },

    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },

    Lambda {
        params: Vec<FunctionParam>,
        return_type: Option<Type>,
        body: Vec<Stmt>,
        is_async: bool,
    },

    New {
        keyword: Token,
        class_name: Token,
        type_args: Vec<Type>,
        arguments: Vec<Expr>,
        paren: Token,
    },

    Match {
        keyword: Token,
        value: Box<Expr>,
        cases: Vec<MatchCase>,
    },

    Typeof(Token),

    Lazy {
        expr: Option<Box<Expr>>,
        statements: Option<Vec<Stmt>>,
    },

    Await {
        keyword: Token,
        value: Box<Expr>,
    },

    WildcardPattern(Token),

    UnionPattern {
        case_name: Token,
        bindings: Vec<Token>,
    },

    ListPattern {
        elements: Vec<Expr>,
        rest: Option<Box<Expr>>,
    },

    ObjectPattern {
        properties: Vec<(Token, Expr)>,
        rest: Option<Box<Expr>>,
    },

    AddressOf {
        operator: crate::token::Token,
        operand: Box<Expr>,
    },

    Dereference {
        operator: crate::token::Token,
        operand: Box<Expr>,
    },

    DereferenceSet {
        operator: crate::token::Token,
        ptr: Box<Expr>,
        value: Box<Expr>,
    },

    Cast {
        value: Box<Expr>,
        operator: Token,
        target_type: Type,
    },
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Literal(_) => write!(f, "literal"),
            Expr::Variable(_) => write!(f, "variable"),
            Expr::This(_) => write!(f, "self"),
            Expr::Grouping(_) => write!(f, "grouping"),
            Expr::Unary { .. } => write!(f, "unary"),
            Expr::Binary { .. } => write!(f, "binary"),
            Expr::Logical { .. } => write!(f, "logical"),
            Expr::Ternary { .. } => write!(f, "ternary"),
            Expr::Assign { .. } => write!(f, "assign"),
            Expr::Get { .. } => write!(f, "get"),
            Expr::Set { .. } => write!(f, "set"),
            Expr::Array { .. } => write!(f, "array"),
            Expr::TupleLiteral { .. } => write!(f, "tuple literal"),
            Expr::SubscriptGet { .. } => write!(f, "subscript get"),
            Expr::SubscriptSet { .. } => write!(f, "subscript set"),
            Expr::Spread { .. } => write!(f, "spread"),
            Expr::Call { .. } => write!(f, "call"),
            Expr::Lambda { .. } => write!(f, "lambda"),
            Expr::New { .. } => write!(f, "new"),
            Expr::Match { .. } => write!(f, "match"),
            Expr::Typeof(_) => write!(f, "typeof"),
            Expr::Lazy { .. } => write!(f, "lazy"),
            Expr::Await { .. } => write!(f, "await"),
            Expr::WildcardPattern(_) => write!(f, "wildcard pattern"),
            Expr::UnionPattern { .. } => write!(f, "union pattern"),
            Expr::ListPattern { .. } => write!(f, "list pattern"),
            Expr::ObjectPattern { .. } => write!(f, "object pattern"),
            Expr::AddressOf { .. } => write!(f, "addressof"),
            Expr::Dereference { .. } => write!(f, "dereference"),
            Expr::DereferenceSet { .. } => write!(f, "dereference set"),
            Expr::Cast { .. } => write!(f, "cast"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GenericParam {
    pub name: Token,
    pub constraints: Vec<Type>,
}

#[derive(Debug, Clone)]
pub struct FunctionParam {
    pub name: Token,
    pub type_annotation: Option<Type>,
    pub default_value: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct EnumCase {
    pub name: Token,
    pub parameters: Vec<(Token, Option<Type>)>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression(Expr),
    Block(Vec<Stmt>),

    Alias {
        name: Token,
        target: Type,
    },

    ExternFunction {
        name: Token,
        params: Vec<FunctionParam>,
        return_type: Option<Type>,
        is_variadic: bool,
    },

    Var {
        name: Token,
        type_annotation: Option<Type>,
        initializer: Option<Expr>,
    },

    ArrayDestructuring {
        keyword: Token,
        bindings: Vec<Token>,
        initializer: Expr,
    },

    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
        if_token: Token,
    },

    While {
        condition: Expr,
        body: Box<Stmt>,
        while_token: Token,
    },

    For {
        keyword: Token,
        initializer: Option<Box<Stmt>>,
        condition: Option<Expr>,
        increment: Option<Expr>,
        body: Vec<Stmt>,
    },

    ForIn {
        keyword: Token,
        key: Token,
        value: Option<Token>,
        iterable: Expr,
        body: Vec<Stmt>,
    },

    Return {
        keyword: Token,
        value: Option<Expr>,
    },

    Throw {
        keyword: Token,
        thrown: Expr,
    },

    TryCatch {
        try_body: Vec<Stmt>,
        catch_body: Vec<Stmt>,
        exception: Token,
        exception_type: Option<Type>,
    },

    Function {
        name: Token,
        type_params: Vec<GenericParam>,
        params: Vec<FunctionParam>,
        return_type: Option<Type>,
        body: Option<Vec<Stmt>>,
        is_abstract: bool,
        is_async: bool,
        is_public: bool,
    },

    Struct {
        name: Token,
        type_params: Vec<GenericParam>,
        fields: Vec<(Token, Type, bool)>,
    },

    Trait {
        name: Token,
        type_params: Vec<GenericParam>,
        traits: Vec<Expr>,
        methods: Vec<Stmt>,
    },

    Impl {
        keyword: Token,
        type_params: Vec<GenericParam>,
        trait_name: Option<Type>,
        target_type: Type,
        methods: Vec<Stmt>,
    },

    Enum {
        name: Token,
        type_params: Vec<GenericParam>,
        cases: Vec<EnumCase>,
        is_union: bool,
    },

    Using {
        keyword: Token,
        names: Vec<Token>,
        source: Expr,
    },
}

impl std::fmt::Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stmt::Expression(_) => write!(f, "expression"),
            Stmt::Block(_) => write!(f, "block"),
            Stmt::Alias { .. } => write!(f, "alias"),
            Stmt::ExternFunction { .. } => write!(f, "extern function"),
            Stmt::Var { .. } => write!(f, "var"),
            Stmt::ArrayDestructuring { .. } => write!(f, "array destructuring"),
            Stmt::If { .. } => write!(f, "if"),
            Stmt::While { .. } => write!(f, "while"),
            Stmt::For { .. } => write!(f, "for"),
            Stmt::ForIn { .. } => write!(f, "for in"),
            Stmt::Return { .. } => write!(f, "return"),
            Stmt::Throw { .. } => write!(f, "throw"),
            Stmt::TryCatch { .. } => write!(f, "try catch"),
            Stmt::Function { .. } => write!(f, "function"),
            Stmt::Struct { .. } => write!(f, "struct"),
            Stmt::Trait { .. } => write!(f, "trait"),
            Stmt::Impl { .. } => write!(f, "impl"),
            Stmt::Enum { .. } => write!(f, "enum"),
            Stmt::Using { .. } => write!(f, "using"),
        }
    }
}

use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenType {
    // single characters
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Colon,
    Comma,
    Semicolon,
    Slash,
    Arrow,
    Percent,
    Question,
    Caret,

    // one or more characters
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Dot,
    DotDot,
    Star,
    Exponentiation,
    Plus,
    PlusPlus,
    Minus,
    MinusMinus,
    Spread,
    QuestionQuestion,
    BitwiseAnd,
    BitwiseOr,
    LogicalAnd,
    LogicalOr,
    MinusEqual,
    PlusEqual,

    // literals
    Identifier,
    String,
    Number,
    StringPart,
    InterpolationStart,

    // keywords
    Struct,
    Else,
    False,
    Fn,
    For,
    If,
    Null,
    Return,
    This,
    True,
    Let,
    While,
    In,
    With,
    Trait,
    Throw,
    Enum,
    Lazy,
    Using,
    Catch,
    Finally,
    From,
    Try,
    Match,
    Impl,
    Async,
    Await,
    Alias,
    Extern,
    Mut,
    Const,
    As,
    Pub,

    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(String),
    Number(f64),
    Integer(i128),
    Bool(bool),
    None,
}

impl Eq for Literal {}

impl std::hash::Hash for Literal {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);

        match self {
            Literal::Number(n) => n.to_bits().hash(state),
            Literal::Integer(n) => n.hash(state),
            Literal::String(s) => s.hash(state),
            Literal::Bool(b) => b.hash(state),
            Literal::None => {}
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Literal,
    pub line: usize,
    pub column: usize,
    pub file_path: Arc<String>,
}

impl Token {
    pub fn new(
        token_type: TokenType,
        lexeme: String,
        literal: Literal,
        line: usize,
        column: usize,
        file_path: Arc<String>
    ) -> Self {
        Self {
            token_type,
            lexeme,
            literal,
            line,
            column,
            file_path,
        }
    }

    pub fn synthetic(token_type: TokenType, lexeme: &str) -> Self {
        Self {
            token_type,
            lexeme: lexeme.to_string(),
            literal: Literal::None,
            line: 0,
            column: 0,
            file_path: Arc::new("<internal>".to_string()),
        }
    }
}

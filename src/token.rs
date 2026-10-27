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

    Alias,
    Extern,
    Mut,
    Const,
    As,
    Pub,

    Eof,
}

#[derive(Debug, Clone)]
pub enum Literal {
    String(String),
    Number(f64),
    Integer(i128),
    Bool(bool),
    None,
}

impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(a), Self::Number(b)) => a.to_bits() == b.to_bits(),
            (Self::Integer(a), Self::Integer(b)) => a == b,
            (Self::String(a), Self::String(b)) => a == b,
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::None, Self::None) => true,
            _ => false,
        }
    }
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
    pub id: usize,
    /// Origin authentication for compiler-injected storage contracts and their
    /// private fields. This never disables ownership or borrow checking.
    pub trusted_prelude: bool,
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Literal,
    pub line: usize,
    pub column: usize,
    pub file_path: Arc<String>,
}

impl Token {
    pub fn fresh_id() -> usize {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
    pub fn new(
        token_type: TokenType,
        lexeme: String,
        literal: Literal,
        line: usize,
        column: usize,
        file_path: Arc<String>
    ) -> Self {
        Self {
            id: Self::fresh_id(),
            trusted_prelude: false,
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
            id: Self::fresh_id(),
            trusted_prelude: false,
            token_type,
            lexeme: lexeme.to_string(),
            literal: Literal::None,
            line: 0,
            column: 0,
            file_path: Arc::new("<internal>".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    #[test]
    fn floating_key_equality_is_bitwise_and_reflexive() {
        let nan = Literal::Number(f64::NAN);
        assert_eq!(nan, nan.clone());
        let keys = HashSet::from([Literal::Number(0.0), Literal::Number(-0.0), nan.clone(), nan]);
        assert_eq!(keys.len(), 3);
    }
}

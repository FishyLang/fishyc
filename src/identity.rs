//! Semantic identities are independent of AST addresses and physical storage.
use crate::token::Token;

macro_rules! identity {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(pub usize);
    };
}
identity!(BindingId);
identity!(ExprId);
identity!(TypeId);
identity!(ScopeId);
identity!(PlaceId);

impl ExprId {
    pub fn fresh() -> Self {
        Self(Token::fresh_id())
    }
}

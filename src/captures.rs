//! Exact lexical free-binding capture plans for ordinary closures.
use crate::{ ast::Type, token::Token };
use std::collections::{ BTreeMap, HashMap };
#[derive(Debug, Clone)]
pub struct CaptureBinding {
    pub declaration: Token,
    pub ty: Type,
}
pub type CapturePlans = HashMap<usize, Vec<CaptureBinding>>;
#[derive(Debug)]
pub struct CaptureContext {
    pub boundary: usize,
    pub bindings: BTreeMap<usize, CaptureBinding>,
}

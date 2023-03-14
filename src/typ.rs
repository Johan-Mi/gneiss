use crate::{ast::SyntaxError, text::node_text};
use ropey::Rope;
use tree_sitter::Node;

#[derive(Debug)]
pub enum Type {
    Unit,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
}

impl Type {
    pub fn parse(node: Node, text: &Rope) -> Result<Self, SyntaxError> {
        if node.kind() != "primitive_type" {
            return Err(SyntaxError);
        }
        match &*node_text(node, text) {
            "unit" => Ok(Self::Unit),
            "i8" => Ok(Self::I8),
            "i16" => Ok(Self::I16),
            "i32" => Ok(Self::I32),
            "i64" => Ok(Self::I64),
            "u8" => Ok(Self::U8),
            "u16" => Ok(Self::U16),
            "u32" => Ok(Self::U32),
            "u64" => Ok(Self::U64),
            _ => Err(SyntaxError),
        }
    }
}

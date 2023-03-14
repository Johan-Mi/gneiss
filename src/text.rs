use ropey::Rope;
use std::borrow::Cow;
use tree_sitter::Node;

pub fn node_text<'a>(node: Node, text: &'a Rope) -> Cow<'a, str> {
    text.byte_slice(node.byte_range()).into()
}

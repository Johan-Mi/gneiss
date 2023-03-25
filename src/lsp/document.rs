use crate::text::PositionEncoding;
use lsp_types::{Diagnostic, DiagnosticSeverity, Range};
use ropey::Rope;
use tree_sitter::{Tree, TreeCursor};

pub struct Document {
    pub text: Rope,
    pub tree: Tree,
    pub ast: crate::ast::File,
    pub diagnostics: Vec<Diagnostic>,
}

impl Document {
    pub fn check_syntax_errors(&mut self, pos_enc: PositionEncoding) {
        let mut cursor = self.tree.walk();
        check_node_for_syntax_errors(
            &mut cursor,
            &mut self.diagnostics,
            &self.text,
            pos_enc,
        );
    }
}

fn check_node_for_syntax_errors(
    cursor: &mut TreeCursor,
    diagnostics: &mut Vec<Diagnostic>,
    text: &Rope,
    pos_enc: PositionEncoding,
) {
    let node = cursor.node();
    if !node.has_error() {
        return;
    }
    if node.is_error() {
        let range = node.byte_range();
        let range = Range {
            start: pos_enc.byte_to_position(text, range.start),
            end: pos_enc.byte_to_position(text, range.end),
        };
        diagnostics.push(Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::ERROR),
            code: None,
            code_description: None,
            source: None,
            message: "syntax error".to_owned(),
            related_information: None,
            tags: None,
            data: None,
        });
        return;
    }
    if !cursor.goto_first_child() {
        return;
    }
    loop {
        check_node_for_syntax_errors(cursor, diagnostics, text, pos_enc);
        if !cursor.goto_next_sibling() {
            cursor.goto_parent();
            return;
        }
    }
}

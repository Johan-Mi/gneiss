use super::LanguageServer;
use lsp_server::{Message, RequestId};
use lsp_types::{Hover, HoverContents, HoverParams, MarkupContent, MarkupKind};

impl LanguageServer {
    pub fn hover(&self, id: RequestId, params: &HoverParams) {
        let doc =
            &self.docs[&params.text_document_position_params.text_document.uri];

        let cursor_byte = self.pos_enc.position_to_byte(
            &doc.text,
            params.text_document_position_params.position,
        );
        let node = doc
            .tree
            .root_node()
            .descendant_for_byte_range(cursor_byte, cursor_byte + 1)
            .unwrap();

        let contents = HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!("Hovering kind `{}`", node.kind()),
        });

        self.connection
            .sender
            .send(Message::Response(lsp_server::Response::new_ok(
                id,
                Hover {
                    contents,
                    range: None,
                },
            )))
            .unwrap();
    }
}

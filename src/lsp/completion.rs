use super::LanguageServer;
use lsp_server::{Message, RequestId};
use lsp_types::{CompletionItem, CompletionParams, CompletionResponse};

impl LanguageServer {
    pub fn complete(&self, id: RequestId, params: &CompletionParams) {
        self.connection
            .sender
            .send(Message::Response(lsp_server::Response::new_ok(
                id,
                CompletionResponse::Array(vec![CompletionItem {
                    label: "foo".to_owned(),
                    ..Default::default()
                }]),
            )))
            .unwrap();
    }
}

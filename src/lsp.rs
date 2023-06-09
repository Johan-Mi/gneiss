mod completion;
mod document;
mod hover;

use crate::text::{byte_to_point, PositionEncoding};
use document::Document;
use lsp_server::{Connection, IoThreads, Message, Notification, Request};
use lsp_types::{
    notification::{
        DidChangeTextDocument, DidOpenTextDocument, Notification as _,
        PublishDiagnostics,
    },
    request::{Completion, HoverRequest, Request as _},
    *,
};
use ropey::Rope;
use std::collections::HashMap;
use tree_sitter::{InputEdit, Parser};

pub struct LanguageServer {
    connection: Connection,
    io_threads: IoThreads,
    parser: Parser,
    pos_enc: PositionEncoding,
    docs: HashMap<Url, Document>,
}

impl LanguageServer {
    pub fn new() -> Self {
        let (connection, io_threads) = Connection::stdio();
        let (id, initialize_params) = connection.initialize_start().unwrap();
        let initialize_params: InitializeParams =
            serde_json::from_value(initialize_params).unwrap();
        let position_encoding = PositionEncoding::from(&initialize_params);
        let server_capabilities = serde_json::to_value(ServerCapabilities {
            position_encoding: Some(position_encoding.into()),
            text_document_sync: Some(TextDocumentSyncKind::INCREMENTAL.into()),
            completion_provider: Some(CompletionOptions::default()),
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            ..Default::default()
        })
        .unwrap();
        connection
            .initialize_finish(
                id,
                serde_json::json!({
                    "capabilities": server_capabilities,
                }),
            )
            .unwrap();
        log::info!("Initialized LSP: {initialize_params:#?}");

        let language = tree_sitter_gneiss::language();
        let mut parser = Parser::new();
        parser.set_language(language).unwrap();

        Self {
            connection,
            io_threads,
            parser,
            pos_enc: position_encoding,
            docs: HashMap::new(),
        }
    }

    pub fn run(mut self) {
        while let Ok(message) = self.connection.receiver.recv() {
            match message {
                Message::Request(request) => {
                    if self.connection.handle_shutdown(&request).unwrap() {
                        break;
                    }
                    self.handle_request(request);
                }
                Message::Response(_) => todo!(),
                Message::Notification(notification) => {
                    self.handle_notification(notification);
                }
            }
        }

        self.io_threads.join().unwrap();
    }

    fn handle_notification(
        &mut self,
        Notification { method, params }: Notification,
    ) {
        match &*method {
            DidOpenTextDocument::METHOD => {
                let params: DidOpenTextDocumentParams =
                    serde_json::from_value(params).unwrap();
                self.open(params.text_document.uri, params.text_document.text);
            }
            DidChangeTextDocument::METHOD => {
                let params: DidChangeTextDocumentParams =
                    serde_json::from_value(params).unwrap();
                self.edit(params.text_document.uri, &params.content_changes);
            }
            _ => log::warn!("Unhandled notification method: {method:?}"),
        }
    }

    fn open(&mut self, uri: Url, text: String) {
        let tree = self.parser.parse(&text, None).unwrap();
        let text = Rope::from(text);
        let ast = crate::ast::File::parse(&tree, &text);
        log::info!("\n{:#?}", ast);

        self.docs.insert(
            uri.clone(),
            Document {
                text,
                tree,
                ast,
                diagnostics: Vec::new(),
            },
        );

        self.update_and_publish_diagnostics(uri);
    }

    fn edit(&mut self, uri: Url, edits: &[TextDocumentContentChangeEvent]) {
        let doc = self.docs.get_mut(&uri).unwrap();

        for edit in edits.iter().rev() {
            let range = edit.range.expect(
                "edits that replace the entire document are not supported",
            );
            let start_byte =
                self.pos_enc.position_to_byte(&doc.text, range.start);
            let old_end_byte =
                self.pos_enc.position_to_byte(&doc.text, range.end);
            let new_end_byte = start_byte + edit.text.len();
            let old_end_position =
                self.pos_enc.position_to_point(&doc.text, range.end);
            let start_char = doc.text.byte_to_char(start_byte);
            doc.text
                .remove(start_char..doc.text.byte_to_char(old_end_byte));
            doc.text.insert(start_char, &edit.text);
            let edit = InputEdit {
                start_byte,
                old_end_byte,
                new_end_byte,
                start_position: self
                    .pos_enc
                    .position_to_point(&doc.text, range.start),
                old_end_position,
                new_end_position: byte_to_point(&doc.text, new_end_byte),
            };
            doc.tree.edit(&edit);
        }

        doc.tree = self
            .parser
            .parse_with(
                &mut |byte_offest, _position| {
                    let (chunk, chunk_start, ..) =
                        doc.text.chunk_at_byte(byte_offest);
                    &chunk.as_bytes()[(byte_offest - chunk_start)..]
                },
                Some(&doc.tree),
            )
            .unwrap();

        doc.ast = crate::ast::File::parse(&doc.tree, &doc.text);

        log::info!("\n{:#?}", doc.ast);

        self.update_and_publish_diagnostics(uri);
    }

    fn handle_request(&self, Request { id, method, params }: Request) {
        match &*method {
            Completion::METHOD => {
                let params = serde_json::from_value(params).unwrap();
                self.complete(id, &params);
            }
            HoverRequest::METHOD => {
                let params = serde_json::from_value(params).unwrap();
                self.hover(id, &params);
            }
            _ => log::warn!("Unhandled request method: {method:?}"),
        }
    }

    fn update_and_publish_diagnostics(&mut self, uri: Url) {
        let doc = self.docs.get_mut(&uri).unwrap();
        doc.diagnostics.clear();

        doc.check_syntax_errors(self.pos_enc);

        self.connection
            .sender
            .send(
                Notification::new(
                    PublishDiagnostics::METHOD.to_owned(),
                    PublishDiagnosticsParams {
                        uri,
                        diagnostics: doc.diagnostics.clone(),
                        version: None,
                    },
                )
                .into(),
            )
            .unwrap();
    }
}

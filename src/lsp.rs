use lsp_server::{Connection, IoThreads, Message, Notification, Request};
use lsp_types::{
    notification::{DidOpenTextDocument, Notification as _},
    *,
};
use ropey::Rope;
use std::collections::HashMap;
use tree_sitter::{Parser, Tree};

pub struct LanguageServer {
    connection: Connection,
    io_threads: IoThreads,
    parser: Parser,
    docs: HashMap<Url, Document>,
}

impl LanguageServer {
    pub fn new() -> Self {
        let (connection, io_threads) = Connection::stdio();
        let (id, initialize_params) = connection.initialize_start().unwrap();
        let initialize_params: InitializeParams =
            serde_json::from_value(initialize_params).unwrap();
        let server_capabilities =
            serde_json::to_value(ServerCapabilities::default()).unwrap();
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
            _ => log::warn!("Unhandled notification method: {method:?}"),
        }
    }

    fn open(&mut self, uri: Url, text: String) {
        let tree = self.parser.parse(&text, None).unwrap();
        let text = Rope::from(text);

        self.docs.insert(uri, Document { text, tree });
    }

    fn handle_request(&self, Request { id, method, params }: Request) {
        match &*method {
            _ => log::warn!("Unhandled request method: {method:?}"),
        }
    }
}

struct Document {
    text: Rope,
    tree: Tree,
}

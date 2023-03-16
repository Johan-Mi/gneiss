use lsp_server::{Connection, IoThreads, Message, Notification, Request};
use lsp_types::*;

pub struct LanguageServer {
    connection: Connection,
    io_threads: IoThreads,
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

        Self {
            connection,
            io_threads,
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
            _ => log::warn!("Unhandled notification method: {method:?}"),
        }
    }

    fn handle_request(&self, Request { id, method, params }: Request) {
        match &*method {
            _ => log::warn!("Unhandled request method: {method:?}"),
        }
    }
}

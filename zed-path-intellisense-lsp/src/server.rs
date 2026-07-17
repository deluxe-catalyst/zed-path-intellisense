use std::path::PathBuf;
use percent_encoding::percent_decode_str;
use crate::completion::{PathCompletionProvider, Position};
use crate::jsonrpc;

pub struct Server {
    completion_provider: PathCompletionProvider,
    workspace_root: Option<PathBuf>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            completion_provider: PathCompletionProvider::new(),
            workspace_root: None,
        }
    }

    pub fn run(&mut self) {
        loop {
            let msg = match jsonrpc::read_message() {
                Some(m) => m,
                None => break,
            };

            match msg {
                jsonrpc::Message::Request(req) => self.handle_request(req),
                jsonrpc::Message::Notification(not) => self.handle_notification(not),
                jsonrpc::Message::Response(_) => {}
            }
        }
    }

    fn handle_request(&mut self, req: jsonrpc::Request) {
        match req.method.as_str() {
            "initialize" => self.handle_initialize(req),
            "textDocument/completion" => self.handle_completion(req),
            "shutdown" => {
                jsonrpc::send_message(&jsonrpc::ResponseResult {
                    jsonrpc: "2.0".to_string(),
                    id: req.id,
                    result: Some(serde_json::json!(null)),
                    error: None,
                });
            }
            _ => {
                jsonrpc::send_message(&jsonrpc::ResponseResult {
                    jsonrpc: "2.0".to_string(),
                    id: req.id,
                    result: None,
                    error: Some(jsonrpc::ResponseError {
                        code: -32601,
                        message: format!("Method not found: {}", req.method),
                    }),
                });
            }
        }
    }

    fn handle_notification(&mut self, not: jsonrpc::Notification) {
        match not.method.as_str() {
            "initialized" => {}
            "textDocument/didOpen" => {
                let params = not.params;
                let text_document = params["textDocument"].clone();
                let uri = text_document["uri"].as_str().unwrap_or("").to_string();
                let text = text_document["text"].as_str().unwrap_or("").to_string();
                self.completion_provider.update_document(uri_to_string(&uri), text);
            }
            "textDocument/didChange" => {
                let params = not.params;
                let uri = params["textDocument"]["uri"].as_str().unwrap_or("").to_string();
                if let Some(changes) = params["contentChanges"].as_array() {
                    if let Some(last_change) = changes.last() {
                        if let Some(text) = last_change["text"].as_str() {
                            self.completion_provider.update_document(uri_to_string(&uri), text.to_string());
                        }
                    }
                }
            }
            "textDocument/didClose" => {
                let params = not.params;
                let uri = params["textDocument"]["uri"].as_str().unwrap_or("").to_string();
                self.completion_provider.remove_document(&uri_to_string(&uri));
            }
            _ => {}
        }
    }

    fn handle_initialize(&mut self, req: jsonrpc::Request) {
        if let Some(root_uri) = req.params["rootUri"].as_str() {
            self.workspace_root = Some(Self::uri_to_file_path(root_uri));
        }
        if let Some(folders) = req.params["workspaceFolders"].as_array() {
            if let Some(first) = folders.first() {
                if let Some(uri) = first["uri"].as_str() {
                    self.workspace_root = Some(Self::uri_to_file_path(uri));
                }
            }
        }

        let capabilities = serde_json::json!({
            "capabilities": {
                "textDocumentSync": 1,
                "completionProvider": {
                    "resolveProvider": false,
                    "triggerCharacters": ["/", "\"", "'", "."]
                }
            },
            "serverInfo": {
                "name": "zed-path-intellisense",
                "version": "0.1.0"
            }
        });

        jsonrpc::send_message(&jsonrpc::ResponseResult {
            jsonrpc: "2.0".to_string(),
            id: req.id,
            result: Some(capabilities),
            error: None,
        });
    }

    fn handle_completion(&mut self, req: jsonrpc::Request) {
        let params = req.params.clone();
        let uri_str = params["textDocument"]["uri"].as_str().unwrap_or("").to_string();
        let uri = uri_to_string(&uri_str);

        let position = Position {
            line: params["position"]["line"].as_u64().unwrap_or(0) as u32,
            character: params["position"]["character"].as_u64().unwrap_or(0) as u32,
        };

        eprintln!("[zed-path-intellisense-lsp] completion request for {} at {}:{}", uri, position.line, position.character);

        let completions = self.completion_provider.provide_completions(
            &uri,
            position,
            self.workspace_root.as_deref(),
        );

        let items: Vec<serde_json::Value> = completions
            .into_iter()
            .map(|item| serde_json::to_value(item).unwrap())
            .collect();

        jsonrpc::send_message(&jsonrpc::ResponseResult {
            jsonrpc: "2.0".to_string(),
            id: req.id,
            result: Some(serde_json::json!(items)),
            error: None,
        });
    }

    fn uri_to_file_path(uri: &str) -> PathBuf {
        PathBuf::from(decode_uri(uri))
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

fn uri_to_string(uri: &str) -> String {
    decode_uri(uri)
}

fn decode_uri(uri: &str) -> String {
    if let Some(rest) = uri.strip_prefix("file://") {
        percent_decode_str(rest)
            .decode_utf8()
            .map(|c| c.to_string())
            .unwrap_or_else(|_| rest.to_string())
    } else {
        uri.to_string()
    }
}

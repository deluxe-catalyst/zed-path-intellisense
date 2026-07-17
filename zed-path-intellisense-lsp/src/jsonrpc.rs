use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

pub type Id = serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Message {
    Request(Request),
    Notification(Notification),
    Response(Response),
}

#[derive(Debug, Deserialize)]
pub struct Request {
    #[serde(rename = "jsonrpc")]
    pub jsonrpc: String,
    pub id: Id,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct Notification {
    #[serde(rename = "jsonrpc")]
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct Response {
    #[serde(rename = "jsonrpc")]
    pub jsonrpc: String,
    #[serde(default)]
    pub id: Option<Id>,
    #[serde(default)]
    pub result: Option<serde_json::Value>,
    #[serde(default)]
    pub error: Option<ResponseError>,
}

#[derive(Debug, Serialize)]
pub struct ResponseResult {
    #[serde(rename = "jsonrpc")]
    pub jsonrpc: String,
    pub id: Id,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ResponseError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseError {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ResponseNotification {
    #[serde(rename = "jsonrpc")]
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
}

pub fn read_message() -> Option<Message> {
    let mut content_length: usize = 0;

    loop {
        let mut line = String::new();
        if std::io::stdin().read_line(&mut line).ok()? == 0 {
            return None;
        }
        let line = line.trim().to_string();
        if line.is_empty() {
            break;
        }
        if let Some(rest) = line.strip_prefix("Content-Length: ") {
            content_length = rest.parse().ok()?;
        }
    }

    if content_length == 0 {
        return None;
    }

    let mut body = vec![0u8; content_length];
    let mut total_read = 0;
    while total_read < content_length {
        let n = std::io::stdin().read(&mut body[total_read..]).ok()?;
        if n == 0 {
            return None;
        }
        total_read += n;
    }

    serde_json::from_slice(&body).ok()
}

pub fn send_message(result: &ResponseResult) {
    let body = serde_json::to_string(result).unwrap();
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    print!("{}{}", header, body);
    std::io::stdout().flush().ok();
}

pub fn send_notification(notification: &ResponseNotification) {
    let body = serde_json::to_string(notification).unwrap();
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    print!("{}{}", header, body);
    std::io::stdout().flush().ok();
}

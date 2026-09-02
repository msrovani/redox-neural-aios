//! Protocolo JSON-lines do scheme memory: (precursor IPC via socket).

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub cmd: String,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub k: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct Response {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Response {
    pub fn success(result: impl Into<Value>) -> Self {
        Self {
            ok: true,
            result: Some(result.into()),
            error: None,
        }
    }

    pub fn fail(msg: impl Into<String>) -> Self {
        Self {
            ok: false,
            result: None,
            error: Some(msg.into()),
        }
    }
}

pub fn parse_request(line: &str) -> Result<Request, String> {
    serde_json::from_str(line.trim()).map_err(|e| format!("json invalido: {e}"))
}

pub fn encode_response(resp: &Response) -> String {
    serde_json::to_string(resp).unwrap_or_else(|_| {
        r#"{"ok":false,"error":"serializacao falhou"}"#.to_string()
    })
}

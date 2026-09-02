//! Cliente RPC para hermesd.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

pub const DEFAULT_HERMES_SOCKET: &str = "127.0.0.1:7742";

pub struct HermesClient {
    addr: String,
}

impl HermesClient {
    pub fn new() -> Self {
        Self {
            addr: std::env::var("REDOX_HERMES_SOCKET")
                .unwrap_or_else(|_| DEFAULT_HERMES_SOCKET.to_string()),
        }
    }

    pub fn intent(&self, text: &str) -> Result<String, String> {
        let mut stream =
            TcpStream::connect(&self.addr).map_err(|e| format!("hermesd connect: {e}"))?;
        let body = serde_json::json!({"cmd":"intent","text":text});
        writeln!(stream, "{}", serde_json::to_string(&body).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        stream.flush().map_err(|e| e.to_string())?;

        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .map_err(|e| e.to_string())?;
        let val: serde_json::Value =
            serde_json::from_str(line.trim()).map_err(|e| e.to_string())?;
        if val.get("ok").and_then(|v| v.as_bool()) == Some(true) {
            return Ok(val
                .get("response")
                .and_then(|r| r.as_str())
                .unwrap_or("")
                .to_string());
        }
        Err(val
            .get("error")
            .and_then(|e| e.as_str())
            .unwrap_or("hermesd erro")
            .to_string())
    }
}

impl Default for HermesClient {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for HermesClient {
    fn clone(&self) -> Self {
        Self {
            addr: self.addr.clone(),
        }
    }
}

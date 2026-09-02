//! Cliente RPC para cortexd.

use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

pub const DEFAULT_CORTEX_SOCKET: &str = "127.0.0.1:7743";

fn rpc_timeout() -> Duration {
    std::env::var("REDOX_RPC_TIMEOUT_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .map(Duration::from_millis)
        .unwrap_or(Duration::from_secs(2))
}

pub struct CortexClient {
    addr: String,
}

impl CortexClient {
    pub fn new() -> Self {
        Self {
            addr: std::env::var("REDOX_CORTEX_SOCKET")
                .unwrap_or_else(|_| DEFAULT_CORTEX_SOCKET.to_string()),
        }
    }

    pub fn complete(&self, prompt: &str, system: Option<&str>) -> Result<String, String> {
        let mut body = serde_json::json!({
            "cmd": "complete",
            "prompt": prompt,
        });
        if let Some(sys) = system {
            body["system"] = serde_json::Value::String(sys.to_string());
        }

        let socket: SocketAddr = self
            .addr
            .parse()
            .map_err(|e| format!("cortexd endereço inválido {}: {e}", self.addr))?;
        let timeout = rpc_timeout();
        let mut stream = TcpStream::connect_timeout(&socket, timeout)
            .map_err(|e| format!("cortexd connect: {e}"))?;
        stream
            .set_read_timeout(Some(timeout))
            .map_err(|e| e.to_string())?;
        stream
            .set_write_timeout(Some(timeout))
            .map_err(|e| e.to_string())?;
        let line = serde_json::to_string(&body).map_err(|e| e.to_string())?;
        writeln!(stream, "{line}").map_err(|e| e.to_string())?;
        stream.flush().map_err(|e| e.to_string())?;

        let mut reader = BufReader::new(stream);
        let mut response = String::new();
        reader
            .read_line(&mut response)
            .map_err(|e| e.to_string())?;

        let val: serde_json::Value =
            serde_json::from_str(response.trim()).map_err(|e| e.to_string())?;
        if val.get("ok").and_then(|v| v.as_bool()) == Some(true) {
            return Ok(val
                .get("result")
                .and_then(|r| r.as_str())
                .unwrap_or("")
                .to_string());
        }
        Err(val
            .get("error")
            .and_then(|e| e.as_str())
            .unwrap_or("cortexd erro")
            .to_string())
    }
}

impl Default for CortexClient {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for CortexClient {
    fn clone(&self) -> Self {
        Self {
            addr: self.addr.clone(),
        }
    }
}

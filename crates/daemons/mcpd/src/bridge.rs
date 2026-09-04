//! Bridge TCP JSON-lines → sgdbd / hermesd.

use serde_json::{json, Value};

use memory_core::{rpc_timeout, MemoryClient, DEFAULT_SGDB_SOCKET};

pub const DEFAULT_HERMES_SOCKET: &str = "127.0.0.1:7742";

pub fn sgdb_addr() -> String {
    std::env::var("REDOX_SGDB_SOCKET").unwrap_or_else(|_| DEFAULT_SGDB_SOCKET.to_string())
}

pub fn hermes_addr() -> String {
    std::env::var("REDOX_HERMES_SOCKET").unwrap_or_else(|_| DEFAULT_HERMES_SOCKET.to_string())
}

pub fn memory_client() -> MemoryClient {
    MemoryClient::new()
}

/// Resposta hermesd completa (`ok` + `response`/`result`).
pub fn hermes_rpc(body: Value) -> Result<Value, String> {
    let addr = hermes_addr();
    let raw = tcp_raw(&addr, body)?;
    if raw.get("ok").and_then(|v| v.as_bool()) == Some(true) {
        Ok(raw)
    } else {
        Err(raw
            .get("error")
            .and_then(|e| e.as_str())
            .unwrap_or("hermesd erro")
            .to_string())
    }
}

pub fn hermes_intent(text: &str) -> Result<Value, String> {
    hermes_rpc(json!({"cmd": "intent", "text": text}))
}

pub fn hermes_ping() -> Result<Value, String> {
    hermes_rpc(json!({"cmd": "ping"}))
}

pub fn hermes_backends() -> Result<Value, String> {
    hermes_rpc(json!({"cmd": "backends"}))
}

fn tcp_raw(addr: &str, body: Value) -> Result<Value, String> {
    use std::io::{BufRead, BufReader, Write};
    use std::net::{SocketAddr, TcpStream};

    let socket: SocketAddr = addr
        .parse()
        .map_err(|e| format!("endereço inválido {addr}: {e}"))?;
    let timeout = rpc_timeout();
    let mut stream = TcpStream::connect_timeout(&socket, timeout)
        .map_err(|e| format!("conectar {addr}: {e}"))?;
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
    serde_json::from_str(response.trim()).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_addrs_contain_ports() {
        assert!(sgdb_addr().contains(':'));
        assert!(hermes_addr().contains(':'));
    }
}

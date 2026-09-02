//! Backend TCP JSON-lines (sgdbd).

use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

pub fn rpc_timeout() -> Duration {
    std::env::var("REDOX_RPC_TIMEOUT_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .map(Duration::from_millis)
        .unwrap_or(Duration::from_secs(2))
}

pub fn rpc(addr: &str, body: serde_json::Value) -> Result<serde_json::Value, String> {
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

    parse_response(&response)
}

fn parse_response(response: &str) -> Result<serde_json::Value, String> {
    let val: serde_json::Value =
        serde_json::from_str(response.trim()).map_err(|e| e.to_string())?;
    if val.get("ok").and_then(|v| v.as_bool()) == Some(true) {
        Ok(val.get("result").cloned().unwrap_or(serde_json::Value::Null))
    } else {
        Err(val
            .get("error")
            .and_then(|e| e.as_str())
            .unwrap_or("erro desconhecido")
            .to_string())
    }
}

//! Backend TCP JSON-lines (sgdbd).

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

pub fn rpc(addr: &str, body: serde_json::Value) -> Result<serde_json::Value, String> {
    let mut stream = TcpStream::connect(addr).map_err(|e| format!("conectar {addr}: {e}"))?;
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

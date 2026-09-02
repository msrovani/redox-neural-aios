//! Cliente EventBus remoto (eventd TCP 7740) + bridge scheme `chan:`.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

pub const DEFAULT_EVENTD_SOCKET: &str = "127.0.0.1:7740";

#[derive(Clone)]
pub struct EventClient {
    addr: String,
}

impl EventClient {
    pub fn new() -> Self {
        Self {
            addr: std::env::var("REDOX_EVENTD_SOCKET")
                .unwrap_or_else(|_| DEFAULT_EVENTD_SOCKET.to_string()),
        }
    }

    pub fn publish(&self, topic: &str, payload: &str) -> Result<(), String> {
        let mut stream =
            TcpStream::connect(&self.addr).map_err(|e| format!("eventd connect: {e}"))?;
        let req = serde_json::json!({
            "cmd": "publish",
            "topic": topic,
            "payload": payload,
        });
        writeln!(stream, "{}", req).map_err(|e| e.to_string())?;
        stream.flush().map_err(|e| e.to_string())?;
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).ok();
        Ok(())
    }
}

impl Default for EventClient {
    fn default() -> Self {
        Self::new()
    }
}

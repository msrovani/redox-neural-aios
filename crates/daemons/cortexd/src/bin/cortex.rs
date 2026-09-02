//! CLI cliente cortexd — teste de inferência no host.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

use cortex_core::DEFAULT_CORTEX_SOCKET;

fn main() {
    let prompt = std::env::args()
        .skip(1)
        .collect::<Vec<_>>()
        .join(" ");
    if prompt.trim().is_empty() {
        eprintln!("uso: cortex <prompt>");
        std::process::exit(1);
    }

    let addr = std::env::var("REDOX_CORTEX_SOCKET")
        .unwrap_or_else(|_| DEFAULT_CORTEX_SOCKET.to_string());
    let body = serde_json::json!({"cmd":"complete","prompt":prompt});
    let mut stream = TcpStream::connect(&addr).unwrap_or_else(|e| {
        eprintln!("cortexd connect {addr}: {e}");
        std::process::exit(1);
    });
    writeln!(stream, "{}", serde_json::to_string(&body).unwrap()).unwrap();
    stream.flush().unwrap();

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    let val: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
    if val.get("ok").and_then(|v| v.as_bool()) == Some(true) {
        println!(
            "{}",
            val.get("result").and_then(|r| r.as_str()).unwrap_or("")
        );
    } else {
        eprintln!(
            "erro: {}",
            val.get("error").and_then(|e| e.as_str()).unwrap_or("?")
        );
        std::process::exit(1);
    }
}

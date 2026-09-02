//! CLI Hermes — envia intents ao hermesd.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

use hermes_core::DEFAULT_HERMES_SOCKET;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Uso: hermes <texto/intent>");
        eprintln!("     hermes ping");
        std::process::exit(1);
    }

    let addr = std::env::var("REDOX_HERMES_SOCKET")
        .unwrap_or_else(|_| DEFAULT_HERMES_SOCKET.to_string());

    let (cmd, text) = if args[1] == "ping" {
        ("ping", String::new())
    } else {
        ("intent", args[1..].join(" "))
    };

    let mut stream = TcpStream::connect(&addr).expect("conectar hermesd");
    let body = if cmd == "ping" {
        serde_json::json!({"cmd":"ping"})
    } else {
        serde_json::json!({"cmd":"intent","text": text})
    };
    writeln!(stream, "{}", body).unwrap();
    stream.flush().unwrap();

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    println!("{}", line.trim());
}

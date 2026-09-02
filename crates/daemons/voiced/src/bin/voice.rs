//! CLI cliente voiced — pipeline Jarvis E2E.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

use voice_core::DEFAULT_VOICE_SOCKET;

fn rpc(body: serde_json::Value) -> Result<serde_json::Value, String> {
    let addr = std::env::var("REDOX_VOICE_SOCKET")
        .unwrap_or_else(|_| DEFAULT_VOICE_SOCKET.to_string());
    let mut stream = TcpStream::connect(&addr).map_err(|e| format!("voiced connect: {e}"))?;
    writeln!(stream, "{}", serde_json::to_string(&body).unwrap()).unwrap();
    stream.flush().unwrap();
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    serde_json::from_str(line.trim()).map_err(|e| e.to_string())
}

fn main() {
    let mut args = std::env::args().skip(1);
    let sub = args.next().unwrap_or_else(|| {
        eprintln!("uso: voice <utter|listen|transcribe|say|status|ping> [texto|wav]");
        std::process::exit(1);
    });
    let rest = args.collect::<Vec<_>>().join(" ");

    let body = match sub.as_str() {
        "utter" | "utterance" => serde_json::json!({"cmd":"utterance","text":rest}),
        "listen" => {
            if rest.ends_with(".wav") {
                serde_json::json!({"cmd":"listen","wav":rest})
            } else {
                serde_json::json!({"cmd":"listen","text":rest})
            }
        }
        "transcribe" => serde_json::json!({"cmd":"transcribe","wav":rest}),
        "say" => serde_json::json!({"cmd":"say","text":rest}),
        "status" => serde_json::json!({"cmd":"status"}),
        "ping" => serde_json::json!({"cmd":"ping"}),
        other => {
            eprintln!("subcomando desconhecido: {other}");
            std::process::exit(1);
        }
    };

    let val = rpc(body).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    });

    if val.get("ok").and_then(|v| v.as_bool()) == Some(true) {
        println!("{}", serde_json::to_string_pretty(&val["result"]).unwrap());
    } else {
        eprintln!(
            "erro: {}",
            val.get("error").and_then(|e| e.as_str()).unwrap_or("?")
        );
        std::process::exit(1);
    }
}

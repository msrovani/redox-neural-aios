//! CLI interativo Jarbas — HUD terminal conectado ao jarbasd.

use std::io::{self, BufRead, Write};
use std::net::TcpStream;

use jarbas_core::DEFAULT_JARBAS_SOCKET;

fn rpc(cmd: &str, text: &str) -> Result<serde_json::Value, String> {
    let addr = std::env::var("REDOX_JARBAS_SOCKET")
        .unwrap_or_else(|_| DEFAULT_JARBAS_SOCKET.to_string());
    let mut body = serde_json::json!({"cmd": cmd});
    if !text.is_empty() {
        body["text"] = serde_json::Value::String(text.to_string());
    }
    let mut stream = TcpStream::connect(&addr).map_err(|e| format!("jarbasd connect: {e}"))?;
    writeln!(stream, "{}", serde_json::to_string(&body).unwrap()).unwrap();
    stream.flush().unwrap();
    let mut reader = io::BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    serde_json::from_str(line.trim()).map_err(|e| e.to_string())
}

fn print_result(val: &serde_json::Value) {
    if val.get("ok").and_then(|v| v.as_bool()) == Some(true) {
        if let Some(hud) = val.pointer("/result/hud").and_then(|h| h.as_str()) {
            print!("{hud}");
        } else if let Some(response) = val.pointer("/result/response").and_then(|r| r.as_str()) {
            println!("{response}");
        } else {
            println!("{}", serde_json::to_string_pretty(&val["result"]).unwrap());
        }
    } else {
        eprintln!(
            "erro: {}",
            val.get("error").and_then(|e| e.as_str()).unwrap_or("?")
        );
        std::process::exit(1);
    }
}

fn repl() {
    println!("JARBAS shell — /hud /quit ou texto livre (Falcon3 via Hermes)\n");
    loop {
        print!("jarbas> ");
        io::stdout().flush().unwrap();
        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() {
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        match trimmed {
            "/quit" | "/exit" => break,
            "/hud" => print_result(&rpc("hud", "").unwrap()),
            "/greet" => print_result(&rpc("greet", "").unwrap()),
            "/status" => print_result(&rpc("status", "").unwrap()),
            cmd if cmd.starts_with("/voice ") => {
                print_result(&rpc("voice", cmd.trim_start_matches("/voice ").trim()).unwrap())
            }
            _ => print_result(&rpc("chat", trimmed).unwrap()),
        }
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let sub = args.next();

    match sub.as_deref() {
        None | Some("repl") => repl(),
        Some("hud") => print_result(&rpc("hud", "").unwrap()),
        Some("greet") => print_result(&rpc("greet", "").unwrap()),
        Some("status") => print_result(&rpc("status", "").unwrap()),
        Some("chat") => {
            let text = args.collect::<Vec<_>>().join(" ");
            print_result(&rpc("chat", &text).unwrap());
        }
        Some(other) => {
            eprintln!("uso: jarbas [repl|hud|greet|status|chat <texto>]");
            eprintln!("subcomando desconhecido: {other}");
            std::process::exit(1);
        }
    }
}

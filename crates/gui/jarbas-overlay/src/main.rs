//! Jarbas overlay — HUD flutuante conectado ao jarbasd (Onda B).
//! Roda em terminal dedicado no Orbital/COSMIC launcher ou autostart.

use std::io::{self, BufRead, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use jarbas_core::DEFAULT_JARBAS_SOCKET;

fn rpc(cmd: &str, text: &str) -> Result<serde_json::Value, String> {
    let addr = std::env::var("REDOX_JARBAS_SOCKET")
        .unwrap_or_else(|_| DEFAULT_JARBAS_SOCKET.to_string());
    let mut body = serde_json::json!({"cmd": cmd});
    if !text.is_empty() {
        body["text"] = serde_json::Value::String(text.to_string());
    }
    let mut stream = TcpStream::connect(&addr).map_err(|e| format!("jarbasd: {e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(30)))
        .ok();
    writeln!(stream, "{}", serde_json::to_string(&body).unwrap()).unwrap();
    stream.flush().unwrap();
    let mut reader = io::BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    serde_json::from_str(line.trim()).map_err(|e| e.to_string())
}

fn clear_screen() {
    print!("\x1B[2J\x1B[H");
    io::stdout().flush().ok();
}

fn render_hud() -> Result<(), String> {
    let val = rpc("hud", "")?;
    if val.get("ok").and_then(|v| v.as_bool()) == Some(true) {
        if let Some(hud) = val.pointer("/result/hud").and_then(|h| h.as_str()) {
            clear_screen();
            print!("{hud}");
            print!("overlay> digite e Enter (Ctrl+C sai)\n> ");
            io::stdout().flush().ok();
            return Ok(());
        }
    }
    Err(val
        .get("error")
        .and_then(|e| e.as_str())
        .unwrap_or("hud falhou")
        .to_string())
}

fn main() {
    let poll_ms = std::env::var("REDOX_OVERLAY_POLL_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3000u64);

    eprintln!("JARBAS overlay (Onda B) — conectando a jarbasd...");

    if render_hud().is_err() {
        eprintln!("jarbasd offline — inicie o daemon primeiro.");
        std::process::exit(1);
    }

    let poll = poll_ms;
    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(poll));
        let _ = render_hud();
    });

    let stdin = io::stdin();
    loop {
        print!("> ");
        io::stdout().flush().ok();
        let mut line = String::new();
        if stdin.lock().read_line(&mut line).is_err() {
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        match trimmed {
            "/quit" | "/exit" => break,
            "/hud" => {
                let _ = render_hud();
            }
            "/voice" => eprintln!("use /voice <texto> — ex: /voice jarvis, status"),
            cmd if cmd.starts_with("/voice ") => {
                let text = cmd.trim_start_matches("/voice ").trim();
                match rpc("voice", text) {
                    Ok(v) => {
                        if let Some(r) = v.pointer("/result/response").and_then(|x| x.as_str()) {
                            println!("{r}");
                        }
                        let _ = render_hud();
                    }
                    Err(e) => eprintln!("{e}"),
                }
            }
            _ => match rpc("chat", trimmed) {
                Ok(v) => {
                    if let Some(r) = v.pointer("/result/response").and_then(|x| x.as_str()) {
                        println!("{r}");
                    }
                    let _ = render_hud();
                }
                Err(e) => eprintln!("{e}"),
            },
        }
    }
}

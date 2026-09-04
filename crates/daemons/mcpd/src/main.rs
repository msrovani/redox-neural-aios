//! mcpd — entrypoint: MCP stdio (+ TCP opcional :7746).

use std::io::{self, BufRead, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use mcpd::{handle_message, MCP_SERVER_NAME, MCP_TOOL_COUNT};
use serde_json::Value;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let tcp = args.iter().any(|a| a == "--tcp")
        || std::env::var("REDOX_MCP_TCP")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

    eprintln!(
        "[mcpd] {MCP_SERVER_NAME} tools={MCP_TOOL_COUNT} stdio=on tcp={}",
        if tcp { "on" } else { "off" }
    );

    if tcp {
        thread::spawn(run_tcp_server);
    }

    // Stdio MCP — stdout SOMENTE mensagens JSON-RPC.
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let line = line.trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            continue;
        }
        match process_line(line) {
            Ok(Some(resp)) => send_stdout(&resp),
            Ok(None) => {}
            Err(err) => send_stdout(&err),
        }
    }
}

fn process_line(line: &str) -> Result<Option<Value>, Value> {
    let msg: Value = serde_json::from_str(line).map_err(|_| {
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": null,
            "error": {"code": -32700, "message": "Parse error"}
        })
    })?;
    Ok(handle_message(&msg))
}

fn send_stdout(v: &Value) {
    let mut out = io::stdout().lock();
    let _ = writeln!(out, "{v}");
    let _ = out.flush();
}

fn run_tcp_server() {
    let bind = std::env::var("REDOX_MCP_SOCKET").unwrap_or_else(|_| "127.0.0.1:7746".into());
    let listener = match TcpListener::bind(&bind) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[mcpd] TCP bind {bind} FAIL: {e}");
            return;
        }
    };
    eprintln!("[mcpd] TCP MCP em {bind}");
    for stream in listener.incoming().flatten() {
        thread::spawn(move || handle_tcp_client(stream));
    }
}

fn handle_tcp_client(stream: TcpStream) {
    let mut reader = io::BufReader::new(stream.try_clone().expect("clone"));
    let mut writer = stream;
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => break,
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            continue;
        }
        match process_line(trimmed) {
            Ok(Some(resp)) => {
                let _ = writeln!(writer, "{resp}");
                let _ = writer.flush();
            }
            Ok(None) => {}
            Err(err) => {
                let _ = writeln!(writer, "{err}");
                let _ = writer.flush();
            }
        }
    }
}

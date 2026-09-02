//! sgdbd — daemon de memória cognitiva (neural-sgdb) + TCP + scheme memory:

mod scheme_watcher;

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

use sgdbd::{handle_request, service::SgdbService, DEFAULT_SOCKET};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn log_line(msg: &str) {
    if let Ok(mut f) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/sgdbd.log")
    {
        let _ = writeln!(f, "{msg}");
    }
    println!("{msg}");
}

fn db_path_from_env() -> Option<std::path::PathBuf> {
    std::env::var("REDOX_SGDB_PATH")
        .ok()
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var("NEURAL_SGDB_DB").ok().map(std::path::PathBuf::from))
}

fn handle_client(service: &SgdbService, stream: TcpStream) {
    let peer = stream.peer_addr().ok();
    let reader = BufReader::new(stream.try_clone().expect("clone stream"));
    let mut writer = stream;

    for line in reader.lines().map_while(Result::ok) {
        if line.trim().is_empty() {
            continue;
        }
        scheme_watcher::scheme_poll_once(service);
        let response = handle_request(service, &line);
        if writeln!(writer, "{response}").is_err() {
            break;
        }
        let _ = writer.flush();
    }
    log_line(&format!("[sgdbd] cliente desconectado {:?}", peer));
}

fn main() {
    log_line(&format!(
        "sgdbd v{VERSION} — Redox Neural AIOS memory daemon (neural-sgdb)"
    ));

    let service = match SgdbService::open(db_path_from_env().as_deref()) {
        Ok(s) => s,
        Err(e) => {
            log_line(&format!("[sgdbd] FATAL: {e}"));
            std::process::exit(1);
        }
    };

    log_line(&format!(
        "[sgdbd] DB aberto em {} ({} docs)",
        service.db_path(),
        service.health().map(|h| h.doc_count).unwrap_or(0)
    ));

    let bind_addr =
        std::env::var("REDOX_SGDB_SOCKET").unwrap_or_else(|_| DEFAULT_SOCKET.to_string());
    let listener = TcpListener::bind(&bind_addr).unwrap_or_else(|e| {
        log_line(&format!("[sgdbd] FATAL bind {bind_addr}: {e}"));
        std::process::exit(1);
    });
    listener
        .set_nonblocking(true)
        .expect("nonblocking listener");

    let _ = fs::write("/tmp/sgdbd.pid", std::process::id().to_string());
    log_line(&format!("[sgdbd] memory: TCP {bind_addr} + scheme memory:"));
    log_line("[sgdbd] comandos: remember | recall | health | ping");

    loop {
        scheme_watcher::scheme_poll_once(&service);
        match listener.accept() {
            Ok((stream, _)) => handle_client(&service, stream),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(25));
            }
            Err(e) => {
                log_line(&format!("[sgdbd] accept erro: {e}"));
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
}

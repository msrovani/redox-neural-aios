//! Polling scheme `memory:` — JSON em `in/` e URIs em `open/in/`.

use std::fs;
use std::path::PathBuf;

use memory_core::uri_to_body;
use sgdbd::{handle_request, service::SgdbService};
const DEFAULT_SCHEME_ROOT: &str = "/scheme/memory";

fn scheme_root() -> PathBuf {
    std::env::var("REDOX_MEMORY_SCHEME_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_SCHEME_ROOT))
}

pub fn scheme_enabled() -> bool {
    std::env::var("REDOX_SGDB_SCHEME")
        .map(|v| v != "0")
        .unwrap_or(true)
}

/// Processa uma rodada de pedidos scheme (chamar no loop principal do daemon).
pub fn scheme_poll_once(service: &SgdbService) {
    if !scheme_enabled() {
        return;
    }

    let root = scheme_root();
    poll_json_in(service, &root);
    poll_uri_open(service, &root);
}

fn poll_json_in(service: &SgdbService, root: &PathBuf) {
    let in_dir = root.join("in");
    let out_dir = root.join("out");

    if fs::create_dir_all(&in_dir).is_err() || fs::create_dir_all(&out_dir).is_err() {
        return;
    }

    let Ok(entries) = fs::read_dir(&in_dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Ok(raw) = fs::read_to_string(&path) else {
            continue;
        };
        let line = raw.lines().next().unwrap_or("").trim();
        if line.is_empty() {
            let _ = fs::remove_file(&path);
            continue;
        }

        let response = handle_request(service, line);
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            let out_path = out_dir.join(format!("{stem}.json"));
            let _ = fs::write(&out_path, format!("{response}\n"));
        }
        let _ = fs::remove_file(&path);
    }
}

fn poll_uri_open(service: &SgdbService, root: &PathBuf) {
    let open_in = root.join("open").join("in");
    let open_out = root.join("open").join("out");

    if fs::create_dir_all(&open_in).is_err() || fs::create_dir_all(&open_out).is_err() {
        return;
    }

    let Ok(entries) = fs::read_dir(&open_in) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("uri") {
            continue;
        }
        let Ok(uri) = fs::read_to_string(&path) else {
            continue;
        };
        let uri = uri.trim();
        if uri.is_empty() {
            let _ = fs::remove_file(&path);
            continue;
        }

        let response = match uri_to_body(uri) {
            Ok(body) => {
                let line = serde_json::to_string(&body).unwrap_or_else(|e| {
                    format!(r#"{{"ok":false,"error":"{e}"}}"#)
                });
                handle_request(service, &line)
            }
            Err(e) => format!(r#"{{"ok":false,"error":"{e}"}}"#),
        };

        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            let out_path = open_out.join(format!("{stem}.json"));
            let _ = fs::write(&out_path, format!("{response}\n"));
        }
        let _ = fs::remove_file(&path);
    }
}

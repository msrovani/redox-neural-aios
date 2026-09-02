//! sgdbd — biblioteca do daemon de memória cognitiva Redox AIOS.

pub mod protocol;
pub mod service;

use protocol::{encode_response, parse_request, Response};
use service::SgdbService;

pub const DEFAULT_SOCKET: &str = "127.0.0.1:7741";

pub fn handle_request(service: &SgdbService, line: &str) -> String {
    let req = match parse_request(line) {
        Ok(r) => r,
        Err(e) => return encode_response(&Response::fail(e)),
    };

    let resp = match req.cmd.as_str() {
        "remember" => {
            let text = req.text.unwrap_or_default();
            let scope = req.scope.as_deref();
            match service.remember(&text, scope) {
                Ok(msg) => Response::success(msg),
                Err(e) => Response::fail(e),
            }
        }
        "recall" => {
            let query = req.query.or(req.text).unwrap_or_default();
            let k = req.k.unwrap_or(5).min(50);
            let scope = req.scope.as_deref();
            match service.recall(&query, scope, k) {
                Ok(hits) => {
                    let lines: Vec<serde_json::Value> = hits
                        .iter()
                        .map(|h| {
                            serde_json::json!({
                                "key": h.key,
                                "text": h.text,
                                "score": h.score,
                                "path": format!("{:?}", h.path),
                                "rel": h.rel,
                            })
                        })
                        .collect();
                    Response::success(serde_json::json!({ "hits": lines, "count": lines.len() }))
                }
                Err(e) => Response::fail(e),
            }
        }
        "health" => match service.health() {
            Ok(h) => Response::success(serde_json::json!({
                "doc_count": h.doc_count,
                "bq_len": h.bq_len,
                "open_conflicts": h.open_conflicts,
                "global_memory_count": h.global_memory_count,
                "scoped_memory_count": h.scoped_memory_count,
                "storage_ok": h.storage_ok,
                "db_path": service.db_path(),
            })),
            Err(e) => Response::fail(e),
        },
        "ping" => Response::success("pong"),
        other => Response::fail(format!("comando desconhecido: {other}")),
    };

    encode_response(&resp)
}

#[cfg(test)]
mod tests {
    use super::handle_request;
    use crate::service::SgdbService;

    #[test]
    fn remember_recall_roundtrip_file_storage() {
        let dir = std::env::temp_dir().join("redox-sgdb-test-dir");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");

        let svc = SgdbService::open_file_dir(&dir).expect("open dir");
        assert!(svc.db_path().ends_with("mem.db"));

        let remember = handle_request(
            &svc,
            r#"{"cmd":"remember","text":"file storage ok","scope":"boot"}"#,
        );
        assert!(remember.contains("\"ok\":true"), "{remember}");

        let svc2 = SgdbService::open_file_dir(&dir).expect("reopen");
        let recall = handle_request(
            &svc2,
            r#"{"cmd":"recall","query":"file storage","scope":"boot","k":3}"#,
        );
        assert!(recall.contains("file storage ok"), "{recall}");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn remember_recall_roundtrip() {
        let svc = SgdbService::open_in_memory().expect("open");

        let remember = handle_request(
            &svc,
            r#"{"cmd":"remember","text":"Redox AIOS boot ok","scope":"boot"}"#,
        );
        assert!(remember.contains("\"ok\":true"), "{remember}");

        let recall = handle_request(
            &svc,
            r#"{"cmd":"recall","query":"Redox boot","scope":"boot","k":3}"#,
        );
        assert!(recall.contains("\"ok\":true"), "{recall}");
        assert!(recall.contains("Redox AIOS"), "{recall}");
    }
}

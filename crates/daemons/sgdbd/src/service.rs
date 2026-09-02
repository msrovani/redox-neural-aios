//! Serviço neural-sgdb — backend compartilhado entre sgdbd e memory CLI.

use std::path::Path;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use neural_sgdb::{
    FileStorage, HealthReport, Hit, RememberOptions, Sgdb, DOCTRINE, DOCTRINE_ENTITIES,
    DOCTRINE_KEY, DOCTRINE_SCOPE,
};

const DEFAULT_DB_DIR: &str = "/var/lib/sgdb";
const DB_FILE_NAME: &str = "mem.db";

pub struct SgdbService {
    db: Mutex<Sgdb>,
    db_path: String,
}

/// `REDOX_SGDB_PATH` é um diretório (ADR-005); `FileStorage` exige arquivo `.db`.
fn resolve_storage_file(path: Option<&Path>) -> std::path::PathBuf {
    let base = path
        .map(std::path::Path::to_path_buf)
        .unwrap_or_else(|| std::path::PathBuf::from(DEFAULT_DB_DIR));

    if base.is_dir() || base.extension().is_none() {
        base.join(DB_FILE_NAME)
    } else {
        base
    }
}

impl SgdbService {
    pub fn open(path: Option<&Path>) -> Result<Self, String> {
        let storage_path = resolve_storage_file(path);
        if let Some(parent) = storage_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("criar {}: {e}", parent.display()))?;
        }

        let db_path = storage_path.to_string_lossy().into_owned();
        let storage =
            FileStorage::open(&storage_path).map_err(|e| format!("abrir storage: {e}"))?;
        let mut db = Sgdb::open(storage).map_err(|e| format!("abrir sgdb: {e}"))?;

        if db.health().doc_count == 0 {
            let _ = db.remember_text_with(
                DOCTRINE_KEY,
                DOCTRINE,
                RememberOptions {
                    scope: Some(DOCTRINE_SCOPE),
                    entities: DOCTRINE_ENTITIES,
                    content_type: Some("text"),
                },
            );
            let _ = db.checkpoint();
        }

        Ok(Self {
            db: Mutex::new(db),
            db_path,
        })
    }

    pub fn db_path(&self) -> &str {
        &self.db_path
    }

    pub fn remember(&self, text: &str, scope: Option<&str>) -> Result<String, String> {
        if text.trim().is_empty() {
            return Err("text vazio".into());
        }
        let key = format!(
            "redox-aios/{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0)
        );
        let mut db = self.db.lock().map_err(|_| "lock poisoned".to_string())?;
        let outcome = db
            .remember_text_with(
                &key,
                text,
                RememberOptions {
                    scope,
                    entities: &[],
                    content_type: Some("text"),
                },
            )
            .map_err(|e| format!("remember: {e}"))?;
        db.checkpoint().map_err(|e| format!("checkpoint: {e}"))?;
        Ok(format!(
            "ok key={} scope={} hint={}",
            outcome.storage_key, outcome.scope, outcome.recall_hint
        ))
    }

    pub fn recall(&self, query: &str, scope: Option<&str>, k: usize) -> Result<Vec<Hit>, String> {
        if query.trim().is_empty() {
            return Err("query vazia".into());
        }
        let mut db = self.db.lock().map_err(|_| "lock poisoned".to_string())?;
        let hits = match scope.filter(|s| !s.is_empty()) {
            Some(sc) => db
                .recall_lexical_scoped(query, k, sc)
                .map_err(|e| format!("recall scoped: {e}"))?,
            None => db
                .recall_lexical(query, k)
                .map_err(|e| format!("recall: {e}"))?,
        };
        if hits.is_empty() {
            if let Some(hint) = db.recall_empty_hint(scope.unwrap_or(""), "lexical") {
                return Err(hint);
            }
        }
        Ok(hits)
    }

    pub fn health(&self) -> Result<HealthReport, String> {
        let mut db = self.db.lock().map_err(|_| "lock poisoned".to_string())?;
        Ok(db.health())
    }

    #[cfg(test)]
    pub fn open_file_dir(base: &Path) -> Result<Self, String> {
        Self::open(Some(base))
    }

    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self, String> {
        use neural_sgdb::InMemory;
        let db = Sgdb::open(InMemory::new()).map_err(|e| format!("abrir in-memory: {e}"))?;
        Ok(Self {
            db: Mutex::new(db),
            db_path: ":memory:".to_string(),
        })
    }
}

pub fn format_hits(hits: &[Hit]) -> String {
    if hits.is_empty() {
        return "(vazio)".to_string();
    }
    hits.iter()
        .map(|h| {
            format!(
                "score={:.3} key={} text={}",
                h.score,
                h.key,
                h.text.chars().take(120).collect::<String>()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn format_health(h: &HealthReport) -> String {
    format!(
        "docs={} bq={} open_conflicts={} global={} scoped={} backend={}",
        h.doc_count,
        h.bq_len,
        h.open_conflicts,
        h.global_memory_count,
        h.scoped_memory_count,
        h.backend
    )
}

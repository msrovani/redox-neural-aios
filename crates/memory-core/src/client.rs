//! API de alto nível — remember / recall / health / ping.

use crate::{scheme, tcp};

pub const DEFAULT_SGDB_SOCKET: &str = "127.0.0.1:7741";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MemoryBackend {
    /// JSON-lines TCP (host dev, default).
    Tcp,
    /// Arquivos em `REDOX_MEMORY_SCHEME_ROOT` (bridge scheme `memory:`).
    Scheme,
}

impl MemoryBackend {
    pub fn from_env() -> Self {
        match std::env::var("REDOX_MEMORY_BACKEND")
            .unwrap_or_else(|_| "tcp".into())
            .to_ascii_lowercase()
            .as_str()
        {
            "scheme" | "memory" => Self::Scheme,
            _ => Self::Tcp,
        }
    }
}

pub struct MemoryClient {
    backend: MemoryBackend,
    tcp_addr: String,
}

impl MemoryClient {
    pub fn new() -> Self {
        Self {
            backend: MemoryBackend::from_env(),
            tcp_addr: std::env::var("REDOX_SGDB_SOCKET")
                .unwrap_or_else(|_| DEFAULT_SGDB_SOCKET.to_string()),
        }
    }

    pub fn with_backend(backend: MemoryBackend) -> Self {
        Self {
            backend,
            tcp_addr: std::env::var("REDOX_SGDB_SOCKET")
                .unwrap_or_else(|_| DEFAULT_SGDB_SOCKET.to_string()),
        }
    }

    pub fn backend(&self) -> &MemoryBackend {
        &self.backend
    }

    fn rpc(&self, body: serde_json::Value) -> Result<serde_json::Value, String> {
        match self.backend {
            MemoryBackend::Tcp => tcp::rpc(&self.tcp_addr, body),
            MemoryBackend::Scheme => scheme::rpc(body),
        }
    }

    pub fn remember(&self, text: &str, scope: Option<&str>) -> Result<serde_json::Value, String> {
        let mut body = serde_json::json!({ "cmd": "remember", "text": text });
        if let Some(s) = scope {
            body["scope"] = s.into();
        }
        self.rpc(body)
    }

    pub fn recall(
        &self,
        query: &str,
        scope: Option<&str>,
        k: Option<usize>,
    ) -> Result<serde_json::Value, String> {
        let mut body = serde_json::json!({ "cmd": "recall", "query": query });
        if let Some(s) = scope {
            body["scope"] = s.into();
        }
        if let Some(n) = k {
            body["k"] = n.into();
        }
        self.rpc(body)
    }

    pub fn health(&self) -> Result<serde_json::Value, String> {
        self.rpc(serde_json::json!({ "cmd": "health" }))
    }

    pub fn ping(&self) -> Result<serde_json::Value, String> {
        self.rpc(serde_json::json!({ "cmd": "ping" }))
    }

    pub fn recall_text(&self, query: &str, scope: &str, k: usize) -> Result<String, String> {
        let result = self.recall(query, Some(scope), Some(k))?;
        if let Some(hits) = result.get("hits").and_then(|h| h.as_array()) {
            let lines: Vec<String> = hits
                .iter()
                .filter_map(|h| h.get("text").and_then(|t| t.as_str()))
                .map(str::to_string)
                .collect();
            if lines.is_empty() {
                return Ok("(sem hits)".into());
            }
            return Ok(lines.join("\n"));
        }
        Ok(result.to_string())
    }
}

impl Default for MemoryClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_from_env_tcp_default() {
        std::env::remove_var("REDOX_MEMORY_BACKEND");
        assert_eq!(MemoryBackend::from_env(), MemoryBackend::Tcp);
    }

    #[test]
    fn backend_from_env_scheme() {
        std::env::set_var("REDOX_MEMORY_BACKEND", "scheme");
        assert_eq!(MemoryBackend::from_env(), MemoryBackend::Scheme);
        std::env::remove_var("REDOX_MEMORY_BACKEND");
    }
}

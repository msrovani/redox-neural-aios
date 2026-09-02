//! Cliente RPC para sgdbd (neural-sgdb) via memory-core.

pub use memory_core::{MemoryBackend, MemoryClient, DEFAULT_SGDB_SOCKET};

pub struct SgdbClient {
    inner: MemoryClient,
}

impl SgdbClient {
    pub fn new() -> Self {
        Self {
            inner: MemoryClient::new(),
        }
    }

    pub fn remember(&self, text: &str, scope: &str) -> Result<String, String> {
        let result = self.inner.remember(text, Some(scope))?;
        Ok(result.as_str().unwrap_or("ok").to_string())
    }

    pub fn recall(&self, query: &str, scope: &str, k: usize) -> Result<String, String> {
        self.inner.recall_text(query, scope, k)
    }

    pub fn health(&self) -> Result<String, String> {
        let result = self.inner.health()?;
        Ok(result.to_string())
    }
}

impl Default for SgdbClient {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for SgdbClient {
    fn clone(&self) -> Self {
        Self {
            inner: MemoryClient::with_backend(self.inner.backend().clone()),
        }
    }
}

impl SgdbClient {
    pub fn clone_for_skills(&self) -> Self {
        self.clone()
    }
}

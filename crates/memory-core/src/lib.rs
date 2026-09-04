//! Cliente unificado para memória cognitiva (`sgdbd` / scheme `memory:`).
//! Fase 1: TCP + scheme file. Fase 2: URIs nativas + bridge até handler Redox.

mod client;
mod scheme;
mod scheme_native;
mod scheme_open;
mod scheme_uri;
mod tcp;
pub use client::{MemoryBackend, MemoryClient, DEFAULT_SGDB_SOCKET};
pub use scheme_native::{backend_label, scheme_native_enabled};
pub use scheme_open::{body_to_uri, rpc_body, rpc_uri_at, uri_to_body};
pub use scheme_uri::{health_uri, parse_memory_uri, recall_uri, remember_uri};
pub use tcp::{rpc as tcp_rpc, rpc_timeout};

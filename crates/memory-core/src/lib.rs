//! Cliente unificado para memória cognitiva (`sgdbd` / scheme `memory:`).
//! Fase 1: TCP + scheme file (preparação para handler nativo no Redox).

mod client;
mod scheme;
mod tcp;

pub use client::{MemoryBackend, MemoryClient, DEFAULT_SGDB_SOCKET};

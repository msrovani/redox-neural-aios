//! Cortex — inferência LLM/MoE em userspace (Redox AIOS).
//! Modelo padrão: Falcon3-3B-Instruct-1.58bit (TII).

pub mod client;
pub mod engine;
pub mod falcon3;
pub mod inference;

pub use client::{CortexClient, DEFAULT_CORTEX_SOCKET};
pub use engine::{engine_from_env, CortexEngine, StubEngine};
pub use falcon3::{format_falcon3_prompt, Falcon3Config, DEFAULT_SYSTEM_PROMPT};
pub use inference::{AdaptiveEngine, Falcon3Engine};

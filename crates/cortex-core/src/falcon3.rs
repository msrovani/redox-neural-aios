//! Configuração e formatação Falcon3-3B-Instruct (1.58bit / GGUF).

use std::path::{Path, PathBuf};

pub const DEFAULT_SYSTEM_PROMPT: &str = "You are JARBAS, the AI assistant of Redox Neural AIOS. \
You are helpful, witty, and concise. Respond in Portuguese (Brazil) when the user writes in Portuguese.";

/// Modelo default: Falcon3-3B-Instruct Q4_K_M (qualidade).
pub const DEFAULT_MODEL_REL: &str = "models/Falcon3-3B-Instruct-Q4_K_M.gguf";

/// Variante leve 1.58bit (BitNet).
pub const LITE_MODEL_REL: &str = "models/Falcon3-3B-Instruct-1.58bit/ggml-model-i2_s.gguf";

/// Formata prompt no template Falcon3 (`<|system|>`, `<|user|>`, `<|assistant|>`).
pub fn format_falcon3_prompt(system: &str, user: &str) -> String {
    format!(
        "<|system|>\n{system}\n<|user|>\n{user}\n<|assistant|>\n"
    )
}

#[derive(Clone, Debug)]
pub struct Falcon3Config {
    pub model_path: PathBuf,
    pub backend: Falcon3Backend,
    pub llama_cli: PathBuf,
    pub bitnet_python: PathBuf,
    pub bitnet_script: PathBuf,
    pub max_tokens: u32,
    pub temperature: f32,
    pub system_prompt: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Falcon3Backend {
    /// Microsoft BitNet — modelo oficial 1.58bit (TII).
    BitNet,
    /// llama.cpp CLI — GGUF compatível (~IQ3_M / Q4_K_M).
    LlamaCpp,
}

impl Falcon3Config {
    pub fn from_env() -> Self {
        let model_path = std::env::var("REDOX_CORTEX_MODEL")
            .map(PathBuf::from)
            .unwrap_or_else(|_| default_model_path());

        let backend = match std::env::var("REDOX_CORTEX_BACKEND")
            .unwrap_or_else(|_| "auto".into())
            .to_ascii_lowercase()
            .as_str()
        {
            "bitnet" | "1.58bit" | "1.58" => Falcon3Backend::BitNet,
            "llama-cpp" | "llama" | "gguf" => Falcon3Backend::LlamaCpp,
            _ => detect_backend(&model_path),
        };

        let llama_cli = std::env::var("REDOX_LLAMA_CLI")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("llama-cli"));

        let bitnet_dir = std::env::var("REDOX_BITNET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("BitNet"));

        let bitnet_python = std::env::var("REDOX_BITNET_PYTHON")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("python"));

        let bitnet_script = bitnet_dir.join("run_inference.py");

        let max_tokens = std::env::var("REDOX_CORTEX_MAX_TOKENS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(256);

        let temperature = std::env::var("REDOX_CORTEX_TEMPERATURE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.7);

        let system_prompt = std::env::var("REDOX_CORTEX_SYSTEM")
            .unwrap_or_else(|_| DEFAULT_SYSTEM_PROMPT.to_string());

        Self {
            model_path,
            backend,
            llama_cli,
            bitnet_python,
            bitnet_script,
            max_tokens,
            temperature,
            system_prompt,
        }
    }

    pub fn model_exists(&self) -> bool {
        self.model_path.is_file()
    }
}

pub fn default_model_path() -> PathBuf {
    if let Ok(home) = std::env::var("REDOX_AIOS_HOME") {
        return PathBuf::from(home).join(DEFAULT_MODEL_REL);
    }
    if let Ok(cwd) = std::env::current_dir() {
        let local = cwd.join(DEFAULT_MODEL_REL);
        if local.is_file() {
            return local;
        }
    }
    dirs_home().join(DEFAULT_MODEL_REL)
}

fn dirs_home() -> PathBuf {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn detect_backend(model_path: &Path) -> Falcon3Backend {
    let name = model_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if name.contains("1.58") || name.contains("i2_s") || name.contains("q2b") {
        Falcon3Backend::BitNet
    } else {
        Falcon3Backend::LlamaCpp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn falcon3_prompt_template() {
        let p = format_falcon3_prompt("sys", "oi");
        assert!(p.contains("<|system|>"));
        assert!(p.contains("<|user|>\noi"));
        assert!(p.ends_with("<|assistant|>\n"));
    }
}

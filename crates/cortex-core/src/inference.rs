//! Backends de inferência Falcon3 (BitNet 1.58bit + llama.cpp GGUF).

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;

use crate::engine::CortexEngine;
use crate::engine::StubEngine;
use crate::falcon3::{format_falcon3_prompt, Falcon3Backend, Falcon3Config};

pub struct Falcon3Engine {
    config: Falcon3Config,
    lock: Mutex<()>,
}

impl Falcon3Engine {
    pub fn new(config: Falcon3Config) -> Result<Self, String> {
        if !config.model_exists() {
            return Err(format!(
                "modelo Falcon3 não encontrado: {} (rode tools/download-falcon3.ps1)",
                config.model_path.display()
            ));
        }
        validate_backend(&config)?;
        Ok(Self {
            config,
            lock: Mutex::new(()),
        })
    }

    pub fn config(&self) -> &Falcon3Config {
        &self.config
    }

    fn run_inference(&self, prompt: &str, system: &str) -> Result<String, String> {
        let _guard = self.lock.lock().map_err(|e| e.to_string())?;
        match self.config.backend {
            Falcon3Backend::BitNet => run_bitnet(&self.config, prompt, system),
            Falcon3Backend::LlamaCpp => run_llama_cpp(&self.config, prompt, system),
        }
    }
}

impl CortexEngine for Falcon3Engine {
    fn complete(&self, prompt: &str, system: Option<&str>) -> Result<String, String> {
        let trimmed = prompt.trim();
        if trimmed.is_empty() {
            return Ok("Estou ouvindo.".into());
        }
        let system_text = system
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(&self.config.system_prompt);
        let out = self.run_inference(trimmed, system_text)?;
        Ok(clean_model_output(&out))
    }
}

fn validate_backend(config: &Falcon3Config) -> Result<(), String> {
    match config.backend {
        Falcon3Backend::BitNet => {
            if !config.bitnet_script.is_file() {
                return Err(format!(
                    "BitNet não encontrado em {} — clone https://github.com/microsoft/BitNet \
                     ou defina REDOX_BITNET_DIR / REDOX_CORTEX_BACKEND=llama-cpp",
                    config.bitnet_script.display()
                ));
            }
        }
        Falcon3Backend::LlamaCpp => {
            if which_llama_cli(&config.llama_cli).is_none() {
                return Err(format!(
                    "llama-cli não encontrado ({}) — instale llama.cpp ou defina REDOX_LLAMA_CLI",
                    config.llama_cli.display()
                ));
            }
        }
    }
    Ok(())
}

fn which_llama_cli(path: &Path) -> Option<PathBuf> {
    if path.is_file() {
        return Some(path.to_path_buf());
    }
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths).find_map(|dir| {
            let candidate = dir.join(path);
            if candidate.is_file() {
                Some(candidate)
            } else {
                None
            }
        })
    })
}

fn run_llama_cpp(config: &Falcon3Config, user: &str, system: &str) -> Result<String, String> {
    let cli = which_llama_cli(&config.llama_cli)
        .ok_or_else(|| format!("llama-cli indisponível: {}", config.llama_cli.display()))?;
    let full_prompt = format_falcon3_prompt(system, user);

    let output = Command::new(&cli)
        .arg("-m")
        .arg(&config.model_path)
        .arg("-p")
        .arg(&full_prompt)
        .arg("-n")
        .arg(config.max_tokens.to_string())
        .arg("--temp")
        .arg(config.temperature.to_string())
        .arg("--no-display-prompt")
        .arg("-e")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("llama-cli spawn: {e}"))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("llama-cli falhou: {err}"));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn run_bitnet(config: &Falcon3Config, user: &str, system: &str) -> Result<String, String> {
    let conversation = format_falcon3_prompt(system, user);

    let output = Command::new(&config.bitnet_python)
        .arg(&config.bitnet_script)
        .arg("-m")
        .arg(&config.model_path)
        .arg("-p")
        .arg(&conversation)
        .arg("-cnv")
        .arg("-n")
        .arg(config.max_tokens.to_string())
        .arg("-t")
        .arg(config.temperature.to_string())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("bitnet spawn: {e}"))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("bitnet falhou: {err}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    extract_bitnet_reply(&stdout)
}

fn extract_bitnet_reply(stdout: &str) -> Result<String, String> {
    let mut lines: Vec<&str> = stdout.lines().collect();
    while let Some(last) = lines.last() {
        let t = last.trim();
        if t.is_empty() || t.starts_with(">>>") || t.starts_with("Loading") {
            lines.pop();
        } else {
            break;
        }
    }
    lines
        .last()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "bitnet sem resposta".into())
}

fn clean_model_output(raw: &str) -> String {
    let mut text = raw.trim().to_string();
    for marker in ["<|assistant|>", "<|user|>", "<|system|>", "<|endoftext|>"] {
        if let Some(idx) = text.find(marker) {
            text = text[..idx].trim().to_string();
        }
    }
    text.trim().to_string()
}

/// Engine híbrido: Falcon3 quando disponível, stub como fallback.
pub struct AdaptiveEngine {
    falcon: Option<Falcon3Engine>,
    stub: StubEngine,
}

impl AdaptiveEngine {
    pub fn from_env() -> Self {
        let stub = StubEngine {
            persona: std::env::var("REDOX_CORTEX_PERSONA").unwrap_or_else(|_| "JARBAS".into()),
        };

        let force_stub = std::env::var("REDOX_CORTEX_ENGINE")
            .map(|v| v.eq_ignore_ascii_case("stub"))
            .unwrap_or(false);

        let falcon = if force_stub {
            None
        } else {
            let config = Falcon3Config::from_env();
            match Falcon3Engine::new(config) {
                Ok(engine) => Some(engine),
                Err(e) => {
                    eprintln!("[cortex] Falcon3 indisponível, usando stub: {e}");
                    None
                }
            }
        };

        Self { falcon, stub }
    }

    pub fn engine_name(&self) -> &'static str {
        if self.falcon.is_some() {
            "falcon3-3b-1.58bit"
        } else {
            "stub"
        }
    }
}

impl CortexEngine for AdaptiveEngine {
    fn complete(&self, prompt: &str, system: Option<&str>) -> Result<String, String> {
        if let Some(falcon) = &self.falcon {
            falcon.complete(prompt, system)
        } else {
            self.stub.complete(prompt, system)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_strips_markers() {
        assert_eq!(
            clean_model_output("Olá mundo<|endoftext|>"),
            "Olá mundo"
        );
    }
}

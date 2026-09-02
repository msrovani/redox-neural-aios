//! Registro e execução de tools Hermes (host userspace).

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

use crate::cortex_client::CortexClient;
use crate::sgdb_client::SgdbClient;

pub struct ToolContext<'a> {
    pub intent: &'a str,
    pub sgdb: &'a SgdbClient,
    pub cortex: &'a CortexClient,
}

pub struct ToolResult {
    pub step_id: String,
    pub ok: bool,
    pub output: String,
}

pub trait HermesTool: Send + Sync {
    fn id(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn run(&self, ctx: &ToolContext<'_>, arg: &str) -> Result<String, String>;
}

pub struct ToolRegistry {
    tools: BTreeMap<&'static str, Box<dyn HermesTool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: BTreeMap::new(),
        }
    }

    pub fn register(&mut self, tool: Box<dyn HermesTool>) {
        let id = tool.id();
        self.tools.insert(id, tool);
    }

    pub fn list(&self) -> Vec<(&'static str, &'static str)> {
        self.tools
            .values()
            .map(|t| (t.id(), t.description()))
            .collect()
    }

    pub fn run_step(&self, step_id: &str, ctx: &ToolContext<'_>, arg: &str) -> ToolResult {
        match self.tools.get(step_id) {
            Some(tool) => match tool.run(ctx, arg) {
                Ok(output) => ToolResult {
                    step_id: step_id.to_string(),
                    ok: true,
                    output,
                },
                Err(e) => ToolResult {
                    step_id: step_id.to_string(),
                    ok: false,
                    output: e,
                },
            },
            None => ToolResult {
                step_id: step_id.to_string(),
                ok: false,
                output: format!("tool não registrada: {step_id}"),
            },
        }
    }

    pub fn has(&self, step_id: &str) -> bool {
        self.tools.contains_key(step_id)
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

struct UnderstandIntentTool;
impl HermesTool for UnderstandIntentTool {
    fn id(&self) -> &'static str {
        "understand_intent"
    }
    fn description(&self) -> &'static str {
        "Normaliza e classifica o intent do usuário"
    }
    fn run(&self, ctx: &ToolContext<'_>, _arg: &str) -> Result<String, String> {
        Ok(format!("intent={}", ctx.intent.trim()))
    }
}

struct MemoryRecallTool;
impl HermesTool for MemoryRecallTool {
    fn id(&self) -> &'static str {
        "memory_recall"
    }
    fn description(&self) -> &'static str {
        "Recall lexical no neural-sgdb (scope hermes)"
    }
    fn run(&self, ctx: &ToolContext<'_>, arg: &str) -> Result<String, String> {
        let query = if arg.is_empty() { ctx.intent } else { arg };
        match ctx.sgdb.recall(query, "hermes", 3) {
            Ok(text) => Ok(text),
            Err(e) => Ok(format!("memory_unavailable: {e}")),
        }
    }
}

struct PlanDataSourcesTool;
impl HermesTool for PlanDataSourcesTool {
    fn id(&self) -> &'static str {
        "plan_data_sources"
    }
    fn description(&self) -> &'static str {
        "Planeja fontes/provedores necessários (Cortex local)"
    }
    fn run(&self, ctx: &ToolContext<'_>, _arg: &str) -> Result<String, String> {
        let prompt = format!(
            "Liste em uma linha as fontes de dados ou APIs que seriam necessárias \
             para responder: \"{}\". Se não souber, diga 'local_only'.",
            ctx.intent
        );
        ctx.cortex
            .complete(&prompt, None)
            .map(|s| format!("plan: {s}"))
    }
}

struct FetchExternalTool;
impl HermesTool for FetchExternalTool {
    fn id(&self) -> &'static str {
        "fetch_external"
    }
    fn description(&self) -> &'static str {
        "Busca dados externos via providers HTTP (REDOX_TOOLS_NET=1 + REDOX_TOOLS_PROVIDERS)"
    }
    fn run(&self, ctx: &ToolContext<'_>, arg: &str) -> Result<String, String> {
        let context = if arg.is_empty() { ctx.intent } else { arg };
        super::providers::fetch_via_providers(ctx.intent, context)
    }
}

struct SynthesizeResponseTool;
impl HermesTool for SynthesizeResponseTool {
    fn id(&self) -> &'static str {
        "synthesize_response"
    }
    fn description(&self) -> &'static str {
        "Síntese final via Cortex (local)"
    }
    fn run(&self, ctx: &ToolContext<'_>, arg: &str) -> Result<String, String> {
        let prompt = if arg.is_empty() {
            ctx.intent.to_string()
        } else {
            format!(
                "Com base neste contexto:\n{arg}\n\nResponda ao usuário: {}",
                ctx.intent
            )
        };
        ctx.cortex.complete(&prompt, None)
    }
}

struct PrepareVoiceOutputTool;
impl HermesTool for PrepareVoiceOutputTool {
    fn id(&self) -> &'static str {
        "prepare_voice_output"
    }
    fn description(&self) -> &'static str {
        "Marca resposta para pipeline TTS (voiced)"
    }
    fn run(&self, _ctx: &ToolContext<'_>, arg: &str) -> Result<String, String> {
        Ok(format!("tts_ready: {}", arg.chars().take(120).collect::<String>()))
    }
}

pub fn default_tool_registry() -> ToolRegistry {
    let mut reg = ToolRegistry::new();
    reg.register(Box::new(UnderstandIntentTool));
    reg.register(Box::new(MemoryRecallTool));
    reg.register(Box::new(PlanDataSourcesTool));
    reg.register(Box::new(FetchExternalTool));
    reg.register(Box::new(SynthesizeResponseTool));
    reg.register(Box::new(PrepareVoiceOutputTool));
    reg
}

fn global_registry() -> &'static Mutex<ToolRegistry> {
    static REG: OnceLock<Mutex<ToolRegistry>> = OnceLock::new();
    REG.get_or_init(|| Mutex::new(default_tool_registry()))
}

pub fn shared_tool_registry() -> std::sync::MutexGuard<'static, ToolRegistry> {
    global_registry().lock().expect("tool registry")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_tools_registered() {
        let reg = default_tool_registry();
        assert!(reg.has("memory_recall"));
        assert!(reg.has("fetch_external"));
        assert_eq!(reg.list().len(), 6);
    }
}

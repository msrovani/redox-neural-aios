//! Degrau 0 — execução efêmera ReAct + tools (ADR-010).
//! Pipeline genérico: entender → memória → plano → fetch → síntese → voz.
//! Steps observados alimentam skill_gen na recorrência.

use crate::cortex_client::CortexClient;
use crate::sgdb_client::SgdbClient;
use crate::skill_gen::{record_observed_steps, EPHEMERAL_PIPELINE};
use crate::skill_observer::normalize_key;
use crate::tools::{ToolContext, ToolRegistry, ToolResult};

#[derive(Clone, Debug)]
pub struct EphemeralResult {
    pub response: String,
    pub steps: Vec<String>,
    pub traces: Vec<String>,
}

pub fn run_ephemeral(
    intent: &str,
    sgdb: &SgdbClient,
    cortex: &CortexClient,
    tools: &ToolRegistry,
) -> EphemeralResult {
    let ctx = ToolContext {
        intent,
        sgdb,
        cortex,
    };

    let mut traces = Vec::new();
    let mut context_buf = String::new();
    let mut final_response = String::new();

    for step_id in EPHEMERAL_PIPELINE {
        let arg = match *step_id {
            "synthesize_response" | "prepare_voice_output" => context_buf.as_str(),
            "fetch_external" => {
                if context_buf.is_empty() {
                    intent
                } else {
                    context_buf.as_str()
                }
            }
            _ => intent,
        };

        let result: ToolResult = tools.run_step(step_id, &ctx, arg);
        traces.push(format!(
            "{}: {}",
            result.step_id,
            if result.ok { "ok" } else { "err" }
        ));

        if result.ok {
            if *step_id == "synthesize_response" {
                final_response = result.output.clone();
            }
            if !result.output.is_empty() {
                if !context_buf.is_empty() {
                    context_buf.push('\n');
                }
                context_buf.push_str(&result.output);
            }
        }
    }

    if final_response.is_empty() {
        final_response = cortex
            .complete(intent, None)
            .unwrap_or_else(|e| format!("(efêmera fallback) {e}"));
    }

    let steps: Vec<String> = EPHEMERAL_PIPELINE.iter().map(|s| (*s).to_string()).collect();
    let key = normalize_key(intent);
    record_observed_steps(&key, intent, &steps);

    EphemeralResult {
        response: final_response,
        steps,
        traces,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::default_tool_registry;

    #[test]
    fn ephemeral_runs_pipeline() {
        std::env::set_var("REDOX_RPC_TIMEOUT_MS", "500");
        let sgdb = SgdbClient::new();
        let cortex = CortexClient::new();
        let tools = default_tool_registry();
        let out = run_ephemeral("qual a temperatura?", &sgdb, &cortex, &tools);
        assert!(!out.response.is_empty());
        assert_eq!(out.steps.len(), EPHEMERAL_PIPELINE.len());
        assert!(out.traces.iter().any(|t| t.starts_with("understand_intent:")));
    }
}

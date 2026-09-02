//! Execução de workflow SKILL.md (degrau 1) via tool registry.

use skill_registry::DynamicSkill;

use crate::cortex_client::CortexClient;
use crate::sgdb_client::SgdbClient;
use crate::tools::{ToolContext, ToolRegistry};

pub fn execute_skill_workflow(
    skill: &DynamicSkill,
    input: &str,
    sgdb: &SgdbClient,
    cortex: &CortexClient,
    tools: &ToolRegistry,
) -> Result<String, String> {
    if skill.wasm.is_some() {
        return skill.execute(input);
    }

    let ctx = ToolContext {
        intent: input,
        sgdb,
        cortex,
    };

    let steps = if skill.workflow.is_empty() {
        crate::skill_gen::generic_workflow_steps()
    } else {
        skill.workflow.clone()
    };

    let mut lines = vec![format!("[{}] SKILL.md workflow", skill.name)];
    let mut context = String::new();

    for (i, step_id) in steps.iter().enumerate() {
        let arg = match step_id.as_str() {
            "synthesize_response" | "prepare_voice_output" => context.as_str(),
            "fetch_external" => {
                if context.is_empty() {
                    input
                } else {
                    context.as_str()
                }
            }
            _ => input,
        };
        let result = tools.run_step(step_id, &ctx, arg);
        lines.push(format!(
            "  {}. {} → {}",
            i + 1,
            step_id,
            if result.ok { "ok" } else { "err" }
        ));
        if result.ok && !result.output.is_empty() {
            if !context.is_empty() {
                context.push('\n');
            }
            context.push_str(&result.output);
        }
    }

    let tail = if context.is_empty() {
        skill.description.clone()
    } else {
        context.chars().take(200).collect()
    };
    lines.push(format!("  input: {input}"));
    lines.push(format!("  ctx: {tail}"));
    Ok(lines.join("\n"))
}

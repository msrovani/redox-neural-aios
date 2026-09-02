//! Hermes router — intent → skill ou Falcon3 (cortexd).

use skill_registry::SkillRegistry;

use crate::cortex_client::CortexClient;
use crate::intent::{command_to_skill, parse_command, Command};
use crate::prompt::system_prompt;
use crate::react::ReActTrace;
use crate::sgdb_client::SgdbClient;

pub struct HermesRouter {
    pub registry: SkillRegistry,
    pub sgdb: SgdbClient,
    pub cortex: CortexClient,
}

pub struct RouteResult {
    pub response: String,
    pub topic: &'static str,
    pub trace: Vec<String>,
}

impl HermesRouter {
    pub fn new(registry: SkillRegistry) -> Self {
        Self {
            registry,
            sgdb: SgdbClient::new(),
            cortex: CortexClient::new(),
        }
    }

    fn cortex_complete(&self, text: &str) -> String {
        match self.cortex.complete(text, Some(&system_prompt())) {
            Ok(r) => r,
            Err(e) => i18n_core::t_fmt("jarbas.offline", &[("error", &e)]),
        }
    }

    fn finish_intelligence(
        &self,
        text: &str,
        response: String,
        trace: &mut ReActTrace,
        lines: &mut Vec<String>,
    ) -> RouteResult {
        let _ = self
            .sgdb
            .remember(&format!("Q:{text} A:{response}"), "hermes");
        trace.advance("cortex+remember");
        lines.push(trace.line());
        RouteResult {
            response,
            topic: "HERMES_RESPONSE",
            trace: lines.clone(),
        }
    }

    pub fn handle_intent(&self, text: &str) -> RouteResult {
        let mut trace = ReActTrace::new(format!("intent: {text}"));
        let mut lines = vec![trace.line()];

        if let Some(blocked) = crate::hitl::gate_response(text) {
            trace.advance("hitl_blocked");
            lines.push(trace.line());
            return RouteResult {
                response: blocked,
                topic: "HERMES_HITL_BLOCKED",
                trace: lines,
            };
        }

        let cmd = parse_command(text);
        trace.advance("parse_command");
        lines.push(trace.line());

        if let Command::Chat(ref chat) = cmd {
            trace.advance("cortex complete (chat)");
            lines.push(trace.line());
            let response = self.cortex_complete(chat);
            return self.finish_intelligence(text, response, &mut trace, &mut lines);
        }

        if let Some((skill, input)) = command_to_skill(&cmd) {
            // Skills determinísticas locais; demais passam pelo Falcon3.
            if matches!(
                skill,
                "echo" | "time" | "status" | "remember" | "recall" | "skills" | "help"
            ) {
                trace.advance(format!("execute skill={skill}"));
                lines.push(trace.line());
                let response = match self.registry.execute(skill, &input) {
                    Ok(r) => r,
                    Err(e) => format!("erro skill {skill}: {e}"),
                };
                let _ = self
                    .sgdb
                    .remember(&format!("Q:{text} A:{response}"), "hermes");
                trace.advance("verify+remember");
                lines.push(trace.line());
                return RouteResult {
                    response,
                    topic: "HERMES_RESPONSE",
                    trace: lines,
                };
            }

            trace.advance(format!("cortex skill={skill}"));
            lines.push(trace.line());
            let prompt = if input.is_empty() {
                format!("O usuário pediu a skill /{skill}. Responda de forma útil.")
            } else {
                format!(
                    "O usuário pediu a skill /{skill} com entrada: {input}. \
                     Execute mentalmente e responda."
                )
            };
            let response = self.cortex_complete(&prompt);
            return self.finish_intelligence(text, response, &mut trace, &mut lines);
        }

        if let Command::Unknown(ref unknown) = cmd {
            trace.advance("cortex complete (unknown)");
            lines.push(trace.line());
            let response = self.cortex_complete(unknown);
            return self.finish_intelligence(text, response, &mut trace, &mut lines);
        }

        RouteResult {
            response: "nada a executar".into(),
            topic: "HERMES_RESPONSE",
            trace: lines,
        }
    }
}

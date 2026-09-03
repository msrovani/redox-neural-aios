//! Trinity MoE skeleton — roteamento multi-expert (ADR-007 / ADR-011).
//! Host: stub de seleção; produção liga Falcon3 + experts futuros.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpertKind {
    General,
    Code,
    Voice,
    Memory,
    Safety,
}

impl ExpertKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::General => "general",
            Self::Code => "code",
            Self::Voice => "voice",
            Self::Memory => "memory",
            Self::Safety => "safety",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExpertRoute {
    pub expert: ExpertKind,
    pub weight: f32,
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrinityPlan {
    pub primary: ExpertRoute,
    pub secondary: Option<ExpertRoute>,
    pub moe_enabled: bool,
}

impl TrinityPlan {
    pub fn format(&self) -> String {
        let mut out = format!(
            "trinity moe={} primary={}@{:.2} ({})",
            self.moe_enabled,
            self.primary.expert.as_str(),
            self.primary.weight,
            self.primary.reason
        );
        if let Some(ref s) = self.secondary {
            out.push_str(&format!(
                " secondary={}@{:.2}",
                s.expert.as_str(),
                s.weight
            ));
        }
        out
    }
}

pub fn moe_enabled() -> bool {
    std::env::var("REDOX_CORTEX_MOE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Roteia intent para expert(s) — heurística lexical (skeleton).
pub fn route_intent(text: &str) -> TrinityPlan {
    let lower = text.to_ascii_lowercase();
    let (expert, reason) = if lower.contains("rm -rf")
        || lower.contains("delete")
        || lower.contains("format")
        || lower.contains("apague")
    {
        (ExpertKind::Safety, "padrões destrutivos")
    } else if lower.contains("código")
        || lower.contains("code")
        || lower.contains("compile")
        || lower.contains("rust")
        || lower.contains("fn ")
    {
        (ExpertKind::Code, "intent de código")
    } else if lower.contains("voz")
        || lower.contains("voice")
        || lower.contains("tts")
        || lower.contains("wake")
    {
        (ExpertKind::Voice, "pipeline de voz")
    } else if lower.contains("lembre")
        || lower.contains("remember")
        || lower.contains("recall")
        || lower.contains("memória")
        || lower.contains("memory")
    {
        (ExpertKind::Memory, "memória cognitiva")
    } else {
        (ExpertKind::General, "chat geral")
    };

    let secondary = match expert {
        ExpertKind::Code => Some(ExpertRoute {
            expert: ExpertKind::General,
            weight: 0.25,
            reason: "fallback geral".into(),
        }),
        ExpertKind::Safety => Some(ExpertRoute {
            expert: ExpertKind::General,
            weight: 0.1,
            reason: "explicação segura".into(),
        }),
        _ => None,
    };

    TrinityPlan {
        primary: ExpertRoute {
            expert,
            weight: 0.85,
            reason: reason.into(),
        },
        secondary,
        moe_enabled: moe_enabled(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_code_and_safety() {
        let code = route_intent("compile este rust");
        assert_eq!(code.primary.expert, ExpertKind::Code);
        let safe = route_intent("please rm -rf /");
        assert_eq!(safe.primary.expert, ExpertKind::Safety);
    }
}

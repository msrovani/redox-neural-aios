//! Self-skill generation — TaskPattern → SKILL.md (Onda 7h).
//! Genérico: steps vêm do pipeline efêmero observado, não de domínio hardcoded.

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

/// Pipeline padrão degrau 0 (efêmera) — espelha o exemplo usabilidade ADR-010.
pub const EPHEMERAL_PIPELINE: &[&str] = &[
    "understand_intent",
    "memory_recall",
    "plan_data_sources",
    "fetch_external",
    "synthesize_response",
    "prepare_voice_output",
];

#[derive(Clone, Debug)]
pub struct TaskPattern {
    pub name: String,
    pub description: String,
    pub triggers: Vec<String>,
    pub steps: Vec<String>,
    pub uses: u32,
}

fn patterns() -> &'static Mutex<BTreeMap<String, TaskPattern>> {
    static PATTERNS: OnceLock<Mutex<BTreeMap<String, TaskPattern>>> = OnceLock::new();
    PATTERNS.get_or_init(|| Mutex::new(BTreeMap::new()))
}

pub fn generic_workflow_steps() -> Vec<String> {
    EPHEMERAL_PIPELINE.iter().map(|s| (*s).to_string()).collect()
}

/// Nome estável da skill a partir da chave normalizada (sem heurística de domínio).
pub fn skill_name_from_key(normalized_key: &str) -> String {
    let mut name = normalized_key.to_string();
    for prefix in ["qual_", "como_", "onde_", "quando_", "quanto_", "o_que_"] {
        if let Some(rest) = name.strip_prefix(prefix) {
            name = rest.to_string();
            break;
        }
    }
    name = name.trim_matches('_').to_string();
    if name.is_empty() {
        return "auto_skill".into();
    }
    if name.len() > 40 {
        name.truncate(40);
    }
    name
}

/// Registra padrão observado (contagem de uses por chave normalizada).
pub fn record_task(name: &str, description: &str, steps: &[&str]) {
    let Ok(mut map) = patterns().lock() else {
        return;
    };
    if let Some(p) = map.get_mut(name) {
        p.uses += 1;
        if !steps.is_empty() && p.steps.is_empty() {
            p.steps = steps.iter().map(|s| (*s).to_string()).collect();
        }
        return;
    }
    map.insert(
        name.to_string(),
        TaskPattern {
            name: name.to_string(),
            description: description.to_string(),
            triggers: vec![name.to_string()],
            steps: steps.iter().map(|s| (*s).to_string()).collect(),
            uses: 1,
        },
    );
}

/// Atualiza steps observados após execução efêmera (degrau 0).
pub fn record_observed_steps(normalized_key: &str, description: &str, steps: &[String]) {
    let Ok(mut map) = patterns().lock() else {
        return;
    };
    if let Some(p) = map.get_mut(normalized_key) {
        p.uses += 1;
        if !steps.is_empty() {
            p.steps = steps.to_vec();
        }
        if p.description.is_empty() {
            p.description = description.to_string();
        }
        return;
    }
    map.insert(
        normalized_key.to_string(),
        TaskPattern {
            name: normalized_key.to_string(),
            description: description.to_string(),
            triggers: vec![normalized_key.to_string()],
            steps: steps.to_vec(),
            uses: 1,
        },
    );
}

pub fn workflow_steps_for(normalized_key: &str) -> Vec<String> {
    patterns()
        .lock()
        .ok()
        .and_then(|m| m.get(normalized_key).map(|p| p.steps.clone()))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(generic_workflow_steps)
}

/// Gera SKILL.md no formato ADR-0052 subset.
pub fn generate_skill_md(
    name: &str,
    description: &str,
    steps: &[String],
    trigger: Option<&str>,
) -> String {
    let mut md = String::new();
    md.push_str("---\n");
    md.push_str("schema: 1\n");
    md.push_str("kind: skill\n");
    md.push_str(&format!("name: {name}\n"));
    md.push_str(&format!("description: {description}\n"));
    if let Some(t) = trigger {
        md.push_str(&format!("trigger: {t}\n"));
    }
    md.push_str("contexto: \"Auto-generated from observed ephemeral pipeline\"\n");
    md.push_str("acionaveis: [\"on_demand\"]\n");
    md.push_str("required_tokens: [1]\n");
    md.push_str("provenance: hermes_created\n");
    md.push_str("sandbox_status: none\n");
    md.push_str("---\n\n");
    md.push_str("## Contexto\n\nGerada após recorrência do pipeline efêmero.\n\n");
    md.push_str(&format!("## Goal\n\n{description}\n\n"));
    md.push_str("## Acionaveis\n\n- on_demand\n\n");
    md.push_str("## Workflow\n");
    for (i, step) in steps.iter().enumerate() {
        md.push_str(&format!("{}. {step}\n", i + 1));
    }
    md.push_str("\n## Pre-Flight\n- [ ] Verify output matches expected format\n");
    md.push_str("## Success Criteria\n- [ ] All workflow steps completed\n");
    md.push_str("## Failure Policy\nReport failure and retry with corrected steps\n");
    md
}

pub fn maybe_auto_skill(normalized_key: &str, observer_hits: u32) -> Option<String> {
    let map = patterns().lock().ok()?;
    let pattern = map.get(normalized_key);
    let uses = pattern.map(|p| p.uses).unwrap_or(0);
    if observer_hits < skill_registry::AUTO_SKILL_MIN_HITS
        && uses < skill_registry::AUTO_SKILL_MIN_HITS
    {
        return None;
    }
    let skill_name = skill_name_from_key(normalized_key);
    let steps = workflow_steps_for(normalized_key);
    let description = pattern
        .map(|p| {
            if p.description.len() > 200 {
                p.description.chars().take(200).collect()
            } else {
                p.description.clone()
            }
        })
        .unwrap_or_else(|| normalized_key.replace('_', " "));
    Some(generate_skill_md(
        &skill_name,
        &description,
        &steps,
        Some(normalized_key),
    ))
}

pub fn pattern_uses(normalized_key: &str) -> u32 {
    patterns()
        .lock()
        .ok()
        .and_then(|m| m.get(normalized_key).map(|p| p.uses))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_skill_uses_observed_pipeline() {
        for i in 0..3 {
            record_observed_steps(
                "qual_a_temperatura",
                "qual a temperatura?",
                &generic_workflow_steps(),
            );
            assert_eq!(pattern_uses("qual_a_temperatura"), i + 1);
        }
        let md = maybe_auto_skill("qual_a_temperatura", 3).expect("skill md");
        assert!(md.contains("name: a_temperatura"));
        assert!(md.contains("synthesize_response"));
        assert!(md.contains("fetch_external"));
    }

    #[test]
    fn skill_name_from_key_strips_qual_prefix() {
        assert_eq!(skill_name_from_key("qual_a_temperatura"), "a_temperatura");
    }
}

//! Parser e verificação mínima de SKILL.md (contrato ADR-0052 subset — Onda 7h).

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedSkillMd {
    pub name: String,
    pub description: String,
    pub contexto: String,
    pub trigger: Option<String>,
    pub workflow: Vec<String>,
    pub raw: String,
}

const REQUIRED_FRONTMATTER: &[&str] = &[
    "schema: 1",
    "kind: skill",
];

const REQUIRED_SECTIONS: &[&str] = &[
    "## Contexto",
    "## Goal",
    "## Acionaveis",
    "## Workflow",
    "## Pre-Flight",
    "## Success Criteria",
    "## Failure Policy",
];

pub fn verify_skill_md(content: &str) -> Result<(), String> {
    let trimmed = content.trim();
    if !trimmed.starts_with("---") {
        return Err("SKILL.md: frontmatter ausente".into());
    }
    let parts: Vec<&str> = trimmed.splitn(3, "---").collect();
    if parts.len() < 3 {
        return Err("SKILL.md: frontmatter malformado".into());
    }
    let fm = parts[1];
    for req in REQUIRED_FRONTMATTER {
        if !fm.contains(req) {
            return Err(format!("SKILL.md: falta `{req}`"));
        }
    }
    if !fm.contains("name:") {
        return Err("SKILL.md: falta name".into());
    }
    let body = parts[2];
    for sec in REQUIRED_SECTIONS {
        if !body.contains(sec) {
            return Err(format!("SKILL.md: seção obrigatória ausente: {sec}"));
        }
    }
    Ok(())
}

pub fn parse_skill_md(content: &str) -> Result<ParsedSkillMd, String> {
    verify_skill_md(content)?;
    let trimmed = content.trim();
    let parts: Vec<&str> = trimmed.splitn(3, "---").collect();
    let fm = parts[1];
    let body = parts[2];

    let mut meta = BTreeMap::new();
    for line in fm.lines() {
        if let Some((k, v)) = line.split_once(':') {
            meta.insert(k.trim().to_string(), v.trim().trim_matches('"').to_string());
        }
    }

    let name = meta
        .get("name")
        .cloned()
        .filter(|n| !n.is_empty())
        .ok_or_else(|| "SKILL.md: name vazio".to_string())?;
    let description = meta
        .get("description")
        .cloned()
        .unwrap_or_else(|| name.clone());
    let contexto = meta
        .get("contexto")
        .cloned()
        .unwrap_or_else(|| "Auto-generated".into());
    let trigger = meta.get("trigger").cloned().filter(|t| !t.is_empty());

    let workflow = extract_workflow_steps(body);

    Ok(ParsedSkillMd {
        name,
        description,
        contexto,
        trigger,
        workflow,
        raw: content.to_string(),
    })
}

fn extract_workflow_steps(body: &str) -> Vec<String> {
    let mut steps = Vec::new();
    let mut in_workflow = false;
    for line in body.lines() {
        let t = line.trim();
        if t.starts_with("## Workflow") {
            in_workflow = true;
            continue;
        }
        if in_workflow && t.starts_with("## ") {
            break;
        }
        if in_workflow {
            if let Some(rest) = t.strip_prefix(|c: char| c.is_ascii_digit()) {
                let rest = rest.trim_start_matches('.').trim_start_matches(')').trim();
                if !rest.is_empty() {
                    steps.push(rest.to_string());
                }
            }
        }
    }
    if steps.is_empty() {
        steps.push("parse_intent".into());
        steps.push("execute_workflow".into());
        steps.push("format_response".into());
    }
    steps
}

pub fn persist_skill_md(name: &str, content: &str) -> Result<std::path::PathBuf, String> {
    let dir = super::dynamic::skills_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("mkdir skills: {e}"))?;
    let path = dir.join(format!("{name}.md"));
    std::fs::write(&path, content).map_err(|e| format!("write skill md: {e}"))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"---
schema: 1
kind: skill
name: weather_query
description: Consulta temperatura
contexto: Intent recorrente clima
acionaveis: ["on_demand"]
provenance: hermes_created
sandbox_status: none
---

## Contexto

Auto-generated.

## Goal

Consulta temperatura.

## Acionaveis

- on_demand

## Workflow
1. geolocate
2. fetch_weather_api
3. format_response_ptbr

## Pre-Flight
- [ ] API reachable

## Success Criteria
- [ ] Temperature returned

## Failure Policy
Report and retry
"#;

    #[test]
    fn parse_weather_skill() {
        let p = parse_skill_md(SAMPLE).expect("parse");
        assert_eq!(p.name, "weather_query");
        assert_eq!(p.workflow.len(), 3);
        assert!(p.workflow[1].contains("fetch"));
    }
}

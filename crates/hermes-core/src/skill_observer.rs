//! Observação de padrões de uso — recorrência → candidato a skill (ADR-010).

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

use skill_registry::AUTO_SKILL_MIN_HITS;

#[derive(Clone, Debug)]
pub struct TaskObservation {
    pub key: String,
    pub sample: String,
    pub hits: u32,
}

fn observations() -> &'static Mutex<BTreeMap<String, TaskObservation>> {
    static OBS: OnceLock<Mutex<BTreeMap<String, TaskObservation>>> = OnceLock::new();
    OBS.get_or_init(|| Mutex::new(BTreeMap::new()))
}

pub fn normalize_key(text: &str) -> String {
    let mut out = String::new();
    let mut prev_space = true;
    for c in text.chars().take(96) {
        let lc = c.to_ascii_lowercase();
        if lc.is_ascii_alphanumeric() {
            out.push(lc);
            prev_space = false;
        } else if !prev_space {
            out.push('_');
            prev_space = true;
        }
    }
    out.trim_matches('_').chars().take(48).collect()
}

pub fn observe_intent(text: &str) -> Option<TaskObservation> {
    let key = normalize_key(text);
    if key.is_empty() {
        return None;
    }
    let mut map = observations().lock().ok()?;
    let entry = map.entry(key.clone()).or_insert(TaskObservation {
        key: key.clone(),
        sample: text.chars().take(120).collect(),
        hits: 0,
    });
    entry.hits += 1;
    if entry.sample.is_empty() {
        entry.sample = text.chars().take(120).collect();
    }
    Some(entry.clone())
}

pub fn hits_for(text: &str) -> u32 {
    let key = normalize_key(text);
    observations()
        .lock()
        .ok()
        .and_then(|m| m.get(&key).map(|o| o.hits))
        .unwrap_or(0)
}

pub fn is_auto_skill_candidate(text: &str) -> bool {
    hits_for(text) >= AUTO_SKILL_MIN_HITS
}

pub fn top_observations(limit: usize) -> Vec<TaskObservation> {
    let mut list: Vec<TaskObservation> = observations()
        .lock()
        .ok()
        .map(|m| m.values().cloned().collect())
        .unwrap_or_default();
    list.sort_by(|a, b| b.hits.cmp(&a.hits));
    list.truncate(limit);
    list
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_collapses_spaces() {
        assert_eq!(normalize_key("  Que   Horas  "), "que_horas");
    }
}

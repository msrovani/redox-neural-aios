//! i18n minimal — Redox Neural AIOS v0.1 (pt-BR + en-US).

use std::collections::HashMap;
use std::sync::OnceLock;

const PT_BR: &str = include_str!("../../../locales/pt-BR.json");
const EN_US: &str = include_str!("../../../locales/en-US.json");

static CACHE: OnceLock<HashMap<String, HashMap<String, String>>> = OnceLock::new();

fn catalogs() -> &'static HashMap<String, HashMap<String, String>> {
    CACHE.get_or_init(|| {
        let mut map = HashMap::new();
        map.insert(
            "pt-BR".into(),
            serde_json::from_str(PT_BR).expect("pt-BR.json"),
        );
        map.insert(
            "en-US".into(),
            serde_json::from_str(EN_US).expect("en-US.json"),
        );
        map
    })
}

/// Locale ativo: `REDOX_LANG` → `language` em soul → pt-BR.
pub fn active_locale() -> String {
    std::env::var("REDOX_LANG")
        .or_else(|_| std::env::var("REDOX_SOUL_LANGUAGE"))
        .unwrap_or_else(|_| "pt-BR".into())
}

/// Traduz chave; fallback en-US → chave literal.
pub fn t(key: &str) -> String {
    let locale = active_locale();
    let catalogs = catalogs();
    if let Some(cat) = catalogs.get(&locale) {
        if let Some(val) = cat.get(key) {
            return val.clone();
        }
    }
    if let Some(cat) = catalogs.get("en-US") {
        if let Some(val) = cat.get(key) {
            return val.clone();
        }
    }
    key.to_string()
}

/// Traduz com interpolação `{name}` → valor.
pub fn t_fmt(key: &str, vars: &[(&str, &str)]) -> String {
    let mut out = t(key);
    for (k, v) in vars {
        out = out.replace(&format!("{{{k}}}"), v);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pt_br_greeting() {
        std::env::set_var("REDOX_LANG", "pt-BR");
        assert!(t("jarbas.greeting").contains("JARBAS"));
    }

    #[test]
    fn en_us_greeting() {
        std::env::set_var("REDOX_LANG", "en-US");
        assert!(t("jarbas.greeting").contains("JARBAS"));
    }
}

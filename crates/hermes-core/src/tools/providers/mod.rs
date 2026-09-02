//! Provedores HTTP plugáveis para `fetch_external` (Onda 7i — genérico).
//! Domínios concretos (clima, câmbio, etc.) registram implementações aqui.

mod open_meteo;

use std::sync::{Mutex, OnceLock};

pub trait FetchProvider: Send + Sync {
    fn id(&self) -> &'static str;
    /// 0.0 = não aplicável; 1.0 = match forte.
    fn match_score(&self, intent: &str, context: &str) -> f32;
    fn fetch(&self, intent: &str, context: &str) -> Result<String, String>;
}

struct ProviderRegistry {
    providers: Vec<Box<dyn FetchProvider>>,
}

impl ProviderRegistry {
    fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    fn register(&mut self, provider: Box<dyn FetchProvider>) {
        self.providers.push(provider);
    }

    fn best_match(&self, intent: &str, context: &str) -> Option<(&dyn FetchProvider, f32)> {
        self.providers
            .iter()
            .map(|p| (p.as_ref(), p.match_score(intent, context)))
            .filter(|(_, score)| *score > 0.15)
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(p, s)| (p as &dyn FetchProvider, s))
    }
}

fn registry() -> &'static Mutex<ProviderRegistry> {
    static REG: OnceLock<Mutex<ProviderRegistry>> = OnceLock::new();
    REG.get_or_init(|| {
        let mut reg = ProviderRegistry::new();
        load_env_providers(&mut reg);
        Mutex::new(reg)
    })
}

fn load_env_providers(reg: &mut ProviderRegistry) {
    let list = std::env::var("REDOX_TOOLS_PROVIDERS").unwrap_or_default();
    let enable_all = list.eq_ignore_ascii_case("all");
    let ids: Vec<String> = if enable_all {
        vec!["open_meteo".into()]
    } else {
        list.split(',')
            .map(|s| s.trim().to_ascii_lowercase())
            .filter(|s| !s.is_empty())
            .collect()
    };

    for id in ids {
        match id.as_str() {
            "open_meteo" | "weather" | "clima" => {
                reg.register(Box::new(open_meteo::OpenMeteoProvider));
            }
            other => eprintln!("[hermes] provider desconhecido ignorado: {other}"),
        }
    }
}

pub fn tools_net_enabled() -> bool {
    std::env::var("REDOX_TOOLS_NET")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

pub fn fetch_via_providers(intent: &str, context: &str) -> Result<String, String> {
    if !tools_net_enabled() {
        return Ok(format!(
            "stub: fetch_external disabled (REDOX_TOOLS_NET=0) intent={intent}"
        ));
    }

    let reg = registry().lock().map_err(|_| "provider registry lock")?;
    let Some((provider, score)) = reg.best_match(intent, context) else {
        return Ok(format!(
            "no_provider_match: intent={intent}\n\
             context={context}\n\
             configure REDOX_TOOLS_NET=1 e REDOX_TOOLS_PROVIDERS=open_meteo (ou all)"
        ));
    };

    let body = provider.fetch(intent, context)?;
    Ok(format!(
        "provider={} score={:.2}\n{body}",
        provider.id(),
        score
    ))
}

pub fn registered_provider_ids() -> Vec<&'static str> {
    registry()
        .lock()
        .ok()
        .map(|reg| reg.providers.iter().map(|p| p.id()).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_meteo_scores_weather_intent() {
        let p = open_meteo::OpenMeteoProvider;
        assert!(p.match_score("qual a temperatura em sp?", "") > 0.3);
        assert!(p.match_score("que horas são", "") < 0.2);
    }
}

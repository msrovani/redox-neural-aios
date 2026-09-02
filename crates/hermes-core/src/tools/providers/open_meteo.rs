//! Exemplo de provider plugável — clima via Open-Meteo (sem API key).
//! Ilustra o processo ADR-010; não acopla o pipeline efêmero a este domínio.

use super::FetchProvider;

const WEATHER_TERMS: &[&str] = &[
    "temperatura",
    "tempo",
    "clima",
    "weather",
    "chuva",
    "previsao",
    "previsão",
    "graus",
    "frio",
    "calor",
    "umidade",
];

const DEFAULT_LAT: f64 = -23.55;
const DEFAULT_LON: f64 = -46.63;

pub struct OpenMeteoProvider;

impl FetchProvider for OpenMeteoProvider {
    fn id(&self) -> &'static str {
        "open_meteo"
    }

    fn match_score(&self, intent: &str, context: &str) -> f32 {
        let blob = format!("{intent} {context}").to_ascii_lowercase();
        let hits = WEATHER_TERMS.iter().filter(|t| blob.contains(*t)).count();
        if hits == 0 {
            return 0.0;
        }
        (hits as f32 * 0.35).min(1.0)
    }

    fn fetch(&self, intent: &str, context: &str) -> Result<String, String> {
        let (lat, lon, label) = infer_coords(intent, context);
        let url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={lat}&longitude={lon}\
             &current=temperature_2m,relative_humidity_2m,weather_code&timezone=auto"
        );
        let response = ureq::get(&url)
            .call()
            .map_err(|e| format!("open_meteo HTTP: {e}"))?;
        let json: serde_json::Value = response
            .into_json()
            .map_err(|e| format!("open_meteo JSON: {e}"))?;
        let temp = json
            .pointer("/current/temperature_2m")
            .and_then(|v| v.as_f64())
            .ok_or("open_meteo: temperature_2m ausente")?;
        let humidity = json
            .pointer("/current/relative_humidity_2m")
            .and_then(|v| v.as_f64());
        let code = json
            .pointer("/current/weather_code")
            .and_then(|v| v.as_i64());

        let mut out = format!("local={label} lat={lat} lon={lon} temp_c={temp:.1}");
        if let Some(h) = humidity {
            out.push_str(&format!(" humidity={h:.0}%"));
        }
        if let Some(c) = code {
            out.push_str(&format!(" wmo={c}"));
        }
        Ok(out)
    }
}

fn infer_coords(intent: &str, context: &str) -> (f64, f64, &'static str) {
    let blob = format!("{intent} {context}").to_ascii_lowercase();
    if blob.contains("rio de janeiro") || blob.contains(" no rio") || blob.contains(" rio ") {
        return (-22.91, -43.20, "Rio de Janeiro");
    }
    if blob.contains("brasilia") || blob.contains("brasília") {
        return (-15.79, -47.88, "Brasília");
    }
    if blob.contains(" sp") || blob.contains("sao paulo") || blob.contains("são paulo") {
        return (DEFAULT_LAT, DEFAULT_LON, "São Paulo");
    }
    (DEFAULT_LAT, DEFAULT_LON, "default(São Paulo)")
}

//! Protocolo remoto do EventBus (eventd TCP).

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct RemoteRequest {
    pub cmd: String,
    #[serde(default)]
    pub topic: Option<String>,
    #[serde(default)]
    pub payload: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RemoteResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl RemoteResponse {
    pub fn ok() -> Self {
        Self {
            ok: true,
            error: None,
        }
    }

    pub fn fail(msg: impl Into<String>) -> Self {
        Self {
            ok: false,
            error: Some(msg.into()),
        }
    }
}

pub fn handle_remote(
    bus: &crate::EventBus,
    line: &str,
) -> String {
    let req: RemoteRequest = match serde_json::from_str(line.trim()) {
        Ok(r) => r,
        Err(e) => {
            return serde_json::to_string(&RemoteResponse::fail(format!("json: {e}")))
                .unwrap_or_default();
        }
    };

    let resp = match req.cmd.as_str() {
        "publish" => {
            let topic = req.topic.unwrap_or_default();
            let payload = req.payload.unwrap_or_default();
            let token = crate::CapabilityToken::system("eventd", "remote_publish");
            match bus.publish(crate::Event::new(topic.clone(), payload.clone(), token)) {
                Ok(()) => {
                    let _ = crate::chan::publish_file(&topic, &payload);
                    RemoteResponse::ok()
                }
                Err(e) => RemoteResponse::fail(e),
            }
        }
        "ping" => RemoteResponse::ok(),
        other => RemoteResponse::fail(format!("cmd desconhecido: {other}")),
    };

    serde_json::to_string(&resp).unwrap_or_else(|_| {
        r#"{"ok":false,"error":"serialize"}"#.to_string()
    })
}

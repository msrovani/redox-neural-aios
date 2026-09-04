//! Definições e dispatch das tools MCP.

use serde_json::{json, Value};

use crate::bridge::{self, memory_client};
use crate::protocol::tool_result;

pub fn listed_tools() -> Value {
    json!([
        {
            "name": "health",
            "description": "Saúde do stack AIOS. view=status (sgdb+hermes ping) ou view=backends (honesty tiers).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "view": {"type": "string", "enum": ["status", "backends"], "default": "status"}
                }
            },
            "annotations": {"readOnlyHint": true}
        },
        {
            "name": "remember",
            "description": "Grava memória via sgdbd. text obrigatório; scope opcional (default mcp).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "text": {"type": "string"},
                    "scope": {"type": "string"}
                },
                "required": ["text"]
            },
            "annotations": {"destructiveHint": true, "idempotentHint": true}
        },
        {
            "name": "recall",
            "description": "Recall lexical via sgdbd. query obrigatório; scope/k opcionais.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": {"type": "string"},
                    "scope": {"type": "string"},
                    "k": {"type": "integer", "minimum": 1, "maximum": 20, "default": 5}
                },
                "required": ["query"]
            },
            "annotations": {"readOnlyHint": true}
        },
        {
            "name": "hermes_intent",
            "description": "Envia intent ao hermesd (texto livre ou /skills, /factory, /lifecycle, …).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "text": {"type": "string"}
                },
                "required": ["text"]
            },
            "annotations": {"readOnlyHint": false}
        },
        {
            "name": "caps",
            "description": "CapGate Redox. op=list|probe|ns|bootstrap (default list). role opcional p/ ns/probe.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "op": {"type": "string", "enum": ["list", "probe", "ns", "bootstrap"], "default": "list"},
                    "role": {"type": "string", "default": "hermes"}
                }
            },
            "annotations": {"readOnlyHint": true}
        },
        {
            "name": "backends",
            "description": "Relatório BackendReport (memory/event/cortex/stt/tts/caps) via hermesd.",
            "inputSchema": {"type": "object", "properties": {}},
            "annotations": {"readOnlyHint": true}
        }
    ])
}

pub fn call_tool(name: &str, args: &Value) -> Value {
    match name {
        "health" => tool_health(args),
        "remember" => tool_remember(args),
        "recall" => tool_recall(args),
        "hermes_intent" | "intent" | "user_intent" => tool_intent(args),
        "caps" => tool_caps(args),
        "backends" => tool_backends(),
        other => tool_result(
            format!("tool desconhecida: {other}"),
            json!({"error": "unknown_tool", "name": other}),
            true,
        ),
    }
}

fn tool_health(args: &Value) -> Value {
    let view = args["view"].as_str().unwrap_or("status");
    if view == "backends" {
        return tool_backends();
    }

    let mem = memory_client();
    let mut parts = Vec::new();
    let mut structured = json!({"view": "status"});

    match mem.health() {
        Ok(h) => {
            parts.push(format!("sgdbd: ok {h}"));
            structured["sgdb"] = json!({"ok": true, "health": h});
        }
        Err(e) => {
            parts.push(format!("sgdbd: FAIL {e}"));
            structured["sgdb"] = json!({"ok": false, "error": e});
        }
    }

    match bridge::hermes_ping() {
        Ok(p) => {
            parts.push(format!("hermesd: ok {p}"));
            structured["hermes"] = json!({"ok": true, "ping": p});
        }
        Err(e) => {
            parts.push(format!("hermesd: FAIL {e}"));
            structured["hermes"] = json!({"ok": false, "error": e});
        }
    }

    let ok = structured["sgdb"]["ok"].as_bool() == Some(true)
        && structured["hermes"]["ok"].as_bool() == Some(true);
    structured["ok"] = json!(ok);
    tool_result(parts.join("\n"), structured, !ok)
}

fn tool_remember(args: &Value) -> Value {
    let Some(text) = args["text"].as_str().filter(|s| !s.is_empty()) else {
        return tool_result(
            "remember exige text",
            json!({"error": "missing_text"}),
            true,
        );
    };
    let scope = args["scope"].as_str().unwrap_or("mcp");
    match memory_client().remember(text, Some(scope)) {
        Ok(r) => tool_result(
            format!("remember ok scope={scope}"),
            json!({"ok": true, "scope": scope, "result": r}),
            false,
        ),
        Err(e) => tool_result(e.clone(), json!({"ok": false, "error": e}), true),
    }
}

fn tool_recall(args: &Value) -> Value {
    let Some(query) = args["query"].as_str().filter(|s| !s.is_empty()) else {
        return tool_result(
            "recall exige query",
            json!({"error": "missing_query"}),
            true,
        );
    };
    let scope = args["scope"].as_str();
    let k = args["k"].as_u64().unwrap_or(5) as usize;
    match memory_client().recall(query, scope, Some(k)) {
        Ok(r) => {
            let text = if let Some(hits) = r.get("hits").and_then(|h| h.as_array()) {
                let lines: Vec<&str> = hits
                    .iter()
                    .filter_map(|h| h.get("text").and_then(|t| t.as_str()))
                    .collect();
                if lines.is_empty() {
                    "(sem hits)".into()
                } else {
                    lines.join("\n")
                }
            } else {
                r.to_string()
            };
            tool_result(text, json!({"ok": true, "result": r}), false)
        }
        Err(e) => tool_result(e.clone(), json!({"ok": false, "error": e}), true),
    }
}

fn tool_intent(args: &Value) -> Value {
    let Some(text) = args["text"].as_str().filter(|s| !s.is_empty()) else {
        return tool_result(
            "hermes_intent exige text",
            json!({"error": "missing_text"}),
            true,
        );
    };
    match bridge::hermes_intent(text) {
        Ok(raw) => {
            let response = raw
                .get("response")
                .and_then(|v| v.as_str())
                .or_else(|| raw.get("result").and_then(|v| v.as_str()))
                .unwrap_or("")
                .to_string();
            let display = if response.is_empty() {
                raw.to_string()
            } else {
                response
            };
            tool_result(display, json!({"ok": true, "raw": raw}), false)
        }
        Err(e) => tool_result(e.clone(), json!({"ok": false, "error": e}), true),
    }
}

fn tool_caps(args: &Value) -> Value {
    let op = args["op"].as_str().unwrap_or("list");
    let role = args["role"].as_str().unwrap_or("hermes");
    let intent = match op {
        "probe" => format!("/caps probe {role}"),
        "ns" | "namespace" => format!("/caps ns {role}"),
        "bootstrap" => "/caps bootstrap".into(),
        _ => "/caps list".into(),
    };
    tool_intent(&json!({"text": intent}))
}

fn tool_backends() -> Value {
    match bridge::hermes_backends() {
        Ok(raw) => {
            let result = raw.get("result").cloned().unwrap_or(raw.clone());
            tool_result(result.to_string(), json!({"ok": true, "backends": result}), false)
        }
        Err(e) => {
            // Fallback local se hermesd offline
            let local = agent_core::collect_stack_backends();
            let val = serde_json::to_value(&local).unwrap_or(json!([]));
            tool_result(
                format!("hermesd offline ({e}); backends locais:\n{val}"),
                json!({"ok": false, "error": e, "backends": val, "source": "local"}),
                false,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_six_tools() {
        let tools = listed_tools();
        let arr = tools.as_array().expect("array");
        assert_eq!(arr.len(), crate::MCP_TOOL_COUNT);
        let names: Vec<&str> = arr
            .iter()
            .filter_map(|t| t["name"].as_str())
            .collect();
        assert!(names.contains(&"health"));
        assert!(names.contains(&"hermes_intent"));
        assert!(names.contains(&"caps"));
    }

    #[test]
    fn unknown_tool_is_error() {
        let out = call_tool("nope", &json!({}));
        assert_eq!(out["isError"], true);
    }
}

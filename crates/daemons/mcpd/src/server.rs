//! Dispatcher JSON-RPC MCP (stdio / TCP linha).

use serde_json::{json, Value};

use crate::protocol::{
    err_rpc, initialize_result, listed_resources, ok_result, session_packet, DOCTRINE,
};
use crate::tools::{call_tool, listed_tools};

/// Processa uma mensagem MCP. `None` = notificação (sem resposta).
pub fn handle_message(msg: &Value) -> Option<Value> {
    let id = msg.get("id").cloned().unwrap_or(Value::Null);
    let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");

    // Request sem method → erro
    if method.is_empty() {
        if id.is_null() {
            return None;
        }
        return Some(err_rpc(&id, -32600, "Invalid Request"));
    }

    match method {
        "initialize" => Some(ok_result(&id, initialize_result())),
        "notifications/initialized" | "notifications/cancelled" | "notifications/progress" => {
            None
        }
        "ping" => Some(ok_result(&id, json!({}))),
        "tools/list" => Some(ok_result(&id, json!({"tools": listed_tools()}))),
        "tools/call" | "tools.call" => {
            let params = msg.get("params").cloned().unwrap_or(json!({}));
            let name = params["name"].as_str().unwrap_or("");
            if name.is_empty() {
                return Some(err_rpc(&id, -32602, "tools/call exige params.name"));
            }
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            let result = call_tool(name, &args);
            Some(ok_result(&id, result))
        }
        "resources/list" => Some(ok_result(&id, json!({"resources": listed_resources()}))),
        "resources/read" => {
            let uri = msg
                .pointer("/params/uri")
                .and_then(|u| u.as_str())
                .unwrap_or("");
            match uri {
                "aios://doctrine" => Some(ok_result(
                    &id,
                    json!({
                        "contents": [{
                            "uri": uri,
                            "mimeType": "text/plain",
                            "text": DOCTRINE
                        }]
                    }),
                )),
                "aios://session" => Some(ok_result(
                    &id,
                    json!({
                        "contents": [{
                            "uri": uri,
                            "mimeType": "application/json",
                            "text": session_packet().to_string()
                        }]
                    }),
                )),
                _ => Some(err_rpc(&id, -32002, format!("resource desconhecido: {uri}"))),
            }
        }
        "server/discover" => {
            // Clients modernos: -32601 → fallback initialize (paridade neural-sgdb).
            Some(err_rpc(&id, -32601, "Method not found"))
        }
        _ => Some(err_rpc(&id, -32601, format!("Method not found: {method}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_handshake() {
        let msg = json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}});
        let out = handle_message(&msg).unwrap();
        assert_eq!(out["result"]["protocolVersion"], "2025-11-25");
        assert_eq!(out["result"]["serverInfo"]["name"], "redox-aios");
        assert_eq!(out["result"]["serverInfo"]["mcp_tool_count"], 6);
    }

    #[test]
    fn tools_list_ok() {
        let msg = json!({"jsonrpc":"2.0","id":2,"method":"tools/list"});
        let out = handle_message(&msg).unwrap();
        let tools = out["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 6);
    }

    #[test]
    fn notification_no_response() {
        let msg = json!({"jsonrpc":"2.0","method":"notifications/initialized"});
        assert!(handle_message(&msg).is_none());
    }

    #[test]
    fn session_resource() {
        let msg = json!({
            "jsonrpc":"2.0","id":3,"method":"resources/read",
            "params":{"uri":"aios://session"}
        });
        let out = handle_message(&msg).unwrap();
        let text = out["result"]["contents"][0]["text"].as_str().unwrap();
        assert!(text.contains("7742"));
    }
}

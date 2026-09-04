//! Constantes e helpers JSON-RPC MCP (stdio, uma mensagem por linha).

use serde_json::{json, Value};

pub const MCP_PROTOCOL_VERSION: &str = "2025-11-25";
pub const MCP_SERVER_NAME: &str = "redox-aios";
pub const MCP_TOOL_COUNT: usize = 6;
pub const MCP_CONTRACT_VERSION: &str = "0.1.0";

pub const DOCTRINE: &str = r#"Redox Neural AIOS MCP — shim sobre daemons userspace.

Tools: health, remember, recall, hermes_intent, caps, backends.
Memória → sgdbd (REDOX_SGDB_SOCKET, default 127.0.0.1:7741).
Orquestração → hermesd (REDOX_HERMES_SOCKET, default 127.0.0.1:7742).
Caps = CapGate userspace (/caps); backends = honesty ADR-001.
Suba a stack: tools/start-stack.ps1. Resource aios://session no cold-start.
"#;

pub fn ok_result(id: &Value, result: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "result": result})
}

pub fn err_rpc(id: &Value, code: i32, message: impl Into<String>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {"code": code, "message": message.into()}
    })
}

pub fn tool_result(text: impl Into<String>, structured: Value, is_error: bool) -> Value {
    json!({
        "content": [{"type": "text", "text": text.into()}],
        "structuredContent": structured,
        "isError": is_error
    })
}

pub fn initialize_result() -> Value {
    json!({
        "protocolVersion": MCP_PROTOCOL_VERSION,
        "capabilities": {"tools": {}, "resources": {}},
        "instructions": DOCTRINE,
        "serverInfo": {
            "name": MCP_SERVER_NAME,
            "version": env!("CARGO_PKG_VERSION"),
            "title": "Redox Neural AIOS MCP",
            "mcp_contract_version": MCP_CONTRACT_VERSION,
            "mcp_tool_count": MCP_TOOL_COUNT
        }
    })
}

pub fn listed_resources() -> Value {
    json!([
        {
            "uri": "aios://doctrine",
            "name": "aios-doctrine",
            "mimeType": "text/plain",
            "description": "Como usar o MCP AIOS (igual initialize.instructions)"
        },
        {
            "uri": "aios://session",
            "name": "aios-session",
            "mimeType": "application/json",
            "description": "Cold-start: sockets, tools, stack ports"
        }
    ])
}

pub fn session_packet() -> Value {
    json!({
        "server": MCP_SERVER_NAME,
        "contract": MCP_CONTRACT_VERSION,
        "tools": MCP_TOOL_COUNT,
        "sgdb": crate::bridge::sgdb_addr(),
        "hermes": crate::bridge::hermes_addr(),
        "ports": {
            "eventd": 7740,
            "sgdbd": 7741,
            "hermesd": 7742,
            "cortexd": 7743,
            "voiced": 7744,
            "jarbasd": 7745,
            "mcpd_tcp": 7746
        }
    })
}

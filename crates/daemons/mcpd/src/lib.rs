//! mcpd — MCP server do Redox Neural AIOS (ADR-011).
//! Shim JSON-RPC 2.0 (stdio) sobre sgdbd (:7741) e hermesd (:7742).

pub mod bridge;
pub mod protocol;
pub mod server;
pub mod tools;

pub use protocol::{MCP_PROTOCOL_VERSION, MCP_SERVER_NAME, MCP_TOOL_COUNT};
pub use server::handle_message;

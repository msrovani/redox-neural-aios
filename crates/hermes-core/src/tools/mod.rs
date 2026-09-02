//! Tool registry — extensível para degrau 0 (efêmera) e degrau 1 (SKILL.md workflow).
//! Provedores concretos (clima, câmbio, etc.) plugam em `providers`.

mod providers;
mod registry;

pub use providers::{fetch_via_providers, registered_provider_ids, tools_net_enabled};
pub use registry::{default_tool_registry, ToolContext, ToolRegistry, ToolResult};

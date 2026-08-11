//! Local-first agent primitives for Thinking Computer.

pub mod agent;
pub mod bridge;
pub mod config;
pub mod memory;
pub mod model;
pub mod permissions;
pub mod plugin;
pub mod providers;
pub mod schedule;
pub mod tools;

pub use agent::Agent;
pub use bridge::{ChannelKind, InboundMessage, normalize as normalize_bridge_message, sender_is_trusted};
pub use config::{AppConfig, ChannelConfig, ProviderConfig, ProviderKind, ResolvedProvider};
pub use memory::{AgentMemory, KnowledgeRecord, SessionStore, VmCapabilityProfile};
pub use model::{ChatMessage, Role, ToolCall, ToolDefinition};
pub use permissions::{Approval, ApprovalRequest, Capability, PermissionPolicy};
pub use plugin::{PluginManifest, PluginTool};
pub use schedule::{ScheduleStore, ScheduledTask};
pub use tools::ToolExecutor;

/// Describes the host platform through the C++ bridge while keeping the rest of
/// the program in safe Rust.
pub fn system_summary() -> String {
    format!("{} ({})", tc_system_bridge::platform_name(), tc_system_bridge::cpu_architecture())
}

//! Local-first agent primitives for Thinking Computer.

pub mod agent;
pub mod bridge;
pub mod config;
pub mod delegation;
pub mod memory;
pub mod model;
pub mod outbound;
pub mod permissions;
pub mod plugin;
pub mod providers;
pub mod schedule;
pub mod skills;
pub mod tools;
pub mod webhook;

pub use agent::Agent;
pub use bridge::{
    normalize as normalize_bridge_message, sender_is_trusted, ChannelKind, InboundMessage,
    PairedSender, PairingStore,
};
pub use config::{
    AppConfig, ChannelConfig, ProviderConfig, ProviderKind, ResolvedProvider, ResolvedService,
    ServiceConfig,
};
pub use delegation::{DelegationPolicy, ExpertResult, ExpertStatus, ExpertTask};
pub use memory::{AgentMemory, KnowledgeRecord, SessionStore, VmCapabilityProfile};
pub use model::{ChatMessage, Role, ToolCall, ToolDefinition};
pub use outbound::{
    recipient_is_trusted, send_message as send_outbound_message, HttpOutboundTransport,
    OutboundDelivery, OutboundTransport,
};
pub use permissions::{Approval, ApprovalRequest, Capability, PermissionPolicy};
pub use plugin::{PluginManifest, PluginStore, PluginTool};
pub use schedule::{RegistrationTarget, ScheduleStore, ScheduledTask};
pub use skills::{SkillManifest, SkillStore};
pub use tools::ToolExecutor;
pub use webhook::{request_fingerprint, verify_discord_ed25519, verify_hmac_sha256, ReplayGuard};

/// Describes the host platform through the C++ bridge while keeping the rest of
/// the program in safe Rust.
pub fn system_summary() -> String {
    format!(
        "{} ({})",
        tc_system_bridge::platform_name(),
        tc_system_bridge::cpu_architecture()
    )
}

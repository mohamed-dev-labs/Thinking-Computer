use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    ReadFile,
    WriteFile,
    WriteMemory,
    WebSearch,
    Shell,
    InspectSystem,
    InstallPackage,
    GenerateMedia,
    Plugin(String),
    BridgeDispatch,
}

#[derive(Clone, Debug)]
pub struct ApprovalRequest {
    pub capability: Capability,
    pub summary: String,
}

pub trait Approval: Send + Sync {
    fn approve(&self, request: &ApprovalRequest) -> Result<bool>;
}

#[derive(Clone, Debug, Default)]
pub struct PermissionPolicy {
    always_allowed: BTreeSet<Capability>,
    pub assume_yes: bool,
}

impl PermissionPolicy {
    pub fn with_read_access() -> Self {
        let mut policy = Self::default();
        policy.always_allowed.insert(Capability::ReadFile);
        policy
    }
    pub fn needs_confirmation(&self, capability: &Capability) -> bool {
        !self.assume_yes && !self.always_allowed.contains(capability)
    }
}

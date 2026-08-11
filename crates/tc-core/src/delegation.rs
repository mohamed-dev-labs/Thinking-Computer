use crate::permissions::Capability;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ExpertTask {
    pub id: String,
    pub title: String,
    pub prompt: String,
    pub requested_capabilities: Vec<Capability>,
    pub max_steps: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExpertStatus {
    Completed,
    Denied,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ExpertResult {
    pub task_id: String,
    pub status: ExpertStatus,
    pub output: String,
    pub inherited_capabilities: Vec<Capability>,
    pub completed_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct DelegationPolicy {
    allowed_capabilities: BTreeSet<Capability>,
    pub max_subagents: usize,
    pub max_steps_per_subagent: u8,
}

impl DelegationPolicy {
    pub fn inherited_from(parent: impl IntoIterator<Item = Capability>) -> Self {
        Self {
            allowed_capabilities: parent.into_iter().collect(),
            max_subagents: 4,
            max_steps_per_subagent: 12,
        }
    }

    pub fn validate(&self, task: &ExpertTask, active_subagents: usize) -> Result<()> {
        if task.id.trim().is_empty()
            || task.title.trim().is_empty()
            || task.prompt.trim().is_empty()
        {
            anyhow::bail!("expert task id, title, and prompt are required");
        }
        if task.prompt.chars().count() > 8_000 {
            anyhow::bail!("expert task prompt exceeds the 8,000-character limit");
        }
        if task.max_steps == 0 || task.max_steps > self.max_steps_per_subagent {
            anyhow::bail!("expert task step budget exceeds the inherited delegation policy");
        }
        if active_subagents >= self.max_subagents {
            anyhow::bail!("expert task denied: maximum concurrent sub-agents reached");
        }
        if let Some(capability) = task
            .requested_capabilities
            .iter()
            .find(|capability| !self.allowed_capabilities.contains(*capability))
        {
            anyhow::bail!(
                "expert task requested a capability not held by its parent: {capability:?}"
            );
        }
        Ok(())
    }

    pub fn delegate<F>(&self, task: ExpertTask, active_subagents: usize, run: F) -> ExpertResult
    where
        F: FnOnce(&ExpertTask) -> Result<String>,
    {
        let inherited_capabilities = task.requested_capabilities.clone();
        match self
            .validate(&task, active_subagents)
            .and_then(|_| run(&task))
        {
            Ok(output) => ExpertResult {
                task_id: task.id,
                status: ExpertStatus::Completed,
                output,
                inherited_capabilities,
                completed_at: Utc::now(),
            },
            Err(error) => ExpertResult {
                task_id: task.id,
                status: ExpertStatus::Denied,
                output: error.to_string(),
                inherited_capabilities,
                completed_at: Utc::now(),
            },
        }
    }
}

impl ExpertTask {
    pub fn new(title: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title: title.into(),
            prompt: prompt.into(),
            requested_capabilities: Vec::new(),
            max_steps: 4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delegates_typed_result_with_inherited_permissions() {
        let policy = DelegationPolicy::inherited_from([Capability::WebSearch]);
        let mut task = ExpertTask::new("research", "Find public sources.");
        task.requested_capabilities = vec![Capability::WebSearch];
        let result = policy.delegate(task, 0, |_| Ok("source list".into()));
        assert_eq!(result.status, ExpertStatus::Completed);
        assert_eq!(result.inherited_capabilities, vec![Capability::WebSearch]);
    }

    #[test]
    fn denies_capabilities_not_held_by_parent() {
        let policy = DelegationPolicy::inherited_from([Capability::ReadFile]);
        let mut task = ExpertTask::new("shell", "Run a command.");
        task.requested_capabilities = vec![Capability::Shell];
        let result = policy.delegate(task, 0, |_| Ok("should not run".into()));
        assert_eq!(result.status, ExpertStatus::Denied);
        assert!(result.output.contains("not held by its parent"));
    }
}

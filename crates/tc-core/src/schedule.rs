use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{env, fs, path::{Path, PathBuf}};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ScheduledTask {
    pub id: String,
    pub name: String,
    pub cron: String,
    pub prompt: String,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct ScheduleStore { path: PathBuf }

impl ScheduleStore {
    pub fn local() -> Result<Self> { Self::in_directory(state_root()) }
    pub fn in_directory(root: impl AsRef<Path>) -> Result<Self> {
        let directory = root.as_ref().join("schedules");
        fs::create_dir_all(&directory).with_context(|| format!("failed to create schedule directory {}", directory.display()))?;
        Ok(Self { path: directory.join("tasks.json") })
    }
    pub fn list(&self) -> Result<Vec<ScheduledTask>> {
        if !self.path.exists() { return Ok(Vec::new()); }
        Ok(serde_json::from_slice(&fs::read(&self.path)?)?)
    }
    pub fn add(&self, name: &str, cron: &str, prompt: &str, provider: Option<String>, model: Option<String>) -> Result<ScheduledTask> {
        validate_cron(cron)?;
        if name.trim().is_empty() || prompt.trim().is_empty() { anyhow::bail!("scheduled task name and prompt must not be empty"); }
        let mut tasks = self.list()?;
        if tasks.iter().any(|task| task.name == name) { anyhow::bail!("a scheduled task named {name} already exists"); }
        let task = ScheduledTask { id: Uuid::new_v4().to_string(), name: name.to_string(), cron: cron.to_string(), prompt: prompt.to_string(), provider, model, enabled: true, created_at: Utc::now() };
        tasks.push(task.clone());
        self.save(&tasks)?;
        Ok(task)
    }
    pub fn remove(&self, name: &str) -> Result<bool> {
        let mut tasks = self.list()?;
        let previous = tasks.len();
        tasks.retain(|task| task.name != name);
        self.save(&tasks)?;
        Ok(previous != tasks.len())
    }
    pub fn cron_command(&self, task: &ScheduledTask) -> String {
        let mut command = format!("thinking-computer chat --session schedule-{}", task.id);
        if let Some(provider) = &task.provider { command.push_str(&format!(" --provider {}", shell_quote(provider))); }
        if let Some(model) = &task.model { command.push_str(&format!(" --model {}", shell_quote(model))); }
        command.push(' ');
        command.push_str(&shell_quote(&task.prompt));
        format!("{} {} # thinking-computer:{}", task.cron, command, task.name)
    }
    fn save(&self, tasks: &[ScheduledTask]) -> Result<()> { fs::write(&self.path, serde_json::to_vec_pretty(tasks)?).context("failed to save scheduled tasks") }
}

pub fn validate_cron(value: &str) -> Result<()> {
    let fields: Vec<_> = value.split_whitespace().collect();
    if fields.len() != 5 || fields.iter().any(|field| field.is_empty() || !field.chars().all(|ch| ch.is_ascii_digit() || matches!(ch, '*' | '/' | ',' | '-' | '?'))) {
        anyhow::bail!("cron must contain five standard fields using digits, *, /, comma, dash, or ?");
    }
    Ok(())
}

fn shell_quote(value: &str) -> String { format!("'{}'", value.replace('\'', "'\\''")) }
fn state_root() -> PathBuf { env::var("TC_HOME").map(PathBuf::from).unwrap_or_else(|_| dirs::data_local_dir().unwrap_or_else(|| PathBuf::from(".")).join("thinking-computer")) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn persists_a_validated_task_and_exports_a_cron_line() {
        let temp = tempfile::tempdir().unwrap();
        let store = ScheduleStore::in_directory(temp.path()).unwrap();
        let task = store.add("daily-note", "0 9 * * *", "write a concise note", Some("ollama".into()), None).unwrap();
        assert!(store.cron_command(&task).contains("thinking-computer:daily-note"));
        assert_eq!(store.list().unwrap().len(), 1);
    }
}

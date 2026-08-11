use crate::model::ChatMessage;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{env, fs::{self, OpenOptions}, io::Write, path::{Path, PathBuf}};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SessionEntry { pub timestamp: DateTime<Utc>, pub message: ChatMessage }

#[derive(Clone, Debug)]
pub struct SessionStore { id: String, path: PathBuf }

impl SessionStore {
    pub fn local(session_id: Option<String>) -> Result<Self> {
        let root = env::var("TC_HOME").map(PathBuf::from).unwrap_or_else(|_| dirs::data_local_dir().unwrap_or_else(|| PathBuf::from(".")).join("thinking-computer"));
        Self::in_directory(root, session_id)
    }
    pub fn in_directory(root: impl AsRef<Path>, session_id: Option<String>) -> Result<Self> {
        let id = session_id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let directory = root.as_ref().join("sessions");
        fs::create_dir_all(&directory).with_context(|| format!("failed to create local session directory {}", directory.display()))?;
        Ok(Self { id: id.clone(), path: directory.join(format!("{id}.jsonl")) })
    }
    pub fn id(&self) -> &str { &self.id }
    pub fn path(&self) -> &Path { &self.path }
    pub fn append(&self, message: &ChatMessage) -> Result<()> {
        let entry = SessionEntry { timestamp: Utc::now(), message: message.clone() };
        let mut file = OpenOptions::new().create(true).append(true).open(&self.path).with_context(|| format!("failed to open local session {}", self.path.display()))?;
        writeln!(file, "{}", serde_json::to_string(&entry)?)?;
        Ok(())
    }
    pub fn read_all(&self) -> Result<Vec<ChatMessage>> {
        if !self.path.exists() { return Ok(Vec::new()); }
        let content = fs::read_to_string(&self.path)?;
        content.lines().filter(|line| !line.trim().is_empty()).map(|line| serde_json::from_str::<SessionEntry>(line).map(|entry| entry.message).map_err(Into::into)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Role;
    #[test]
    fn writes_and_reads_a_local_jsonl_session() {
        let temp = tempfile::tempdir().unwrap();
        let store = SessionStore::in_directory(temp.path(), Some("test-session".into())).unwrap();
        store.append(&ChatMessage::text(Role::User, "hello")).unwrap();
        assert_eq!(store.read_all().unwrap()[0].content, "hello");
    }
}


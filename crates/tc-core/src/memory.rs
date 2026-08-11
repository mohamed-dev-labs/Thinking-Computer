use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{env, fs::{self, OpenOptions}, io::Write, path::{Path, PathBuf}};
use sysinfo::{Disks, System};
use uuid::Uuid;

use crate::model::ChatMessage;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SessionEntry { pub timestamp: DateTime<Utc>, pub message: ChatMessage }

#[derive(Clone, Debug)]
pub struct SessionStore { id: String, path: PathBuf }

impl SessionStore {
    pub fn local(session_id: Option<String>) -> Result<Self> { Self::in_directory(state_root(), session_id) }
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DiskSnapshot { pub name: String, pub total_bytes: u64, pub available_bytes: u64 }

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VmCapabilityProfile {
    pub captured_at: DateTime<Utc>,
    pub platform: String,
    pub cpu_architecture: String,
    pub kernel_version: Option<String>,
    pub os_version: Option<String>,
    pub cpu_count: usize,
    pub total_memory_bytes: u64,
    pub used_memory_bytes: u64,
    pub process_count: usize,
    pub disks: Vec<DiskSnapshot>,
    pub available_tools: Vec<String>,
}

impl VmCapabilityProfile {
    pub fn collect() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        let disks = Disks::new_with_refreshed_list().iter().map(|disk| DiskSnapshot {
            name: disk.name().to_string_lossy().to_string(), total_bytes: disk.total_space(), available_bytes: disk.available_space(),
        }).collect();
        let available_tools = ["cargo", "rustc", "python3", "node", "npm", "git", "ollama", "docker", "podman", "ffmpeg"]
            .into_iter().filter(|tool| command_on_path(tool)).map(str::to_string).collect();
        Self {
            captured_at: Utc::now(), platform: tc_system_bridge::platform_name(), cpu_architecture: tc_system_bridge::cpu_architecture(),
            kernel_version: System::kernel_version(), os_version: System::long_os_version(), cpu_count: system.cpus().len(),
            total_memory_bytes: system.total_memory(), used_memory_bytes: system.used_memory(), process_count: system.processes().len(), disks, available_tools,
        }
    }
}

fn command_on_path(name: &str) -> bool {
    let path = match env::var_os("PATH") { Some(path) => path, None => return false };
    env::split_paths(&path).any(|directory| {
        let direct = directory.join(name);
        let windows = directory.join(format!("{name}.exe"));
        direct.is_file() || windows.is_file()
    })
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MemoryAuditEvent { pub timestamp: DateTime<Utc>, pub action: String, pub summary: String, pub success: bool }

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KnowledgeRecord { pub id: String, pub timestamp: DateTime<Utc>, pub topic: String, pub content: String, pub source: Option<String> }

#[derive(Clone, Debug)]
pub struct AgentMemory { root: PathBuf }

impl AgentMemory {
    pub fn local() -> Result<Self> { Self::in_directory(state_root()) }
    pub fn in_directory(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().join("memory");
        fs::create_dir_all(&root).with_context(|| format!("failed to create agent memory at {}", root.display()))?;
        Ok(Self { root })
    }
    pub fn save_capability_profile(&self, profile: &VmCapabilityProfile) -> Result<()> {
        fs::write(self.root.join("capabilities.json"), serde_json::to_vec_pretty(profile)?)?;
        self.append_audit("capability_profile", "Captured VM OS, resource, disk, process, and installed-tool indicators", true)
    }
    pub fn load_capability_profile(&self) -> Result<Option<VmCapabilityProfile>> {
        let path = self.root.join("capabilities.json");
        if !path.exists() { return Ok(None); }
        Ok(Some(serde_json::from_slice(&fs::read(path)?)?))
    }
    pub fn append_audit(&self, action: &str, summary: &str, success: bool) -> Result<()> {
        let event = MemoryAuditEvent { timestamp: Utc::now(), action: action.to_string(), summary: summary.to_string(), success };
        let mut file = OpenOptions::new().create(true).append(true).open(self.root.join("audit.jsonl"))?;
        writeln!(file, "{}", serde_json::to_string(&event)?)?;
        Ok(())
    }
    pub fn remember(&self, topic: &str, content: &str, source: Option<&str>) -> Result<KnowledgeRecord> {
        if topic.trim().is_empty() || content.trim().is_empty() { anyhow::bail!("memory topic and content must not be empty"); }
        if looks_like_secret(content) { anyhow::bail!("refusing to persist content that appears to contain a secret"); }
        let record = KnowledgeRecord { id: Uuid::new_v4().to_string(), timestamp: Utc::now(), topic: topic.to_string(), content: content.to_string(), source: source.map(ToOwned::to_owned) };
        let mut file = OpenOptions::new().create(true).append(true).open(self.root.join("knowledge.jsonl"))?;
        writeln!(file, "{}", serde_json::to_string(&record)?)?;
        self.append_audit("remember", &format!("topic={topic}; bytes={}", content.len()), true)?;
        Ok(record)
    }
    pub fn recall(&self, limit: usize) -> Result<Vec<KnowledgeRecord>> {
        let path = self.root.join("knowledge.jsonl");
        if !path.exists() { return Ok(Vec::new()); }
        let content = fs::read_to_string(path)?;
        let mut records: Vec<KnowledgeRecord> = content.lines().filter(|line| !line.trim().is_empty()).map(serde_json::from_str).collect::<std::result::Result<_, _>>()?;
        records.reverse();
        records.truncate(limit.min(100));
        Ok(records)
    }
}

fn looks_like_secret(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    lowered.contains("api_key=") || lowered.contains("api key:") || lowered.contains("authorization: bearer") || lowered.contains("-----begin private key")
}

fn state_root() -> PathBuf {
    env::var("TC_HOME").map(PathBuf::from).unwrap_or_else(|_| dirs::data_local_dir().unwrap_or_else(|| PathBuf::from(".")).join("thinking-computer"))
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
    #[test]
    fn persists_a_capability_snapshot_and_audit_record() {
        let temp = tempfile::tempdir().unwrap();
        let memory = AgentMemory::in_directory(temp.path()).unwrap();
        let profile = VmCapabilityProfile::collect();
        memory.save_capability_profile(&profile).unwrap();
        assert_eq!(memory.load_capability_profile().unwrap().unwrap().cpu_architecture, profile.cpu_architecture);
    }
}

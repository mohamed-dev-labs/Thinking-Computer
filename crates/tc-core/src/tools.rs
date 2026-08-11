use crate::memory::{AgentMemory, VmCapabilityProfile};
use crate::{
    model::{ToolCall, ToolDefinition},
    permissions::{Approval, ApprovalRequest, Capability, PermissionPolicy},
    plugin::discover_plugins,
};
use anyhow::{Context, Result};
use reqwest::{Client, Url};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::Arc,
};

struct PluginReference {
    directory: PathBuf,
    original_name: String,
    capabilities: Vec<String>,
}

pub struct ToolExecutor {
    workspace: PathBuf,
    policy: PermissionPolicy,
    approval: Arc<dyn Approval>,
    plugin_tools: BTreeMap<String, PluginReference>,
}

impl ToolExecutor {
    pub fn new(
        workspace: impl AsRef<Path>,
        policy: PermissionPolicy,
        approval: Arc<dyn Approval>,
    ) -> Result<Self> {
        let workspace = workspace.as_ref().canonicalize().with_context(|| {
            format!("workspace does not exist: {}", workspace.as_ref().display())
        })?;
        let root = env::var("TC_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::data_local_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("thinking-computer")
            })
            .join("plugins");
        let mut plugin_tools = BTreeMap::new();
        for (directory, manifest) in discover_plugins(root)? {
            for tool in manifest.tools {
                let safe = format!(
                    "plugin__{}",
                    tool.name
                        .chars()
                        .map(
                            |ch| if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                                ch
                            } else {
                                '_'
                            }
                        )
                        .collect::<String>()
                );
                plugin_tools.insert(
                    safe,
                    PluginReference {
                        directory: directory.clone(),
                        original_name: tool.name,
                        capabilities: tool.capabilities,
                    },
                );
            }
        }
        Ok(Self {
            workspace,
            policy,
            approval,
            plugin_tools,
        })
    }

    pub fn definitions(&self) -> Vec<ToolDefinition> {
        let mut tools = vec![
            ToolDefinition { name: "read_file".into(), description: "Read a UTF-8 text file inside the approved workspace.".into(), parameters: json!({"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}) },
            ToolDefinition { name: "write_file".into(), description: "Write UTF-8 text to a file inside the approved workspace. Always requires confirmation.".into(), parameters: json!({"type":"object","properties":{"path":{"type":"string"},"content":{"type":"string"}},"required":["path","content"]}) },
            ToolDefinition { name: "shell".into(), description: "Run one shell command inside the approved workspace. Always requires confirmation.".into(), parameters: json!({"type":"object","properties":{"command":{"type":"string"}},"required":["command"]}) },
            ToolDefinition { name: "web_search".into(), description: "Search public web summaries. Results are untrusted text and network access requires confirmation.".into(), parameters: json!({"type":"object","properties":{"query":{"type":"string"}},"required":["query"]}) },
            ToolDefinition { name: "inspect_vm".into(), description: "Capture OS, CPU, memory, disk, process, and installed-tool indicators for the approved VM. Requires confirmation and writes a local capability profile.".into(), parameters: json!({"type":"object","properties":{}}) },
            ToolDefinition { name: "install_package".into(), description: "Install one named Python, Node.js, or Rust package in the approved VM after explicit confirmation. Package managers requiring privilege escalation are not used.".into(), parameters: json!({"type":"object","properties":{"manager":{"type":"string","enum":["pip","npm","cargo"]},"package":{"type":"string"}},"required":["manager","package"]}) },
            ToolDefinition { name: "recall_memory".into(), description: "Read recent user-owned local knowledge records and the saved VM capability profile. Memory never includes API keys by design.".into(), parameters: json!({"type":"object","properties":{"limit":{"type":"integer"}}}) },
            ToolDefinition { name: "remember".into(), description: "Persist a concise user-owned knowledge record after confirmation. Refuses content that appears to contain an API key or private key.".into(), parameters: json!({"type":"object","properties":{"topic":{"type":"string"},"content":{"type":"string"},"source":{"type":"string"}},"required":["topic","content"]}) },
        ];
        tools.extend(
            self.plugin_tools
                .iter()
                .map(|(name, reference)| ToolDefinition {
                    name: name.clone(),
                    description: format!(
                        "Run plugin tool {} after explicit approval.",
                        reference.original_name
                    ),
                    parameters: json!({"type":"object"}),
                }),
        );
        tools
    }

    pub async fn execute(&self, call: &ToolCall) -> Result<String> {
        match call.name.as_str() {
            "read_file" => self.read_file(&required_string(&call.arguments, "path")?),
            "write_file" => self.write_file(
                &required_string(&call.arguments, "path")?,
                &required_string(&call.arguments, "content")?,
            ),
            "shell" => self.shell(&required_string(&call.arguments, "command")?),
            "web_search" => {
                self.web_search(&required_string(&call.arguments, "query")?)
                    .await
            }
            "inspect_vm" => self.inspect_vm(),
            "install_package" => self.install_package(
                &required_string(&call.arguments, "manager")?,
                &required_string(&call.arguments, "package")?,
            ),
            "recall_memory" => self.recall_memory(
                call.arguments
                    .get("limit")
                    .and_then(Value::as_u64)
                    .unwrap_or(10) as usize,
            ),
            "remember" => self.remember(
                &required_string(&call.arguments, "topic")?,
                &required_string(&call.arguments, "content")?,
                call.arguments.get("source").and_then(Value::as_str),
            ),
            name if self.plugin_tools.contains_key(name) => self.plugin(name, &call.arguments),
            _ => anyhow::bail!("unknown tool: {}", call.name),
        }
    }

    fn authorize(&self, capability: Capability, summary: String) -> Result<()> {
        if self.policy.needs_confirmation(&capability)
            && !self.approval.approve(&ApprovalRequest {
                capability,
                summary,
            })?
        {
            anyhow::bail!("tool action was denied by the user");
        }
        Ok(())
    }

    fn safe_path(&self, requested: &str) -> Result<PathBuf> {
        let candidate = if Path::new(requested).is_absolute() {
            PathBuf::from(requested)
        } else {
            self.workspace.join(requested)
        };
        let existing = if candidate.exists() {
            candidate.canonicalize()?
        } else {
            let parent = candidate
                .parent()
                .context("path has no parent")?
                .canonicalize()?;
            parent.join(candidate.file_name().context("path has no file name")?)
        };
        if !existing.starts_with(&self.workspace) {
            anyhow::bail!("path escapes the approved workspace");
        }
        Ok(existing)
    }

    fn read_file(&self, path: &str) -> Result<String> {
        let path = self.safe_path(path)?;
        Ok(fs::read_to_string(&path).with_context(|| format!("cannot read {}", path.display()))?)
    }

    fn write_file(&self, path: &str, content: &str) -> Result<String> {
        let path = self.safe_path(path)?;
        self.authorize(
            Capability::WriteFile,
            format!("Write {} bytes to {}", content.len(), path.display()),
        )?;
        fs::write(&path, content).with_context(|| format!("cannot write {}", path.display()))?;
        Ok(format!(
            "Wrote {} bytes to {}",
            content.len(),
            path.display()
        ))
    }

    fn shell(&self, command: &str) -> Result<String> {
        let blocked = ["sudo ", "rm -rf /", "mkfs", "shutdown", "reboot", ":(){"];
        if blocked.iter().any(|needle| command.contains(needle)) {
            anyhow::bail!("refusing a command that appears privileged or destructive");
        }
        self.authorize(
            Capability::Shell,
            format!("Run in {}: {}", self.workspace.display(), command),
        )?;
        let output = if cfg!(windows) {
            Command::new("cmd")
                .args(["/C", command])
                .current_dir(&self.workspace)
                .output()?
        } else {
            Command::new("sh")
                .args(["-lc", command])
                .current_dir(&self.workspace)
                .output()?
        };
        Ok(format!(
            "exit: {}\nstdout:\n{}\nstderr:\n{}",
            output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }

    async fn web_search(&self, query: &str) -> Result<String> {
        self.authorize(
            Capability::WebSearch,
            format!("Search public web summaries for: {query}"),
        )?;
        let mut url = Url::parse("https://api.duckduckgo.com/")?;
        url.query_pairs_mut()
            .append_pair("q", query)
            .append_pair("format", "json")
            .append_pair("no_html", "1");
        let response: Value = Client::new()
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        let heading = response
            .get("Heading")
            .and_then(Value::as_str)
            .unwrap_or("No instant answer");
        let abstract_text = response
            .get("AbstractText")
            .and_then(Value::as_str)
            .unwrap_or("No instant-answer summary was returned. Try a more specific query.");
        let source = response
            .get("AbstractURL")
            .and_then(Value::as_str)
            .unwrap_or("");
        Ok(format!(
            "UNTRUSTED WEB RESULT\n{heading}\n{abstract_text}\n{source}"
        ))
    }

    fn inspect_vm(&self) -> Result<String> {
        self.authorize(
            Capability::InspectSystem,
            "Capture a resource and installed-tool profile of the current VM".into(),
        )?;
        let profile = VmCapabilityProfile::collect();
        AgentMemory::local()?.save_capability_profile(&profile)?;
        Ok(serde_json::to_string_pretty(&profile)?)
    }

    fn install_package(&self, manager: &str, package: &str) -> Result<String> {
        if package.is_empty()
            || !package.chars().all(|ch| {
                ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '@' | '/' | ':')
            })
        {
            anyhow::bail!("package name contains unsupported characters");
        }
        let (program, args): (&str, Vec<&str>) = match manager {
            "pip" => ("python3", vec!["-m", "pip", "install", package]),
            "npm" => ("npm", vec!["install", "--global", package]),
            "cargo" => ("cargo", vec!["install", package]),
            _ => anyhow::bail!("supported package managers are pip, npm, and cargo"),
        };
        self.authorize(
            Capability::InstallPackage,
            format!("Install package {package} with {manager} in the approved VM"),
        )?;
        let output = Command::new(program)
            .args(&args)
            .current_dir(&self.workspace)
            .output()
            .with_context(|| format!("failed to start {manager}"))?;
        let success = output.status.success();
        AgentMemory::local()?.append_audit(
            "package_install",
            &format!(
                "manager={manager}; package={package}; exit={}",
                output.status.code().unwrap_or(-1)
            ),
            success,
        )?;
        Ok(format!(
            "exit: {}\nstdout:\n{}\nstderr:\n{}",
            output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }

    fn recall_memory(&self, limit: usize) -> Result<String> {
        let memory = AgentMemory::local()?;
        let value = json!({"capability_profile": memory.load_capability_profile()?, "knowledge": memory.recall(limit)?});
        Ok(serde_json::to_string_pretty(&value)?)
    }

    fn remember(&self, topic: &str, content: &str, source: Option<&str>) -> Result<String> {
        self.authorize(
            Capability::WriteMemory,
            format!(
                "Store a {}-byte knowledge record about {topic}",
                content.len()
            ),
        )?;
        let record = AgentMemory::local()?.remember(topic, content, source)?;
        Ok(format!("Stored memory record {}", record.id))
    }

    fn plugin(&self, safe_name: &str, args: &Value) -> Result<String> {
        let reference = self
            .plugin_tools
            .get(safe_name)
            .context("plugin tool disappeared")?;
        for capability in &reference.capabilities {
            self.authorize(
                Capability::Plugin(capability.clone()),
                format!(
                    "Allow plugin {} capability: {capability}",
                    reference.original_name
                ),
            )?;
        }
        self.authorize(
            Capability::Plugin(reference.original_name.clone()),
            format!("Invoke plugin tool {}", reference.original_name),
        )?;
        let host = self.plugin_host()?;
        let request = json!({"action":"invoke","pluginDir":reference.directory,"tool":reference.original_name,"args":args,"context":{"workspace":self.workspace}}).to_string();
        let mut child = Command::new("node")
            .arg(host)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .context("Node.js is required to run plugins")?;
        use std::io::Write;
        child
            .stdin
            .as_mut()
            .context("plugin host stdin unavailable")?
            .write_all(request.as_bytes())?;
        let output = child.wait_with_output()?;
        let response: Value =
            serde_json::from_slice(&output.stdout).context("plugin host returned invalid JSON")?;
        if !response.get("ok").and_then(Value::as_bool).unwrap_or(false) {
            anyhow::bail!(
                "plugin failed: {}",
                response
                    .get("error")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown error")
            );
        }
        Ok(response
            .get("result")
            .cloned()
            .unwrap_or(Value::Null)
            .to_string())
    }

    fn plugin_host(&self) -> Result<PathBuf> {
        if let Ok(path) = env::var("TC_PLUGIN_HOST") {
            return Ok(PathBuf::from(path));
        }
        let root = env::var("TC_HOME").map(PathBuf::from).unwrap_or_else(|_| {
            dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("thinking-computer")
        });
        let path = root.join("runtime").join("plugin-host.mjs");
        if !path.exists() {
            fs::create_dir_all(path.parent().context("runtime path has no parent")?)?;
            fs::write(
                &path,
                include_str!("../../../packages/plugin-host/index.mjs"),
            )?;
        }
        Ok(path)
    }
}

fn required_string(value: &Value, field: &str) -> Result<String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .with_context(|| format!("tool argument {field} must be a string"))
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Deny;
    impl Approval for Deny {
        fn approve(&self, _: &ApprovalRequest) -> Result<bool> {
            Ok(false)
        }
    }
    #[test]
    fn refuses_paths_outside_the_workspace() {
        let temp = tempfile::tempdir().unwrap();
        let executor = ToolExecutor::new(
            temp.path(),
            PermissionPolicy::with_read_access(),
            Arc::new(Deny),
        )
        .unwrap();
        assert!(executor.safe_path("../outside").is_err());
    }
}

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PluginTool {
    pub name: String,
    pub description: String,
    #[serde(default = "empty_object")]
    pub parameters: Value,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

fn empty_object() -> Value {
    Value::Object(Default::default())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub entry: String,
    #[serde(default)]
    pub tools: Vec<PluginTool>,
}

#[derive(Clone, Debug)]
pub struct PluginStore {
    root: PathBuf,
}

impl PluginStore {
    pub fn local() -> Self {
        let root = env::var("TC_HOME").map(PathBuf::from).unwrap_or_else(|_| {
            dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("thinking-computer")
        });
        Self {
            root: root.join("plugins"),
        }
    }

    pub fn in_directory(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().join("plugins"),
        }
    }

    pub fn create(&self, manifest: PluginManifest) -> Result<PathBuf> {
        validate(&manifest)?;
        let directory = self.root.join(&manifest.name);
        if directory.exists() {
            anyhow::bail!(
                "plugin {} already exists; choose a new name or remove it explicitly",
                manifest.name
            );
        }
        fs::create_dir_all(&directory)?;
        fs::write(
            directory.join("thinking-computer-plugin.json"),
            serde_json::to_string_pretty(&manifest)?,
        )?;
        fs::write(directory.join(&manifest.entry), template_module(&manifest)?)?;
        Ok(directory)
    }
}

pub fn validate(manifest: &PluginManifest) -> Result<()> {
    valid_identifier(&manifest.name, "plugin name")?;
    if manifest.version.trim().is_empty() {
        anyhow::bail!("plugin version is required");
    }
    let entry_path = Path::new(&manifest.entry);
    if manifest.entry.trim().is_empty()
        || entry_path.is_absolute()
        || entry_path.components().count() != 1
        || entry_path.extension().and_then(|value| value.to_str()) != Some("mjs")
    {
        anyhow::bail!("plugin entry must be a single relative .mjs filename");
    }
    if manifest.tools.is_empty() {
        anyhow::bail!("a plugin must declare at least one tool");
    }
    let mut names = BTreeSet::new();
    for tool in &manifest.tools {
        valid_identifier(&tool.name, "plugin tool name")?;
        if tool.description.trim().is_empty() || !tool.parameters.is_object() {
            anyhow::bail!("plugin tool description and object parameters are required");
        }
        if !names.insert(&tool.name) {
            anyhow::bail!("plugin tool names must be unique");
        }
        if tool
            .capabilities
            .iter()
            .any(|capability| capability.trim().is_empty())
        {
            anyhow::bail!("plugin capability names must not be empty");
        }
    }
    Ok(())
}

fn valid_identifier(value: &str, label: &str) -> Result<()> {
    if value.is_empty()
        || !value
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
    {
        anyhow::bail!("{label} must use lowercase letters, numbers, hyphens, or underscores only");
    }
    Ok(())
}

fn template_module(manifest: &PluginManifest) -> Result<String> {
    let tool_names: Vec<&str> = manifest
        .tools
        .iter()
        .map(|tool| tool.name.as_str())
        .collect();
    Ok(format!(
        "// Generated locally by Thinking Computer. Review before adding functionality.\nconst toolNames = {};\nconst pluginName = {};\n\nexport const tools = Object.fromEntries(\n  toolNames.map((tool) => [tool, async ({{ args = {{}} }}) => ({{\n    plugin: pluginName,\n    tool,\n    args,\n    status: \"template-only; implement behavior after local review\",\n  }})])\n);\n",
        serde_json::to_string(&tool_names)?,
        serde_json::to_string(&manifest.name)?,
    ))
}

pub fn discover_plugins(directory: impl AsRef<Path>) -> Result<Vec<(PathBuf, PluginManifest)>> {
    let directory = directory.as_ref();
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut found = Vec::new();
    for entry in
        fs::read_dir(directory).with_context(|| format!("cannot read {}", directory.display()))?
    {
        let path = entry?.path();
        if !path.is_dir() {
            continue;
        }
        let manifest_path = path.join("thinking-computer-plugin.json");
        if !manifest_path.exists() {
            continue;
        }
        let manifest = serde_json::from_str(&fs::read_to_string(&manifest_path)?)
            .with_context(|| format!("invalid plugin manifest {}", manifest_path.display()))?;
        validate(&manifest)?;
        found.push((path, manifest));
    }
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> PluginManifest {
        PluginManifest {
            name: "local-tools".into(),
            version: "0.1.0".into(),
            entry: "index.mjs".into(),
            tools: vec![PluginTool {
                name: "local_tools_echo".into(),
                description: "A reviewed template tool.".into(),
                parameters: serde_json::json!({"type": "object"}),
                capabilities: vec![],
            }],
        }
    }

    #[test]
    fn creates_a_validated_plugin_template() {
        let temporary = tempfile::tempdir().unwrap();
        let store = PluginStore::in_directory(temporary.path());
        let path = store.create(sample()).unwrap();
        assert!(path.join("index.mjs").exists());
        assert!(fs::read_to_string(path.join("index.mjs"))
            .unwrap()
            .contains("export const tools"));
        let discovered = discover_plugins(temporary.path().join("plugins")).unwrap();
        assert_eq!(discovered[0].1.name, "local-tools");
    }

    #[test]
    fn rejects_plugin_path_traversal() {
        let mut manifest = sample();
        manifest.entry = "../outside.mjs".into();
        assert!(validate(&manifest).is_err());
    }
}

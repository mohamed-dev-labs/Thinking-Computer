use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs, path::{Path, PathBuf}};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PluginTool {
    pub name: String,
    pub description: String,
    #[serde(default = "empty_object")]
    pub parameters: Value,
    #[serde(default)]
    pub capabilities: Vec<String>,
}
fn empty_object() -> Value { Value::Object(Default::default()) }

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PluginManifest { pub name: String, pub version: String, pub entry: String, #[serde(default)] pub tools: Vec<PluginTool> }

pub fn discover_plugins(directory: impl AsRef<Path>) -> Result<Vec<(PathBuf, PluginManifest)>> {
    let directory = directory.as_ref();
    if !directory.exists() { return Ok(Vec::new()); }
    let mut found = Vec::new();
    for entry in fs::read_dir(directory).with_context(|| format!("cannot read {}", directory.display()))? {
        let path = entry?.path();
        if !path.is_dir() { continue; }
        let manifest_path = path.join("thinking-computer-plugin.json");
        if !manifest_path.exists() { continue; }
        let manifest = serde_json::from_str(&fs::read_to_string(&manifest_path)?).with_context(|| format!("invalid plugin manifest {}", manifest_path.display()))?;
        found.push((path, manifest));
    }
    Ok(found)
}


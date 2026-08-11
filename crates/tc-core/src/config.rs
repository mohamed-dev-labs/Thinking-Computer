use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, env, fs, path::{Path, PathBuf}};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ProviderConfig {
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub base_url: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    Openai,
    Anthropic,
    Gemini,
    Ollama,
}

impl ProviderKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Openai => "openai",
            Self::Anthropic => "anthropic",
            Self::Gemini => "gemini",
            Self::Ollama => "ollama",
        }
    }

    pub fn default_model(self) -> &'static str {
        match self {
            Self::Openai => "gpt-4.1-mini",
            Self::Anthropic => "claude-3-5-haiku-latest",
            Self::Gemini => "gemini-2.5-flash",
            Self::Ollama => "llama3.2",
        }
    }

    pub fn env_key(self) -> Option<&'static str> {
        match self {
            Self::Openai => Some("OPENAI_API_KEY"),
            Self::Anthropic => Some("ANTHROPIC_API_KEY"),
            Self::Gemini => Some("GEMINI_API_KEY"),
            Self::Ollama => None,
        }
    }
}

impl std::str::FromStr for ProviderKind {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "openai" => Ok(Self::Openai),
            "anthropic" => Ok(Self::Anthropic),
            "gemini" | "google" => Ok(Self::Gemini),
            "ollama" => Ok(Self::Ollama),
            _ => anyhow::bail!("unsupported provider: {value}"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ResolvedProvider {
    pub kind: ProviderKind,
    pub api_key: Option<String>,
    pub model: String,
    pub base_url: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AppConfig {
    #[serde(default = "default_provider")]
    pub default_provider: String,
    pub workspace: Option<PathBuf>,
    #[serde(default = "default_max_steps")]
    pub max_steps: usize,
    #[serde(default)]
    pub providers: BTreeMap<String, ProviderConfig>,
}

fn default_provider() -> String { "ollama".to_string() }
fn default_max_steps() -> usize { 8 }

impl Default for AppConfig {
    fn default() -> Self {
        Self { default_provider: default_provider(), workspace: None, max_steps: default_max_steps(), providers: BTreeMap::new() }
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        if let Ok(home) = env::var("TC_HOME") { return PathBuf::from(home).join("config.toml"); }
        dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("thinking-computer").join("config.toml")
    }

    pub fn load_or_default(path: Option<&Path>) -> Result<Self> {
        let path = path.map(PathBuf::from).unwrap_or_else(Self::config_path);
        if !path.exists() { return Ok(Self::default()); }
        let contents = fs::read_to_string(&path).with_context(|| format!("failed to read configuration at {}", path.display()))?;
        toml::from_str(&contents).with_context(|| format!("invalid TOML in {}", path.display()))
    }

    pub fn write_example(path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
        fs::write(path, include_str!("../config.example.toml")).with_context(|| format!("failed to write {}", path.display()))
    }

    pub fn resolve_provider(&self, requested: Option<&str>, model: Option<&str>) -> Result<ResolvedProvider> {
        let kind: ProviderKind = requested.unwrap_or(&self.default_provider).parse()?;
        let configured = self.providers.get(kind.as_str()).cloned().unwrap_or_default();
        let api_key = kind.env_key().and_then(|key| env::var(key).ok()).or(configured.api_key);
        let base_url = if kind == ProviderKind::Ollama { env::var("OLLAMA_HOST").ok().or(configured.base_url) } else { configured.base_url };
        Ok(ResolvedProvider {
            kind,
            api_key,
            model: model.map(ToOwned::to_owned).or(configured.model).unwrap_or_else(|| kind.default_model().to_string()),
            base_url,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_to_an_offline_friendly_provider() {
        let config = AppConfig::default();
        let provider = config.resolve_provider(None, None).unwrap();
        assert_eq!(provider.kind, ProviderKind::Ollama);
        assert_eq!(provider.model, "llama3.2");
    }
}


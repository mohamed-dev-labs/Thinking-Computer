use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ProviderConfig {
    pub api_key: Option<String>,
    pub model: Option<String>,
    /// Full chat-completions endpoint for OpenAI-compatible providers.
    pub base_url: Option<String>,
    /// Set to `openai_compatible` for an arbitrary provider profile.
    pub protocol: Option<String>,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ChannelConfig {
    #[serde(default)]
    pub allowed_senders: Vec<String>,
    #[serde(default)]
    pub allowed_recipients: Vec<String>,
    pub webhook_secret_env: Option<String>,
    pub outbound_token_env: Option<String>,
    pub outbound_phone_number_id_env: Option<String>,
    pub outbound_endpoint_env: Option<String>,
    pub outbound_api_version: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ServiceConfig {
    /// A short family label such as `web_extract`, `speech`, `image`, or `custom_http`.
    pub protocol: Option<String>,
    pub api_key: Option<String>,
    pub api_key_env: Option<String>,
    pub base_url: Option<String>,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    Openai,
    Anthropic,
    Gemini,
    Ollama,
    Openrouter,
    Groq,
    Xai,
    Mistral,
    NvidiaNim,
    CloudflareWorkersAi,
    Perplexity,
    Together,
    Fireworks,
    Cerebras,
    Sambanova,
    Deepseek,
    Moonshot,
    Zai,
    Minimax,
    Dashscope,
    BaiduQianfan,
    OpenaiCompatible,
}

impl ProviderKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Openai => "openai",
            Self::Anthropic => "anthropic",
            Self::Gemini => "gemini",
            Self::Ollama => "ollama",
            Self::Openrouter => "openrouter",
            Self::Groq => "groq",
            Self::Xai => "xai",
            Self::Mistral => "mistral",
            Self::NvidiaNim => "nvidia_nim",
            Self::CloudflareWorkersAi => "cloudflare_workers_ai",
            Self::Perplexity => "perplexity",
            Self::Together => "together",
            Self::Fireworks => "fireworks",
            Self::Cerebras => "cerebras",
            Self::Sambanova => "sambanova",
            Self::Deepseek => "deepseek",
            Self::Moonshot => "moonshot",
            Self::Zai => "zai",
            Self::Minimax => "minimax",
            Self::Dashscope => "dashscope",
            Self::BaiduQianfan => "baidu_qianfan",
            Self::OpenaiCompatible => "openai_compatible",
        }
    }

    pub fn default_model(self) -> &'static str {
        match self {
            Self::Openai => "gpt-4.1-mini",
            Self::Anthropic => "claude-3-5-haiku-latest",
            Self::Gemini => "gemini-2.5-flash",
            Self::Ollama => "llama3.2",
            Self::Openrouter => "openai/gpt-4.1-mini",
            Self::Groq => "llama-3.3-70b-versatile",
            Self::Xai => "grok-4.5",
            Self::Mistral => "mistral-large-latest",
            Self::NvidiaNim => "meta/llama-3.1-8b-instruct",
            Self::CloudflareWorkersAi => "@cf/meta/llama-3.1-8b-instruct",
            Self::Perplexity => "sonar",
            Self::Together => "meta-llama/Llama-3.3-70B-Instruct-Turbo",
            Self::Fireworks => "accounts/fireworks/models/llama-v3p3-70b-instruct",
            Self::Cerebras => "llama-3.3-70b",
            Self::Sambanova => "Meta-Llama-3.3-70B-Instruct",
            Self::Deepseek => "deepseek-chat",
            Self::Moonshot => "kimi-k2",
            Self::Zai => "glm-4.7",
            Self::Minimax => "MiniMax-M2.5",
            Self::Dashscope => "qwen-plus",
            Self::BaiduQianfan => "ernie-4.5-8k-preview",
            Self::OpenaiCompatible => "configured-model-required",
        }
    }

    pub fn env_key(self) -> Option<&'static str> {
        match self {
            Self::Openai => Some("OPENAI_API_KEY"),
            Self::Anthropic => Some("ANTHROPIC_API_KEY"),
            Self::Gemini => Some("GEMINI_API_KEY"),
            Self::Ollama => None,
            Self::Openrouter => Some("OPENROUTER_API_KEY"),
            Self::Groq => Some("GROQ_API_KEY"),
            Self::Xai => Some("XAI_API_KEY"),
            Self::Mistral => Some("MISTRAL_API_KEY"),
            Self::NvidiaNim => Some("NVIDIA_API_KEY"),
            Self::CloudflareWorkersAi => Some("CLOUDFLARE_API_TOKEN"),
            Self::Perplexity => Some("PERPLEXITY_API_KEY"),
            Self::Together => Some("TOGETHER_API_KEY"),
            Self::Fireworks => Some("FIREWORKS_API_KEY"),
            Self::Cerebras => Some("CEREBRAS_API_KEY"),
            Self::Sambanova => Some("SAMBANOVA_API_KEY"),
            Self::Deepseek => Some("DEEPSEEK_API_KEY"),
            Self::Moonshot => Some("MOONSHOT_API_KEY"),
            Self::Zai => Some("ZAI_API_KEY"),
            Self::Minimax => Some("MINIMAX_API_KEY"),
            Self::Dashscope => Some("DASHSCOPE_API_KEY"),
            Self::BaiduQianfan => Some("QIANFAN_API_KEY"),
            Self::OpenaiCompatible => Some("OPENAI_COMPATIBLE_API_KEY"),
        }
    }

    fn env_base_url(self) -> Option<&'static str> {
        match self {
            Self::Ollama => Some("OLLAMA_HOST"),
            Self::OpenaiCompatible => Some("OPENAI_COMPATIBLE_BASE_URL"),
            _ => None,
        }
    }

    fn default_endpoint(self) -> Option<&'static str> {
        match self {
            Self::Openai => Some("https://api.openai.com/v1/chat/completions"),
            Self::Openrouter => Some("https://openrouter.ai/api/v1/chat/completions"),
            Self::Groq => Some("https://api.groq.com/openai/v1/chat/completions"),
            Self::Xai => Some("https://api.x.ai/v1/chat/completions"),
            Self::Mistral => Some("https://api.mistral.ai/v1/chat/completions"),
            Self::Perplexity => Some("https://api.perplexity.ai/chat/completions"),
            Self::Together => Some("https://api.together.xyz/v1/chat/completions"),
            Self::Fireworks => Some("https://api.fireworks.ai/inference/v1/chat/completions"),
            Self::Cerebras => Some("https://api.cerebras.ai/v1/chat/completions"),
            Self::Sambanova => Some("https://api.sambanova.ai/v1/chat/completions"),
            Self::Deepseek => Some("https://api.deepseek.com/chat/completions"),
            Self::Ollama => Some("http://127.0.0.1:11434"),
            _ => None,
        }
    }

    pub fn uses_openai_compatible_transport(self) -> bool {
        !matches!(self, Self::Anthropic | Self::Gemini | Self::Ollama)
    }
}

impl std::str::FromStr for ProviderKind {
    type Err = anyhow::Error;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().replace('-', "_").as_str() {
            "openai" => Ok(Self::Openai), "anthropic" => Ok(Self::Anthropic), "gemini" | "google" => Ok(Self::Gemini), "ollama" => Ok(Self::Ollama),
            "openrouter" => Ok(Self::Openrouter), "groq" => Ok(Self::Groq), "xai" | "grok" => Ok(Self::Xai), "mistral" => Ok(Self::Mistral),
            "nvidia" | "nvidia_nim" | "nim" => Ok(Self::NvidiaNim), "cloudflare" | "cloudflare_workers_ai" | "workers_ai" => Ok(Self::CloudflareWorkersAi),
            "perplexity" => Ok(Self::Perplexity), "together" | "together_ai" => Ok(Self::Together), "fireworks" | "fireworks_ai" => Ok(Self::Fireworks),
            "cerebras" => Ok(Self::Cerebras), "sambanova" => Ok(Self::Sambanova), "deepseek" => Ok(Self::Deepseek), "moonshot" | "kimi" => Ok(Self::Moonshot),
            "zai" | "z_ai" | "glm" => Ok(Self::Zai), "minimax" => Ok(Self::Minimax), "dashscope" | "qwen" | "alibaba" => Ok(Self::Dashscope), "baidu" | "qianfan" | "baidu_qianfan" | "ernie" => Ok(Self::BaiduQianfan),
            "openai_compatible" | "compatible" => Ok(Self::OpenaiCompatible), _ => anyhow::bail!("unsupported provider: {value}; use a configured profile with protocol = \"openai_compatible\" for other providers"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ResolvedProvider {
    pub name: String,
    pub kind: ProviderKind,
    pub api_key: Option<String>,
    pub model: String,
    pub base_url: Option<String>,
    pub headers: BTreeMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct ResolvedService {
    pub name: String,
    pub protocol: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub headers: BTreeMap<String, String>,
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
    #[serde(default)]
    pub channels: BTreeMap<String, ChannelConfig>,
    #[serde(default)]
    pub services: BTreeMap<String, ServiceConfig>,
}

fn default_provider() -> String {
    "ollama".to_string()
}
fn default_max_steps() -> usize {
    8
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            default_provider: default_provider(),
            workspace: None,
            max_steps: default_max_steps(),
            providers: BTreeMap::new(),
            channels: BTreeMap::new(),
            services: BTreeMap::new(),
        }
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        if let Ok(home) = env::var("TC_HOME") {
            return PathBuf::from(home).join("config.toml");
        }
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("thinking-computer")
            .join("config.toml")
    }
    pub fn load_or_default(path: Option<&Path>) -> Result<Self> {
        let path = path.map(PathBuf::from).unwrap_or_else(Self::config_path);
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("failed to read configuration at {}", path.display()))?;
        toml::from_str(&contents).with_context(|| format!("invalid TOML in {}", path.display()))
    }
    pub fn write_example(path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, include_str!("../config.example.toml"))
            .with_context(|| format!("failed to write {}", path.display()))
    }
    pub fn resolve_provider(
        &self,
        requested: Option<&str>,
        model: Option<&str>,
    ) -> Result<ResolvedProvider> {
        let name = requested
            .unwrap_or(&self.default_provider)
            .trim()
            .to_ascii_lowercase()
            .replace('-', "_");
        let configured = self.providers.get(&name).cloned().unwrap_or_default();
        let kind: ProviderKind = match configured.protocol.as_deref() {
            Some(protocol) => protocol.parse()?,
            None => name.parse()?,
        };
        let api_key = kind
            .env_key()
            .and_then(|key| env::var(key).ok())
            .or(configured.api_key);
        let base_url = kind
            .env_base_url()
            .and_then(|key| env::var(key).ok())
            .or(configured.base_url)
            .or_else(|| kind.default_endpoint().map(ToOwned::to_owned));
        let chosen_model = model
            .map(ToOwned::to_owned)
            .or(configured.model)
            .unwrap_or_else(|| kind.default_model().to_string());
        if chosen_model == "configured-model-required" {
            anyhow::bail!("provider profile {name} needs a model in configuration or --model");
        }
        if kind.uses_openai_compatible_transport() && base_url.is_none() {
            anyhow::bail!("provider profile {name} needs base_url set to a full OpenAI-compatible chat-completions endpoint");
        }
        Ok(ResolvedProvider {
            name,
            kind,
            api_key,
            model: chosen_model,
            base_url,
            headers: configured.headers,
        })
    }

    pub fn resolve_service(&self, requested: &str) -> Result<ResolvedService> {
        let name = requested.trim().to_ascii_lowercase().replace('-', "_");
        let service = self
            .services
            .get(&name)
            .context("unknown configured service")?;
        let api_key = service
            .api_key_env
            .as_deref()
            .and_then(|key| env::var(key).ok())
            .or_else(|| service.api_key.clone());
        Ok(ResolvedService {
            name,
            protocol: service
                .protocol
                .clone()
                .unwrap_or_else(|| "custom_http".into()),
            api_key,
            base_url: service.base_url.clone(),
            headers: service.headers.clone(),
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
    #[test]
    fn resolves_a_named_compatible_profile() {
        let mut config = AppConfig::default();
        config.providers.insert(
            "private_gateway".into(),
            ProviderConfig {
                protocol: Some("openai_compatible".into()),
                base_url: Some("https://example.test/v1/chat/completions".into()),
                model: Some("my-model".into()),
                ..ProviderConfig::default()
            },
        );
        let provider = config
            .resolve_provider(Some("private_gateway"), None)
            .unwrap();
        assert_eq!(provider.kind, ProviderKind::OpenaiCompatible);
        assert_eq!(provider.model, "my-model");
    }

    #[test]
    fn parses_a_separate_service_registry() {
        let config: AppConfig = toml::from_str(
            r#"
            [services.firecrawl]
            protocol = "web_extract"
            base_url = "https://api.firecrawl.dev"
            "#,
        )
        .unwrap();
        assert_eq!(
            config.services["firecrawl"].protocol.as_deref(),
            Some("web_extract")
        );
    }

    #[test]
    fn resolves_a_service_key_from_the_configured_environment_name() {
        let config: AppConfig = toml::from_str(
            r#"
            [services.firecrawl]
            protocol = "web_extract"
            api_key_env = "TC_TEST_FIRECRAWL_KEY"
            "#,
        )
        .unwrap();
        std::env::set_var("TC_TEST_FIRECRAWL_KEY", "test-key");
        let service = config.resolve_service("firecrawl").unwrap();
        std::env::remove_var("TC_TEST_FIRECRAWL_KEY");
        assert_eq!(service.api_key.as_deref(), Some("test-key"));
    }

    #[test]
    fn resolves_common_routing_profiles_without_custom_code() {
        let config = AppConfig::default();
        for name in [
            "openrouter",
            "groq",
            "xai",
            "mistral",
            "perplexity",
            "together",
            "deepseek",
        ] {
            let provider = config.resolve_provider(Some(name), None).unwrap();
            assert!(provider.base_url.as_deref().unwrap().starts_with("http"));
            assert!(provider.kind.uses_openai_compatible_transport());
        }
    }
}

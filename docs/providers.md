# Providers and Service Integrations

Thinking Computer separates the model that reasons over a task from optional services that perform a narrower operation. A provider key does not grant filesystem, shell, message-delivery, or VM authority; those actions remain controlled by the Rust permission layer.

## Provider transports

| Transport | Shipped profiles | Credential handling |
| --- | --- | --- |
| Native | OpenAI, Anthropic, Gemini, and Ollama. | Use the provider environment variable or local Ollama endpoint. |
| OpenAI-compatible | OpenRouter, Groq, xAI, Mistral, NVIDIA NIM, Cloudflare Workers AI, Perplexity, Together, Fireworks, Cerebras, SambaNova, DeepSeek, Moonshot, Z.AI, MiniMax, DashScope, Baidu Qianfan, and a custom compatible profile. | Use the profile's documented environment variable; customize the endpoint or model only in local configuration. |
| Local | Ollama. | `OLLAMA_HOST` is optional; the default points to the local Ollama listener. |

The exact profile name, default model, environment-variable name, and default endpoint behavior are implemented in [`crates/tc-core/src/config.rs`](../crates/tc-core/src/config.rs). The safe example configuration is [`crates/tc-core/config.example.toml`](../crates/tc-core/config.example.toml). Treat those two files as the authoritative project contract when preparing a VM.

## Quick configuration

```bash
export OPENAI_API_KEY="..."
thinking-computer chat --provider openai "Summarize the approved workspace."

export OPENROUTER_API_KEY="..."
thinking-computer chat --provider openrouter "Explain the project architecture."

thinking-computer repl --provider ollama --model llama3.2
```

For an endpoint that implements compatible chat completions, define a named local profile:

```toml
[providers.private_gateway]
protocol = "openai_compatible"
model = "your-model-id"
base_url = "https://gateway.example/v1/chat/completions"
# Prefer OPENAI_COMPATIBLE_API_KEY in the environment.
```

Then call `thinking-computer chat --provider private_gateway "..."`. The CLI fails rather than silently inventing a model or endpoint when a required profile field is absent.

## Optional services

Services are not agent models. The example configuration includes Firecrawl for web extraction, ElevenLabs for speech, Fal.ai for image generation, and a generic HTTP service shape. Services are reachable only through the corresponding implemented, permission-gated tool or reviewed Plugin. A service key must never be placed in a Skill, Plugin manifest, message body, audit record, or committed TOML file.

## Contributor checklist

When adding a provider or service profile, document the declared protocol, credential environment-variable name, request/response contract, a fixture-based test, and any account, region, or model caveat. Do not send live calls from tests. This keeps the registry broad without converting a configuration label into an unverified compatibility claim.

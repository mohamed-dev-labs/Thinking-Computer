# Provider API Research Notes

This record tracks official API evidence used for provider classification in Thinking Computer. The registry favors protocol families over one hard-coded client per brand, which keeps the CLI extensible as provider catalogues change.

| Service | Classification | Verified integration detail | Source |
| --- | --- | --- | --- |
| OpenRouter | OpenAI-compatible routing API | Its documentation describes schemas similar to the OpenAI Chat API, a `POST /api/v1/chat/completions` endpoint, Bearer authentication, and model-provider normalization including tool-calling support. | https://openrouter.ai/docs/api_reference/overview |
| Cloudflare Workers AI | OpenAI-compatible model API | Its official documentation exposes chat completions at `/v1/chat/completions` under an account-specific `/ai/v1` base URL with `Authorization: Bearer` authentication and a Cloudflare model identifier. | https://developers.cloudflare.com/workers-ai/configuration/open-ai-compatibility/ |
| NVIDIA NIM | OpenAI-compatible inference API | NVIDIA documents `POST /v1/chat/completions` with streaming and tool-calling support, plus Responses and Anthropic-compatible endpoints. A local or hosted NIM profile can therefore use the generic OpenAI-compatible adapter. | https://docs.nvidia.com/nim/large-language-models/latest/api-reference.html |
| Groq | Mostly OpenAI-compatible inference API | Groq documents the base URL `https://api.groq.com/openai/v1`, a `GROQ_API_KEY`, and a mostly compatible OpenAI client surface with documented feature differences. | https://console.groq.com/docs/openai |
| xAI | OpenAI-compatible API | xAI documents `XAI_API_KEY` and an OpenAI client configuration with base URL `https://api.x.ai/v1`; its product API also includes model-native tool and media capabilities. | https://docs.x.ai/developers/quickstart |
| Mistral AI | Chat-completions API compatible in shape | Mistral documents `POST /v1/chat/completions`, Bearer authentication, message arrays, tools, and tool-choice controls. | https://docs.mistral.ai/api/ |
| Moonshot Kimi | OpenAI-compatible model API | Moonshot’s official migration guide identifies the Kimi API as compatible with OpenAI interface specifications. | https://platform.kimi.ai/docs/guide/migrating-from-openai-to-kimi |
| MiniMax | OpenAI-compatible model API | MiniMax documents an OpenAI-format text API for integration with the broader OpenAI API ecosystem. | https://platform.minimax.io/docs/api-reference/text-openai-api |
| Alibaba DashScope / Model Studio | OpenAI-compatible model API | Alibaba Cloud documents OpenAI-compatible chat and Responses interfaces for Qwen models. | https://www.alibabacloud.com/help/en/model-studio/compatibility-of-openai-with-dashscope |

## Design consequence

OpenRouter and Cloudflare Workers AI can reuse the agent's OpenAI-compatible message and tool-call transport while retaining dedicated configuration profiles for their API key, base URL, model identifier, and any service-specific headers. The registry will also retain an arbitrary `openai_compatible` profile so a user can add a new compatible provider without waiting for a binary release.

# Thinking Computer

<p align="center">
  <img src="assets/thinking-computer-wordmark.png" width="560" alt="Thinking Computer mono-pixel terminal wordmark" />
</p>

> **A VM-first, local-memory personal agent for the terminal.**

Thinking Computer is an open-source CLI agent designed to operate inside a dedicated virtual machine, sandbox, or VPS. It combines a Rust control engine with a Python/Hermes-compatible input adapter, an optional Node.js plugin host, and a narrow C++ system bridge. The result is an inspectable personal agent that can reason across multi-step tasks, use approved tools, inspect an assigned VM, remember user-approved knowledge, and schedule work—while keeping the VM owner in control of every high-impact capability.

> **Run this in a disposable or dedicated VM whenever practical.** Thinking Computer can read approved workspaces, execute approved shell commands, inspect VM capabilities, perform network requests, and install user-level packages only after approval. It is not intended to silently control a personal computer, browser profile, credentials, or hardware.

## What it does

| Area | Implemented behavior |
| --- | --- |
| Terminal agent | Interactive REPL, one-shot `chat`, and line-oriented local `rpc` modes. No website or desktop application. |
| Rust control plane | Rust owns permission policy, bounded agent loops, provider dispatch, session state, knowledge memory, VM profiling, task definitions, and tool execution. |
| Hermes-compatible input | A dependency-free Python adapter accepts a normalized Hermes-style task event and forwards it to Rust over a local JSON protocol. Python never owns shell access, policy decisions, or memory persistence. |
| Provider registry | Native adapters for OpenAI, Anthropic, Gemini, and Ollama, plus a configurable OpenAI-compatible transport for routing services and many hosted inference providers. |
| Local memory | JSONL sessions, user-approved knowledge records, VM capability snapshots, and audit events stored locally under `TC_HOME` or the OS application-data directory. |
| Computer use | Workspace-scoped file reads and writes, guarded shell commands, guarded public web summaries, VM capability inspection, and approved `pip`, `npm`, or `cargo` package installation. |
| Scheduling | Local task definitions with Cron validation and exportable Unix crontab lines. The tool does not silently install a background daemon or register a task with the OS. |
| Extensibility | A Node.js plugin host using one-shot JSON over standard input/output. Plugins remain behind the Rust approval boundary. |

## Architecture

```text
Hermes-compatible Python input
              │  local JSON line
              ▼
Rust CLI and policy engine ────────► selected model provider
   │       │        │
   │       │        └──────────────► Node.js plugin host
   │       └───────────────────────► local memory, audit, schedules
   └───────────────────────────────► C++ system bridge and approved VM tools
```

The four languages have intentionally different responsibilities.

| Component | Language | Responsibility |
| --- | --- | --- |
| `crates/thinking-computer` and `crates/tc-core` | Rust | CLI, agent loop, memory, providers, tool policy, VM analytics, schedules, and audit records. |
| `python/hermes_adapter` | Python | Hermes-compatible task normalization and local forwarding to `thinking-computer rpc`. |
| `packages/plugin-host` | Node.js | Optional plugin discovery and invocation through a constrained JSON contract. |
| `native/system-bridge` | C++17 | Minimal platform and CPU identification functions called safely from Rust. |

Read [the VM-first runtime model](docs/vm-first-runtime.md) for the full capability boundary and memory layout.

## Security and operating model

Thinking Computer treats model output, web results, plugin output, file contents, and inbound channel data as **untrusted**. A model may request a tool, but it does not receive implicit authority to execute it. The owner sees a confirmation for guarded actions unless they deliberately choose `--yes` for a controlled VM run.

| Capability | Default behavior | Boundary |
| --- | --- | --- |
| Read file | Allowed only inside the approved workspace. | Canonical paths escaping the workspace are rejected. |
| Write file | Requires confirmation. | Canonical paths escaping the workspace are rejected. |
| Shell | Requires confirmation. | Runs from the workspace; obvious destructive or privileged patterns are rejected; `sudo` is never added. |
| Web search | Requires confirmation. | Results are marked as untrusted text. |
| VM inspection | Requires confirmation. | Persists OS/resource/tool indicators locally; it does not collect credentials. |
| Package install | Requires confirmation. | Supports `pip`, `npm`, and `cargo`; it does not run privileged system-package commands. |
| Memory write | Requires confirmation. | Rejects values that resemble API keys, bearer tokens, or private-key blocks. |

The default storage root can be set explicitly for an isolated VM:

```bash
export TC_HOME="$HOME/.thinking-computer"
```

It contains session JSONL files, user knowledge records, `capabilities.json`, `audit.jsonl`, local schedules, and optional plugins. API keys should be provided through environment variables and are intentionally excluded from the memory model.

## Installation

Release archives are planned for Linux x86_64, macOS x86_64 and Apple Silicon, and Windows x86_64. Once a release is published, use a reviewed installer command:

```bash
# Linux or macOS — inspect the script before running it in a VM.
curl -fsSL https://raw.githubusercontent.com/mohamed-dev-labs/Thinking-Computer/main/scripts/install.sh | bash
```

```powershell
# Windows PowerShell — inspect the script before running it in a VM.
irm https://raw.githubusercontent.com/mohamed-dev-labs/Thinking-Computer/main/scripts/install.ps1 | iex
```

To build from source, install a current stable Rust toolchain, a C++17 compiler, and Node.js only if you need Node.js plugins. Python 3.10+ is needed only for the Hermes-compatible adapter.

```bash
git clone https://github.com/mohamed-dev-labs/Thinking-Computer.git
cd Thinking-Computer
cargo test --workspace
cargo build --release
./target/release/thinking-computer doctor
```

## Quick start

Initialize configuration and the sample plugin in an isolated VM directory:

```bash
export TC_HOME="$HOME/.thinking-computer"
thinking-computer init
thinking-computer config
thinking-computer plugins list
```

Choose an LLM provider. Ollama is the default because it can run locally; hosted providers use environment variables in preference to a local configuration-file secret.

```bash
export OPENAI_API_KEY="..."
thinking-computer chat --provider openai "Review this repository and explain the test layout."

export ANTHROPIC_API_KEY="..."
thinking-computer repl --provider anthropic

export GEMINI_API_KEY="..."
thinking-computer chat --provider gemini "Draft a concise release note from CHANGELOG.md"

# Local Ollama with a tool-capable model.
thinking-computer repl --provider ollama --model llama3.2
```

Capture a VM profile only after reviewing the environment and choosing the global `--yes` flag or confirming interactively:

```bash
thinking-computer memory profile
thinking-computer memory recall --limit 20
```

## Providers and services

Thinking Computer separates **model providers** from other capabilities. A model provider supplies an agent model; a service such as web extraction, speech, image generation, messaging, or a webhook is modeled as a separate, permission-gated tool or channel. This prevents a routing API key from being confused with an OS or messaging permission.

| Transport | Profiles currently represented | Credential environment variable |
| --- | --- | --- |
| Native | OpenAI, Anthropic, Gemini, Ollama | `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `GEMINI_API_KEY`, or local Ollama configuration. |
| OpenAI-compatible | OpenRouter, Groq, xAI, Mistral, NVIDIA NIM, Cloudflare Workers AI, Perplexity, Together, Fireworks, Cerebras, SambaNova, DeepSeek, Moonshot Kimi, Z.AI GLM, MiniMax, and DashScope/Qwen. | Provider-specific variables such as `OPENROUTER_API_KEY`, `GROQ_API_KEY`, or the provider profile’s key. |
| Configurable gateway | Any endpoint implementing compatible chat-completions tool calling. | `OPENAI_COMPATIBLE_API_KEY` or `providers.<name>.api_key`. |

Vendor endpoints can differ by account, region, model, and capability. The bundled `config.example.toml` is therefore the source of truth for the exact endpoint field and profile shape. A private gateway can be added without recompiling:

```toml
[providers.private_gateway]
protocol = "openai_compatible"
model = "your-model-id"
base_url = "https://gateway.example/v1/chat/completions"
# api_key = "prefer OPENAI_COMPATIBLE_API_KEY instead"
```

## Local memory and VM intelligence

Rust persists three distinct forms of memory: conversational sessions, user-approved knowledge records, and VM intelligence. The `inspect_vm` tool captures an owner-approved snapshot of the OS, CPU architecture, CPU count, memory indicators, disk capacity, process count, and a small list of detected developer tools. It does **not** mean the agent owns the VM; it creates a planning artifact so the agent can make better proposals for local models, builds, package installation, and task execution.

The `remember` tool only writes after confirmation. It rejects content that resembles common secret formats. This enables a personalized agent using local knowledge without turning conversation logs into a credential store or silently training a model on unrelated machine data.

## Schedules and background work

Thinking Computer keeps task definitions locally and exports a command that the VM owner may choose to register with their scheduler. It does not install a hidden service.

```bash
thinking-computer schedule add \
  --name daily-summary \
  --cron "0 9 * * *" \
  --provider ollama \
  "Summarize the approved workspace and write a short daily note."

thinking-computer schedule export
```

For an externally reachable webhook or messaging bridge, run a reviewed bridge process on a VM you control, with a verified provider signature, sender allowlist, rate limit, and a separate capability policy. A personal device is not the preferred host for that process.

## Hermes-compatible Python adapter

The Python adapter forwards normalized inputs to Rust over standard input/output. It is dependency-free and has no direct network or shell authority.

```python
from thinking_computer_adapter import handle_hermes_input

result = handle_hermes_input({
    "id": "task-42",
    "prompt": "Inspect the approved workspace and propose a build plan.",
    "provider": "ollama",
})
print(result["text"])
```

The adapter is a compatibility boundary for Hermes-oriented input flows. It is not an untracked Hermes runtime. Any future direct import of Hermes Agent source must preserve its MIT notices and be documented in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

## Plugins

Plugins live under `<TC_HOME>/plugins/<plugin-name>` and consist of a `thinking-computer-plugin.json` manifest plus an ECMAScript module. The `init` command installs `hello-plugin` as a working example. Every declared plugin capability and invocation remains subject to Rust-side approval. The protocol is documented in [the plugin API guide](docs/plugin-api.md).

## Development

Run the core checks before opening a pull request:

```bash
cargo test --workspace
cargo fmt --all -- --check
python3 -m py_compile python/hermes_adapter/thinking_computer_adapter.py
node --check packages/plugin-host/index.mjs
```

Read [CONTRIBUTING.md](CONTRIBUTING.md), [SECURITY.md](SECURITY.md), and [the upstream research record](docs/upstream-research.md) before contributing a provider, channel, or third-party integration.

## License and acknowledgements

Thinking Computer is licensed under the [MIT License](LICENSE). The Rust/C++/Node.js engine is a clean-room implementation. The design acknowledges the personal-agent ideas of [OpenClaw][1] and the CLI, tool, provider, and extension patterns of [Hermes Agent][2]. The project includes a Python/Hermes-compatible adapter but does not silently include upstream source code, logos, or assets. See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for attribution requirements.

## References

[1]: https://github.com/openclaw/openclaw "OpenClaw repository"
[2]: https://github.com/NousResearch/hermes-agent "Hermes Agent repository"
[3]: https://platform.openai.com/docs/guides/function-calling "OpenAI function calling documentation"
[4]: https://docs.anthropic.com/en/docs/build-with-claude/tool-use "Anthropic tool-use documentation"
[5]: https://ai.google.dev/gemini-api/docs/function-calling "Gemini function-calling documentation"
[6]: https://docs.ollama.com/capabilities/tool-calling "Ollama tool-calling documentation"

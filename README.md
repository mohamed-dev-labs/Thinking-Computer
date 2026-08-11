# Thinking Computer

> **A local-first personal agent for the terminal.**

Thinking Computer is an open-source CLI agent for Linux, macOS, Windows, VPS hosts, and local development machines. It keeps configuration and session history on the user's device, connects only to the model provider selected by the user, and asks for approval before actions that can change files, invoke a shell, access the network, or run a plugin.

The project draws architectural inspiration from the local-first personal-agent ideas in [OpenClaw][1] and the CLI, provider, tool, and extension patterns visible in [Hermes Agent][2]. It is a clean-room Rust/C++/Node.js implementation and does not copy either codebase, branding, or assets.

| Capability | Current behavior |
| --- | --- |
| Terminal interface | Interactive REPL plus a one-command `chat` mode. No website, desktop app, gateway, or daemon. |
| Agent loop | Bounded multi-step loop with a default limit of eight steps. The model can return text or request one or more tools. |
| Model providers | OpenAI, Anthropic, Gemini, and Ollama, with environment variables overriding local TOML configuration. |
| Local memory | Append-only JSONL session history stored under the operating system's local data directory, or under `TC_HOME`. |
| Built-in tools | Read files, write files, shell commands, and public web summaries, all scoped to a selected workspace. |
| Permission model | Reads inside the workspace are allowed; writes, shell commands, network search, and plugins require confirmation unless `--yes` is supplied. |
| Extensions | A small Node.js JSON-over-standard-input/output host with manifest-based tool declarations. Node.js is optional unless a plugin is used. |
| Native bridge | Rust owns the agent process; a deliberately narrow C++17 bridge exposes native platform and CPU identification. |

## Security model

Thinking Computer treats model output, web results, plugin output, and file contents as **untrusted data**. An LLM can request an action but does not receive automatic authority to execute it. The terminal user sees and approves any guarded action. The shell tool runs within the selected workspace, never adds `sudo`, and rejects several clearly destructive command patterns. This is a helpful safety boundary, not a substitute for reviewing the command you approve.

> Keep API keys in environment variables whenever possible. Session logs deliberately do not persist keys, but prompts and tool results may contain sensitive information supplied by the user.

## Installation

Release archives will be published for Linux x86_64, macOS x86_64 and Apple Silicon, and Windows x86_64. Once a release is available, install the matching binary with one of the following commands.

```bash
# Linux or macOS
curl -fsSL https://raw.githubusercontent.com/mohamed-dev-labs/Thinking-Computer/main/scripts/install.sh | bash
```

```powershell
# Windows PowerShell
irm https://raw.githubusercontent.com/mohamed-dev-labs/Thinking-Computer/main/scripts/install.ps1 | iex
```

For development or before the first release, build from source. You need a current stable Rust toolchain, a C++17 compiler, CMake or a standard native build environment, and Node.js only if you plan to invoke plugins.

```bash
git clone https://github.com/mohamed-dev-labs/Thinking-Computer.git
cd Thinking-Computer
cargo test --workspace
cargo build --release
./target/release/thinking-computer doctor
```

On Windows, replace the final command with:

```powershell
.\target\release\thinking-computer.exe doctor
```

## Quick start

Create a local configuration file and bundled sample plugin. This writes only inside the normal local application-data directory, or into `TC_HOME` if you set it.

```bash
thinking-computer init
thinking-computer config
thinking-computer plugins list
```

Choose one provider with an environment variable. Ollama is the default because it supports local operation without an API key.

```bash
export OPENAI_API_KEY="..."
thinking-computer chat --provider openai "Review this repository and explain the test layout."

export ANTHROPIC_API_KEY="..."
thinking-computer repl --provider anthropic

export GEMINI_API_KEY="..."
thinking-computer chat --provider gemini "Draft a concise release note from CHANGELOG.md"

# Local Ollama, when a tool-capable model is already available locally.
thinking-computer repl --provider ollama --model llama3.2
```

Tool calling is implemented as an application-controlled conversation loop: the app supplies typed tool declarations, the model requests a call, the CLI executes it only after policy approval, and the result is returned to the model for the next step. This follows the documented application-side tool loop in the provider APIs.[3] [4] [5] [6]

## Configuration

The first `init` command generates a commented TOML file. Its default location follows OS conventions: `~/.config/thinking-computer` on typical Linux systems, the Application Support directory on macOS, and the AppData directory on Windows. Set `TC_HOME` to make both configuration and sessions portable, such as on a VPS.

```toml
default_provider = "ollama"
max_steps = 8
# workspace = "/absolute/path/to/approved/workspace"

[providers.openai]
model = "gpt-4.1-mini"

[providers.anthropic]
model = "claude-3-5-haiku-latest"

[providers.gemini]
model = "gemini-2.5-flash"

[providers.ollama]
model = "llama3.2"
base_url = "http://127.0.0.1:11434"
```

| Provider | Preferred secret source | Optional configuration |
| --- | --- | --- |
| OpenAI | `OPENAI_API_KEY` | `providers.openai.api_key`, `model`, `base_url` |
| Anthropic | `ANTHROPIC_API_KEY` | `providers.anthropic.api_key`, `model`, `base_url` |
| Gemini | `GEMINI_API_KEY` | `providers.gemini.api_key`, `model`, `base_url` |
| Ollama | No key by default; `OLLAMA_HOST` for the host URL | `providers.ollama.model`, `base_url` |

Environment variables take precedence over the file. Avoid committing a configuration file containing a secret.

## Commands

```text
thinking-computer init
thinking-computer doctor
thinking-computer config
thinking-computer plugins list
thinking-computer repl --provider ollama
thinking-computer chat --provider openai "Explain the current directory"
thinking-computer --workspace ./project chat --provider anthropic "Find the main entry point"
thinking-computer --yes chat --provider gemini "Create docs/overview.md from the source tree"
```

The `--yes` flag auto-approves guarded actions for the current run. Use it only in a workspace and environment you trust; normal interactive confirmation is the recommended default.

## Plugins

Plugins live under `<TC_HOME>/plugins/<plugin-name>` and consist of a `thinking-computer-plugin.json` manifest plus an ECMAScript module. The initial `init` command installs `hello-plugin` as a working example. A plugin cannot bypass the Rust policy layer: each invocation and every declared capability is approved by the user. The JSON request/response contract is described in [the plugin API guide](docs/plugin-api.md).

## Development

The repository contains three cooperating layers:

| Path | Language | Purpose |
| --- | --- | --- |
| `crates/tc-core` | Rust | Agent loop, provider adapters, local state, policy, tool definitions, and plugin discovery. |
| `crates/thinking-computer` | Rust | `clap` commands, the interactive REPL, and terminal approval prompts. |
| `native/system-bridge` | C++17 | Tiny, audited platform/CPU integration boundary consumed safely by Rust through `cxx`. |
| `packages/plugin-host` | Node.js | Manifest loading and one-shot JSON protocol for optional plugins. |

Run the core local checks before opening a pull request. Format Rust changes with `cargo fmt --all` before submitting them.

```bash
cargo test --workspace
node --check packages/plugin-host/index.mjs
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for the contribution workflow and [SECURITY.md](SECURITY.md) for responsible disclosure.

## License and acknowledgements

Thinking Computer is licensed under the [MIT License](LICENSE). The initial codebase is a clean-room implementation. The design acknowledges OpenClaw and Hermes Agent as upstream inspiration; each upstream project declares an MIT license in its repository. Their source code, logos, and assets are not included here. See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) and the maintained [upstream research record](docs/upstream-research.md).

## References

[1]: https://github.com/openclaw/openclaw "OpenClaw repository"
[2]: https://github.com/NousResearch/hermes-agent "Hermes Agent repository"
[3]: https://platform.openai.com/docs/guides/function-calling "OpenAI function calling documentation"
[4]: https://docs.anthropic.com/en/docs/build-with-claude/tool-use "Anthropic tool-use documentation"
[5]: https://ai.google.dev/gemini-api/docs/function-calling "Gemini function-calling documentation"
[6]: https://docs.ollama.com/capabilities/tool-calling "Ollama tool-calling documentation"

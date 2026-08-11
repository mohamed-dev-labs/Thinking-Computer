# Upstream Research Record

This record documents the upstream projects considered as architectural references for Thinking Computer. It is not a notice of code incorporation.

| Upstream project | What was observed | Reuse decision |
| --- | --- | --- |
| [OpenClaw](https://github.com/openclaw/openclaw) | A personal AI assistant connecting models, tools, and user-facing channels through a gateway. Although the GitHub API metadata reports a non-standard license identifier (`NOASSERTION`), the repository's `LICENSE` file is an MIT License with a 2026 OpenClaw Foundation copyright notice. | Architectural reference. Any future direct reuse requires retaining the MIT copyright and license notices; the first implementation is clean-room and does not copy its source, assets, or branding. |
| [NousResearch Hermes Agent](https://github.com/NousResearch/hermes-agent) | An agent project with CLI and gateway-oriented components, model-provider integration, session/state functionality, tools, and plugins. The repository declares the MIT License. | Architectural reference. Any future direct reuse requires retaining the MIT copyright and license notices. The initial implementation is intentionally written from scratch in Rust, C++, and Node.js rather than copying Python code. |

## Design implications

Thinking Computer is deliberately narrower and local-first: it is a CLI-only agent, keeps session data on the user's machine, requires explicit confirmation before shell execution, and uses documented provider APIs selected by the user. It will not reuse the OpenClaw branding, user interface, assets, or codebase. The source links above are retained in the README acknowledgements section for transparent attribution.

## Sources

1. [OpenClaw repository](https://github.com/openclaw/openclaw), reviewed 2026-08-11.
2. [Hermes Agent repository](https://github.com/NousResearch/hermes-agent), reviewed 2026-08-11.

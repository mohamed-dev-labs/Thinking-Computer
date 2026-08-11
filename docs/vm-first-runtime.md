# VM-First Runtime Model

Thinking Computer is designed for an **agent-owned virtual machine or disposable sandbox**, not an unrestricted personal laptop. A virtual machine makes it possible to grant a useful set of capabilities—source-tree access, development tools, temporary downloads, provider connectivity, and optional package installation—while keeping the owner’s everyday files and credentials outside the agent’s execution boundary.

> The agent must never acquire silent or implicit control. Every high-impact capability is granted at onboarding or confirmed at execution time, recorded locally, and can be removed by the VM owner.

## Language ownership

| Layer | Language | Owns | Does not own |
| --- | --- | --- | --- |
| Agent engine | Rust | Permission policy, provider dispatch, tool loop, session memory, knowledge memory, task execution, VM profiling, audit events, and background-job definitions. | UI-specific behavior or unreviewed third-party tool execution. |
| Hermes compatibility adapter | Python | Normalizing input from Hermes-compatible frontends and forwarding one typed JSON request to Rust. | Shell execution, API credentials, memory persistence, package installation, or policy decisions. |
| Plugin host | Node.js | Discovering and invoking optional ECMAScript tools through a one-shot JSON protocol. | Direct OS authority; Rust remains the approval boundary. |
| System bridge | C++17 | Narrow, audited operating-system and CPU identification functions consumed from safe Rust. | Agent reasoning or high-level tool policy. |

## Local protocol

Python sends one JSON line to `thinking-computer rpc`; Rust returns one JSON response. The input envelope is deliberately small:

```json
{
  "id": "optional-request-id",
  "prompt": "Inspect the approved workspace and propose a build plan.",
  "provider": "ollama",
  "model": "optional-model-id",
  "session": "optional-local-session-id"
}
```

Rust validates the request, selects a provider, applies the selected workspace and approval policy, runs the bounded agent loop, writes local session memory, and returns `{"id": ..., "ok": true, "result": {"text": ...}}`. No listening network process is created by this protocol.

## Memory and capability intelligence

The Rust memory layer stores data under the application data directory or `TC_HOME`. It has three separate, user-readable stores:

| Store | Format | Contents | Explicit exclusions |
| --- | --- | --- | --- |
| Sessions | JSONL per session | Conversation turns and tool results needed for continuity. | Provider API keys and secrets supplied through environment variables. |
| Knowledge | Append-only JSONL | User-approved facts, project notes, and sources that the agent may recall. | Content that resembles API keys, bearer tokens, or private-key blocks. |
| VM intelligence | `capabilities.json` plus `audit.jsonl` | OS, architecture, CPU count, memory and disk indicators, process count, installed-tool indicators, capability snapshots, and approved package-installation events. | Passwords, provider credentials, browser cookies, and private key material. |

The `inspect_vm` tool creates a fresh capability snapshot only after the owner approves the request. The `install_package` tool supports Python, Node.js, and Rust user-level package managers; it does not add `sudo`, silently modify system package sources, or attempt privileged system installation.

## Capability model

Reads inside the selected workspace are allowed by default. Writing a file, saving a memory record, network search, shell invocation, VM inspection, package installation, plugin execution, a webhook action, or a scheduled job each requires an explicit policy decision. `--yes` is available only for a controlled VM run and is deliberately documented as unsafe for unreviewed tasks.

## Hermes integration boundary

Hermes Agent is an MIT-licensed upstream reference and optional compatibility source. Thinking Computer can track approved Hermes code as a visible upstream component and preserve its license notices, but the Rust engine remains authoritative. The adapter can accept Hermes-originated task input; it cannot grant Hermes unrestricted computer control.

## Operating a bridge or webhook

The CLI itself starts no daemon. A messaging bridge or webhook listener is an optional, explicitly started process intended for a VM or server that the owner controls. It must bind only to the configured interface, verify a channel signature where the provider offers one, reject untrusted sender IDs, rate-limit requests, and forward a normalized task to the Rust engine. For local-only use, polling or a CLI invocation can replace an externally reachable webhook.

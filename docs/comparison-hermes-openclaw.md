# Relationship to Hermes Agent and OpenClaw

Thinking Computer is a clean-room project influenced by publicly described ideas in Hermes Agent and OpenClaw. It does **not** incorporate their source code, copy their architecture wholesale, or claim compatibility with their private/internal formats. Its code, policy model, documentation, and command surface are maintained independently.

## Publicly documented inspirations

Hermes Agent publicly describes a self-improving agent with skills, memory, scheduled automations, multi-provider operation, terminal interfaces, and messaging access.[1] OpenClaw publicly describes a single-operator assistant centered on a local Gateway that connects sessions, tools, events, channels, a CLI, and a TUI.[2]

| Topic | Hermes Agent | OpenClaw | Thinking Computer |
| --- | --- | --- | --- |
| Core implementation | Public project primarily implemented in Python with TypeScript components.[1] | Public project organized around a local Gateway and TypeScript/Node-oriented ecosystem.[2] | Rust policy engine and CLI, a deliberately small C++ inspection bridge, Python compatibility adapter, and Node.js Plugin host. |
| Skills and learning | Publicly describes Skills creation, memory, and a learning loop.[1] | Publicly documents Skills and Plugins as extensions.[2] | Local manifest-based Skills with secret checks; Plugin templates validated in Rust and audited locally. |
| Channels | Publicly lists several messaging channels.[1] | Publicly lists a wide channel set connected through its Gateway.[2] | Trusted inbound pairing plus narrow, allowlisted outbound adapters for Telegram, Discord, WhatsApp Cloud API, and reviewed generic HTTPS endpoints. |
| Control plane | Public agent runtime and associated interfaces.[1] | Gateway is the documented local control plane.[2] | Rust core is the policy owner; no hidden network Gateway is required for the Python adapter. |
| Runtime posture | Supports multiple terminal backends and remote environments.[1] | Designed to run on devices under the operator's control.[2] | VM-first posture with local JSONL state, explicit capability approvals, and a small auditable system bridge. |

## Intentional differences

Thinking Computer emphasizes language separation: Rust controls policy and resource decisions, C++ is constrained to a compact system-inspection interface, Python only forwards through a local protocol, and Node.js only hosts reviewed Plugins. Its local continuous-improvement worker is bounded, resumable, and halt-on-gate-failure rather than an instruction to produce arbitrary volumes of code.

The projects may overlap in broad concepts such as local operation, model choice, terminal use, Skills, Plugins, channels, and scheduled work. That overlap reflects common open-source agent requirements, not a claim of code sharing or feature parity. Contributors should consult each upstream project's own license, documentation, and security guidance before evaluating either project.[1] [2]

## References

[1]: https://github.com/nousresearch/hermes-agent "Hermes Agent repository"
[2]: https://github.com/openclaw/openclaw "OpenClaw repository"

# Thinking Computer Documentation

Thinking Computer is an open-source, **VM-first terminal agent**. Its public interface is a CLI; it does not turn a personal machine into an unrestricted background service by default. This documentation describes what is implemented today, which actions require explicit approval, and how contributors can extend the project safely.

| Guide | Purpose |
| --- | --- |
| [Architecture](architecture.md) | Language boundaries, process flow, local data, and policy ownership. |
| [Security and operations](security-and-operations.md) | VM-first deployment, approvals, audit records, background work, and service boundaries. |
| [Providers](providers.md) | Native, compatible, local-model, and optional service configuration. |
| [Skills](skills.md) | Local manifest lifecycle and validation. |
| [Plugins](plugins.md) | Rust-validated Node.js plugin templates and invocation contract. |
| [Channels](channel-outbound-research.md) | Trusted inbound senders and policy-gated outbound delivery. |
| [Continuous improvement](continuous-improvement.md) | The resumable 20-slot improvement worker and its security gates. |
| [Scheduled improvement cycles](continuous-improvement-scheduling.md) | Optional VM timer template and separate, auditable cycle state. |
| [Improvement log](improvement-log.md) | Bounded improvement decisions, security references, validation evidence, and residual risk. |
| [Hermes and OpenClaw comparison](comparison-hermes-openclaw.md) | Similarities, differences, provenance, and clean-room boundary. |
| [Connected service boundaries](connected-service-boundaries.md) | GitHub and Cloudflare responsibilities and guardrails. |

> **Safety baseline:** Run Thinking Computer in a disposable or otherwise approved VM. Give capabilities deliberately, retain local audit records, and never put provider, channel, or deployment credentials in repository files.

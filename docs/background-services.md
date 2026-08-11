# Background Services and VM Operations

Thinking Computer is a CLI, not a hidden background daemon. The `schedule` command stores task definitions in local data and emits commands that the VM owner can review. The CLI never writes a crontab, creates a `launchd` job, registers a Windows task, opens a firewall port, or creates a system service on its own.

| Target | Command produced by the CLI | Owner action |
| --- | --- | --- |
| Linux | `thinking-computer schedule export --target linux` | Review the resulting entry, then paste it into `crontab -e` for the chosen VM user. |
| macOS | `thinking-computer schedule export --target macos` | Review the user-crontab entry. A maintainer may later add a `launchd` template; it is not installed automatically. |
| Windows | `thinking-computer schedule export --target windows` | Review the generated `schtasks /Create` command and choose a user-owned executable path before running it. |

## Webhook listener lifecycle

`thinking-computer webhook listen` is an explicitly foreground process. A long-lived webhook receiver needs a dedicated VM or VPS selected and maintained by its owner. Run it behind a reverse proxy/TLS endpoint only after configuring the provider’s signature or secret header, a strict `allowed_senders` list, and a network boundary appropriate to the VM. The listener verifies and records incoming messages but intentionally does not send them into the agent loop by itself.

> Do not expose a personal workstation merely to receive bot webhooks. Prefer a dedicated VM with a non-privileged user, a narrow firewall rule, a separate workspace, and environment-provided secrets.

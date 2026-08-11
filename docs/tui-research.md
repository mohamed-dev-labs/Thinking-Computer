# Terminal UI Technology Record

Thinking Computer provides a Rust terminal interface first so the primary binary remains portable across Linux, macOS, Windows, local terminals, SSH, and VPS sessions. The initial implementation uses the open-source Rust TUI stack `ratatui` plus `crossterm`; it is embedded in the same Rust binary and does not create a browser, desktop app, or background server.

OpenTUI remains a supported **optional companion** for an advanced TypeScript visual shell. Its documentation describes a Zig-native core with TypeScript bindings and a C ABI, but its native Node renderer currently requires Bun or a newer Node runtime with FFI enabled. Thinking Computer will therefore keep the agent engine and permission model in Rust, expose a local JSON boundary, and only enable an OpenTUI companion on a VM where its runtime prerequisites are installed and verified.[1] [2]

| Layer | Chosen role |
| --- | --- |
| Rust TUI | Default cross-platform `thinking-computer tui` command; terminal-safe and immediately testable with the core binary. |
| OpenTUI | Optional open-source visual companion that can consume the local Rust JSON protocol once a VM explicitly installs its supported renderer runtime. |
| Rust core | Sole owner of provider access, memory, policy approvals, skills, plugins, channels, and system actions. |

## References

[1]: https://opentui.com/docs/getting-started/ "OpenTUI Getting Started"
[2]: https://github.com/anomalyco/opentui "OpenTUI repository"

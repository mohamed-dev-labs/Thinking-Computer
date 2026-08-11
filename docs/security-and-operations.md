# Security and Operations

Thinking Computer is designed for an isolated VM, not unrestricted use on a personal computer. The project makes this boundary explicit because an agent that can read files, write workspace content, run commands, install packages, browse, or send messages must have clear operator control.

## Deployment posture

| Environment | Recommended use | Constraints |
| --- | --- | --- |
| Disposable VM | Recommended for active computer-use tasks, development, and controlled continuous-improvement sessions. | Keep provider keys in environment variables; snapshot before broad changes. |
| Personal computer | Use only for limited, reviewed tasks with minimum capabilities. | Do not grant broad unattended shell or package-install permissions. |
| Public website | Documentation and project discovery only. | It is not the agent runtime and does not accept visitor credentials or device-control requests. |

## Continuous improvement worker

The local improvement worker executes a fixed 20-slot plan, preserves durable state, and stops after any failed quality gate. Its default behavior is review-only. Autonomous edits require both `--execute-agent` and a VM sentinel file as an explicit acknowledgement that the work occurs in an approved VM.

Quality gates include Rust formatting and tests, Python syntax validation, Node.js syntax validation, a diff check, and a local hard-coded-credential scan. The worker records the exact successful commands before and after every completed task, changed files, rationale, residual risk, and a durable event log. It can create at most the configured number of meaningful commits; it never manufactures commits to reach a numerical target.

## Audit records

The local audit trail records the action category, a minimal summary, and success state. It is designed to answer questions such as whether a VM profile was captured, a Plugin template was created, an inbound message was dispatched, or an approved outbound message was delivered—without saving raw provider keys, channel tokens, or private message content.

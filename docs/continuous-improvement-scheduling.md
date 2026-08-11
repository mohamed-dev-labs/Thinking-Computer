# Scheduled Improvement Cycles

Thinking Computer ships a **bounded** 20-slot improvement worker. It is designed for a disposable or explicitly approved VM, and it stops immediately when a quality or security gate fails. It does not run continuously simply because the repository exists.

| Mode | State file | Intended use | Safety boundary |
| --- | --- | --- | --- |
| Resume | `.thinking-computer/improvement-state.json` | Review or continue one approved cycle | A halted state must be inspected before it is cleared. |
| New cycle | `.thinking-computer/improvement-cycles/<timestamp>.json` | Start a separately auditable scheduled cycle | `--new-cycle` cannot be combined with a custom state file. |

The Linux service and timer templates in `automation/continuous-improvement/` are disabled by default. An operator who owns an approved VM may copy them to their user service directory, create the required VM sentinel, review the `ExecStart` line, and then enable the timer. The default cadence is weekly with a randomized delay; it is intentionally not a high-frequency polling loop.

Every cycle runs the plan's security and quality gates before and after each bounded task. It records commands, rationale, changed-file counts, residual risk, and the exact state path. The worker must not be used to manufacture commits or to bypass the agent's normal capability approvals.

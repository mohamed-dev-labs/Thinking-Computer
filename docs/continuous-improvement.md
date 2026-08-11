# Continuous Improvement Worker

Thinking Computer provides a **bounded, resumable 20-hour worker** for an isolated Linux VM. It is not a hidden daemon and it is not this chat continuing after you leave. The worker follows twenty named one-hour slots, saves state locally after each slot, runs quality gates before and after every task, and stops immediately on a failed gate.

> Run this only in a disposable or explicitly approved VM. The edit-enabled mode passes `--yes` to the local Thinking Computer CLI, so it may change repository files inside that VM.

## Safety boundaries

| Boundary | Behavior |
| --- | --- |
| Explicit activation | The default is review-only. Edit mode requires `--execute-agent` plus a repository-local `.thinking-computer/VM_ONLY` sentinel file. |
| Quality gates | Rust format and workspace tests, Python syntax, Node.js syntax, and `git diff --check` run before and after each slot. |
| Automatic halt | A failed command sets `halted: true` in the durable state file and ends the process. It never skips to a later task. |
| Commits | Commits are off by default. `--commit` permits no more than twenty task-linked commits; the worker does not push. |
| Scope control | The worker rejects a task that changes more than thirty tracked paths by default. Each completed task records its rationale, changed paths, passing gates, and residual risk in the durable state file. |
| Privilege | The service template runs without new privileges and scopes writes to the repository and local agent state. Do not run it as root. |
| Network | Network-enabled agent actions still depend on the local provider and capability policy. Page content and provider results remain untrusted input. |

## Review-only rehearsal

```bash
python3 automation/continuous-improvement/run_improvement.py \
  --repo "$PWD" --no-wait
```

This verifies the plan, state handling, and all gates in sequence, but does not call an LLM, edit source, or create commits.

## Controlled VM run

After inspecting the plan, placing the VM sentinel, configuring a provider locally, and accepting the repository boundary, run:

```bash
mkdir -p .thinking-computer
touch .thinking-computer/VM_ONLY
python3 automation/continuous-improvement/run_improvement.py \
  --repo "$PWD" \
  --execute-agent \
  --agent-binary ./target/release/thinking-computer \
  --provider ollama \
  --commit
```

The process persists its state at `.thinking-computer/improvement-state.json`. It does not auto-push; review the local commits and push them manually after inspection. The included service template can be copied to the VM's service directory after replacing `%h/Thinking-Computer` with the actual clone path. Start it manually once; it is intentionally not enabled by the installer.

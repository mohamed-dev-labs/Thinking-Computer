# Improvement Log

This log records bounded improvements that were selected for a clear security, reliability, or operator-control outcome. It is not a changelog substitute, and it must never be used to justify artificial commits or line-count targets.

## 2026-08: Permission and evidence cycle

The cycle reviewed guidance on least-privilege tools, untrusted external content, auditable high-impact actions, and bounded multi-agent communication. OWASP identifies prompt injection, over-permissioned tools, persistent-memory poisoning, excessive autonomy, and cascading multi-agent failures as relevant agent risks. [1] Its AI testing guidance also emphasizes repeatable testing beyond conventional software checks. [2]

| Improvement | Implementation boundary | Validation evidence |
| --- | --- | --- |
| Skill activation | New Skills begin disabled; explicit local enable/disable records an audit event and requires a terminal approval unless the operator uses the existing explicit non-interactive flag. | Rust unit test verifies disabled-by-default and explicit activation. |
| Task delegation | Expert tasks have typed results, a limited step budget, a concurrent-agent ceiling, and can request only capabilities inherited from the parent. | Rust unit tests verify inherited capability output and refusal of privilege escalation. |
| Web provenance | Search and fetch save a local, bounded record of request, sanitized source URL, and character count; page content is not copied into provenance storage. | Rust unit tests verify record persistence and query/fragment removal. |
| Operator scheduling | A disabled-by-default Linux timer template starts separately auditable bounded cycles with the same security and quality gates. | Python tests verify cycle-state path isolation and gate-stop behavior. |

### Quality gates used

The cycle ran formatting, workspace unit tests, release compilation, Python worker and security-scan tests, a repository secret scan, Node syntax checks, and a Worker response test. A failure in a gate halts the worker and preserves state for review.

### Post-release rehearsal

On 2026-08-11, a complete `--new-cycle --no-wait` review-only rehearsal completed all 20 tasks without calling a model or modifying source files. Its local, deliberately uncommitted state artifact was written to `.thinking-computer/improvement-cycles/2026-08-11T21-19-47.364845-00-00.json`. The artifact records 20 completed tasks, the pre- and post-task command results, and successful security-scan output. The worker's unit suite separately exercises quality-gate and security-gate failures and verifies that either condition writes a halted state instead of continuing.

### Residual risk

The project still relies on an operator to approve capabilities, provision credentials in environment variables, and choose an isolated VM. `--yes` is an intentionally explicit operator bypass for interactive prompts; it is not a background default. Scheduled templates are provided but are never installed or enabled automatically.

## References

[1] [OWASP AI Agent Security Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/AI_Agent_Security_Cheat_Sheet.html)

[2] [OWASP AI Testing Guide](https://owasp.org/www-project-ai-testing-guide/)

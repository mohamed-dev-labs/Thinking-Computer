# Security Policy

Thinking Computer runs model-directed actions on a user's computer. Please report vulnerabilities privately rather than opening a public issue, especially for permission bypasses, path escapes, plugin-host escapes, command-injection paths, credential disclosure, unsafe provider request handling, or malicious prompt/tool output that can execute without consent.

Send a concise report to the repository maintainer through GitHub private security reporting when it is enabled, or otherwise open a minimal issue requesting a private contact channel without including exploit details. Include the affected version or commit, operating system, reproduction steps, impact, and a suggested mitigation if available.

The current security promise is intentionally limited: the tool asks for confirmation before guarded actions and scopes filesystem operations to a workspace, but users must inspect commands before approving them. Do not run the CLI with elevated privileges, do not point it at a sensitive workspace unless necessary, and do not use `--yes` for unreviewed model tasks.


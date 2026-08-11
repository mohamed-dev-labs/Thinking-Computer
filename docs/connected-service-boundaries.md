# Connected Service Boundaries

This project uses connected GitHub and Cloudflare services only for repository stewardship and deployment that the project owner has authorized. Credentials are never copied into project files, documentation, generated configuration, terminal output, or agent memory.

| Service | Read-only verification completed | Authorized project scope | Guardrail |
| --- | --- | --- | --- |
| GitHub | The authenticated account has administrator access to the public `mohamed-dev-labs/Thinking-Computer` repository. | Push tested, meaningful source and documentation commits to that repository. | No force-push, no history rewrite, no artificial commits, and no credentials in commits. |
| Cloudflare | The Cloudflare and Worker Bindings connectors are enabled; the Worker inventory is readable. | Create or update the dedicated Thinking Computer marketing worker and its Worker subdomain after the site passes validation. | Do not alter unrelated Workers, DNS records, storage, or account settings. |

## Operating boundary

Repository and Worker inspection are treated as read-only operations. A deployment changes the public site, so it occurs only for the dedicated Thinking Computer artifact after its source, static assets, and checks have been reviewed. The deployment record should identify the Worker name and public URL but must never disclose credentials, account IDs, or unrelated project inventory.

The CLI agent and the public marketing website remain separate deliverables. The website documents the open-source CLI; it does not introduce a hosted agent service, request provider keys, or gain access to a visitor's computer.

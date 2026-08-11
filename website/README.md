# Website Worker

`worker.js` is a dependency-free Cloudflare Worker that serves the Thinking Computer marketing site. It intentionally has no form handling, visitor accounts, analytics, cookies, or provider-key fields. The public page is a documentation and project-discovery surface only.

The deployed public address is **https://tc.clatterlabs.workers.dev**. Cloudflare's standard Worker subdomain ends in `workers.dev` (plural).

Validate its syntax with:

```bash
node --check website/worker.js
```

The Cloudflare deployment is performed through the connected account for the dedicated Thinking Computer Worker only. See `docs/connected-service-boundaries.md` before deployment.

# Plugins

Plugins extend Thinking Computer through a narrow Node.js ES module contract. The Rust core owns Plugin discovery and manifest validation; the Node.js host only invokes a tool selected from that manifest.

## Create a reviewed template

```bash
thinking-computer plugins create \
  --name incident-tools \
  --version 0.1.0 \
  --tool incident_tools_summarize
```

The command creates a local directory containing `thinking-computer-plugin.json` and `index.mjs`. The generated module is **template-only**: it returns its input arguments and intentionally contains no privileged behavior. The creation event is written to the local audit log without recording credentials or input content.

```json
{
  "name": "incident-tools",
  "version": "0.1.0",
  "entry": "index.mjs",
  "tools": [
    {
      "name": "incident_tools_summarize",
      "description": "Generated template tool; implement only after local review.",
      "parameters": {"type": "object", "properties": {}},
      "capabilities": []
    }
  ]
}
```

## Rust validation

The manifest must use a lowercase name made from letters, digits, hyphens, or underscores. It needs a non-empty version, at least one uniquely named tool, object-shaped tool parameters, and a single relative `.mjs` entry filename. Absolute paths and traversal such as `../outside.mjs` are rejected.

## Node host contract

The host reads a JSON request from standard input, loads the validated manifest from the requested local Plugin directory, then imports the declared entry module. The module exports either `tools` or a default object whose matching member is an async function:

```js
export const tools = {
  incident_tools_summarize: async ({ args, context }) => ({
    summary: String(args.text ?? ""),
    reviewedContext: Boolean(context.sessionId),
  }),
};
```

Avoid network calls, shell execution, credential parsing, or unbounded file access in a Plugin unless a Rust-side capability and approval path has been designed, tested, and documented first. List discovered local Plugins with `thinking-computer plugins list`.

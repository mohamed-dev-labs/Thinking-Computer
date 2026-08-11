# Plugin API

Thinking Computer plugins are optional Node.js ECMAScript modules. The Rust core remains the policy boundary; plugins are invoked only through a one-shot JSON protocol over standard input and output. Standard error is reserved for diagnostics and must not be used as protocol output.

## Manifest

Each plugin lives in `<TC_HOME>/plugins/<plugin-name>` and contains a manifest named `thinking-computer-plugin.json`.

```json
{
  "name": "hello-plugin",
  "version": "0.1.0",
  "entry": "index.mjs",
  "tools": [
    {
      "name": "hello_plugin_greet",
      "description": "Return a greeting.",
      "parameters": {
        "type": "object",
        "properties": { "name": { "type": "string" } },
        "required": ["name"]
      },
      "capabilities": []
    }
  ]
}
```

The `capabilities` field is declarative. Declaring a capability does not grant it; the user must approve each plugin invocation and each requested capability in the terminal.

## Module interface

The module exports a `tools` object. Each key matches a manifest tool name and each value is an asynchronous function receiving `{ args, context }`.

```js
export const tools = {
  hello_plugin_greet: async ({ args }) => ({ greeting: `Hello, ${args.name}!` })
};
```

## Protocol

The Rust core starts the bundled host with Node.js, sends exactly one JSON request on standard input, and reads exactly one JSON response from standard output. A successful invocation has the form `{"ok":true,"result":...}`; failures have the form `{"ok":false,"error":"..."}`. The host does not expose a shell or filesystem API to plugins.

The context currently contains only the approved workspace path. Plugin authors should regard all arguments and context values as untrusted input and should return structured, serializable data.


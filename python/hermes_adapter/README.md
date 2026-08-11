# Python / Hermes Adapter

This package is the Python-facing boundary for Thinking Computer. It can be called by a Hermes-compatible frontend or a small Python integration, but it has no authority to invoke tools itself. Every request is serialized as one JSON line and sent to `thinking-computer rpc`; the Rust engine controls provider access, local memory, permissions, the C++ system bridge, and every OS-facing action.

```python
from thinking_computer_adapter import handle_hermes_input

result = handle_hermes_input({
    "id": "telegram-turn-42",
    "prompt": "Summarize the source files in the approved workspace.",
    "provider": "ollama",
})
print(result["text"])
```

The adapter is intentionally dependency-free. Hermes Agent can be supplied as a separately tracked upstream component under its MIT license; its code is not silently executed by this package.

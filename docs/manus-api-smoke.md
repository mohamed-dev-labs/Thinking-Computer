# Manus OpenAI-Compatible Smoke Test

On 2026-08-11, the configured Manus gateway was probed with an environment-injected bearer credential that was never written to disk or printed. `GET https://forge.manus.ai/v1/models` returned HTTP 200.

The response advertised model identifiers including `gpt-5-mini`, `gpt-5-nano`, `gpt-5`, Claude, and Gemini options. The bounded Thinking Computer smoke test uses `gpt-5-mini` with a temporary OpenAI-compatible provider profile and does not add the key or endpoint to committed configuration.

The public repository was cloned into a separate clean directory, built successfully, and invoked with one bounded prompt. With `OPENAI_COMPATIBLE_BASE_URL` and `OPENAI_COMPATIBLE_API_KEY` supplied only to that process, the command asked the selected model to return `MANUS_GATEWAY_OK`; the returned response was exactly `MANUS_GATEWAY_OK`.

> This record confirms compatibility discovery only. Credentials remain environment-only, and a real outbound messaging test requires its own trusted-recipient configuration.

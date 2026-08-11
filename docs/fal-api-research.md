# Fal.ai Integration Record

Thinking Computer treats Fal.ai as a **media service**, not as the model-provider route that controls the agent loop. The service is enabled only when a user configures its endpoint and provides `FAL_KEY` locally. A model tool call remains subject to Rust-side confirmation before an outbound request is made.

| Integration detail | Decision |
| --- | --- |
| Authentication | Read the API key from `FAL_KEY`, or from the local service profile's configured environment-variable name. Send it as `Authorization: Key <value>`. |
| Default synchronous endpoint | `https://fal.run/fal-ai/flux/schnell`, a documented model API endpoint for a prompt-driven image response. |
| Request body | A JSON object with a required `prompt` field. The agent may select a model identifier, but it cannot set a privileged deployment action. |
| Result handling | Return the provider JSON as untrusted service output, including any temporary media URL. The tool does not download, execute, or automatically publish the resulting media. |
| Asynchronous jobs | Fal's queue API is intentionally left to a later integration. It supports submit/status/result/webhook flows and requires persistent VM operation to manage safely. |

## References

1. [Fal.ai — Get Your API Key](https://fal.ai/docs/documentation/setting-up/authentication)
2. [Fal.ai — Quick Start](https://fal.ai/docs/documentation/quickstart)
3. [Fal.ai — Asynchronous Inference](https://fal.ai/docs/documentation/model-apis/inference/queue)

# Channel and Webhook Research Record

Thinking Computer treats inbound messaging as an optional, VM-hosted bridge rather than an implicit agent capability. A bridge must normalize an inbound event, verify the provider’s security mechanism, enforce a trusted-sender policy, record an audit event, and pass only the approved task text to the Rust engine.

| Channel | Supported delivery model confirmed | Required safety behavior | Source |
| --- | --- | --- | --- |
| Telegram Bot API | Bots can receive JSON `Update` objects through HTTPS `setWebhook`; Telegram also offers mutually exclusive long polling through `getUpdates`. | Configure `secret_token`, verify the `X-Telegram-Bot-Api-Secret-Token` header, deduplicate `update_id`, and restrict permitted chat/user IDs. | [Telegram Bot API](https://core.telegram.org/bots/api) |
| Discord | Application interactions arrive through a Gateway connection or an outgoing HTTP interactions endpoint. | Validate `X-Signature-Ed25519` and `X-Signature-Timestamp` against the raw body; handle `PING` before accepting tasks. | [Discord Interactions Overview](https://discord.com/developers/docs/interactions/overview) |
| WhatsApp Business Platform | Cloud/API webhooks deliver JSON events for inbound messages and other business-account updates. | Use the platform’s endpoint-verification and token/signature mechanics, deduplicate provider message IDs, and allowlist business users or conversations. | [WhatsApp Webhooks](https://developers.facebook.com/documentation/business-messaging/whatsapp/webhooks/overview) |
| LINE Messaging API | Registered bot servers receive webhook event objects for messages and other chat events. | Verify the request signature before processing, process asynchronously, and deduplicate redeliveries by webhook event ID. | [LINE Receive Messages](https://developers.line.biz/en/docs/messaging-api/receiving-messages/) |
| Signal | No official bot/webhook platform was verified in this research pass. | Do not claim direct production support. Any future integration must use an explicit user-managed adapter and a documented security review. | N/A |

## Decision

The first public bridge contract will support a normalized JSON message envelope and provide documented adapter boundaries for the verified channels. A deployed listener is optional and must run on a dedicated VM or server selected by the user. Thinking Computer will not bind a public port, register a webhook URL, or store a channel secret by default.

## References

1. [Telegram Bot API](https://core.telegram.org/bots/api).
2. [Discord Interactions Overview](https://discord.com/developers/docs/interactions/overview).
3. [WhatsApp Business Platform Webhooks](https://developers.facebook.com/documentation/business-messaging/whatsapp/webhooks/overview).
4. [LINE Messaging API: Receive messages](https://developers.line.biz/en/docs/messaging-api/receiving-messages/).

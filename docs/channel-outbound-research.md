# Outbound Channel Adapter Record

Thinking Computer supports narrow outbound text delivery only after three local controls succeed: the recipient appears in the configured allowlist, the operator confirms the terminal approval request unless `--yes` was explicitly supplied, and the provider credential is read only from the named environment variable. The core does not persist transport tokens, message bodies, or raw provider responses in its memory audit log.

| Channel | Official transport used | Required local configuration |
| --- | --- | --- |
| Telegram | HTTPS `sendMessage` using the Bot API token in the request path and JSON `chat_id` plus `text`. | `allowed_recipients` and `outbound_token_env`. |
| Discord | `POST /channels/{channel.id}/messages` with Bot authorization and a JSON `content` body. | Numeric channel ID in `allowed_recipients` and `outbound_token_env`. |
| WhatsApp | `POST /{Version}/{Phone-Number-ID}/messages` with Bearer authorization and the Cloud API text-message payload. | E.164-style approved recipient, `outbound_token_env`, and `outbound_phone_number_id_env`. |
| Generic | Reviewed HTTPS endpoint receiving `{recipient, text}` under Bearer authorization. | Explicit recipient allowlist, token environment variable, and HTTPS endpoint environment variable. |

The implementation intentionally does not send Signal or LINE messages directly. Their deployment models and account-specific adapters require an explicitly reviewed local plugin instead of a guessed core protocol.

## Security properties

The outbound path has no autonomous recipient discovery, no wildcard recipient rule, no inline token option, no fallback to credentials stored in TOML, and no automatic response forwarding. A successful delivery records only the channel, recipient, character count, and HTTP status in the local audit trail.

## References

[1]: https://core.telegram.org/bots/api "Telegram Bot API"
[2]: https://docs.discord.com/developers/resources/message "Discord Message Resource"
[3]: https://developers.facebook.com/documentation/business-messaging/whatsapp/reference/whatsapp-business-phone-number/message-api "WhatsApp Cloud API Message API"

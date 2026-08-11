use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::env;

use crate::{bridge::ChannelKind, config::ChannelConfig};

const MAX_OUTBOUND_CHARS: usize = 2_000;

#[derive(Clone, Debug)]
pub struct OutboundDelivery {
    pub channel: String,
    pub recipient: String,
    pub status: u16,
    pub response_summary: String,
}

#[derive(Clone)]
pub struct OutboundRequest {
    pub channel: String,
    pub recipient: String,
    pub endpoint: String,
    pub body: Value,
    authorization: OutboundAuthorization,
}

#[derive(Clone)]
enum OutboundAuthorization {
    TelegramBot(String),
    Bearer(String),
    DiscordBot(String),
}

#[async_trait]
pub trait OutboundTransport: Send + Sync {
    async fn send(&self, request: OutboundRequest) -> Result<OutboundDelivery>;
}

#[derive(Clone, Default)]
pub struct HttpOutboundTransport {
    client: reqwest::Client,
}

#[async_trait]
impl OutboundTransport for HttpOutboundTransport {
    async fn send(&self, request: OutboundRequest) -> Result<OutboundDelivery> {
        let (url, authorization) = match request.authorization {
            OutboundAuthorization::TelegramBot(token) => {
                (format!("{}/bot{token}/sendMessage", request.endpoint), None)
            }
            OutboundAuthorization::Bearer(token) => {
                (request.endpoint, Some(format!("Bearer {token}")))
            }
            OutboundAuthorization::DiscordBot(token) => {
                (request.endpoint, Some(format!("Bot {token}")))
            }
        };
        let mut call = self.client.post(url).json(&request.body);
        if let Some(value) = authorization {
            call = call.header(reqwest::header::AUTHORIZATION, value);
        }
        let response = call
            .send()
            .await
            .context("outbound channel request failed")?;
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        if !status.is_success() {
            anyhow::bail!(
                "outbound {} delivery failed with HTTP {}",
                request.channel,
                status
            );
        }
        Ok(OutboundDelivery {
            channel: request.channel,
            recipient: request.recipient,
            status: status.as_u16(),
            response_summary: summarize_response(&text),
        })
    }
}

pub fn recipient_is_trusted(config: Option<&ChannelConfig>, recipient: &str) -> bool {
    config
        .map(|channel| {
            channel
                .allowed_recipients
                .iter()
                .any(|allowed| allowed == recipient)
        })
        .unwrap_or(false)
}

pub async fn send_message<T: OutboundTransport>(
    channel: ChannelKind,
    config: &ChannelConfig,
    recipient: &str,
    text: &str,
    transport: &T,
) -> Result<OutboundDelivery> {
    if !recipient_is_trusted(Some(config), recipient) {
        anyhow::bail!("outbound message denied: recipient is not in the channel allowlist");
    }
    if text.trim().is_empty() || text.chars().count() > MAX_OUTBOUND_CHARS {
        anyhow::bail!("outbound message must contain 1-{MAX_OUTBOUND_CHARS} characters");
    }
    transport
        .send(build_request(channel, config, recipient, text)?)
        .await
}

fn build_request(
    channel: ChannelKind,
    config: &ChannelConfig,
    recipient: &str,
    text: &str,
) -> Result<OutboundRequest> {
    let token = env_value(config.outbound_token_env.as_deref(), "outbound_token_env")?;
    match channel {
        ChannelKind::Telegram => Ok(OutboundRequest {
            channel: "telegram".into(),
            recipient: recipient.into(),
            endpoint: "https://api.telegram.org".into(),
            body: json!({"chat_id": recipient, "text": text, "disable_web_page_preview": true}),
            authorization: OutboundAuthorization::TelegramBot(token),
        }),
        ChannelKind::Discord => {
            if !recipient.chars().all(|character| character.is_ascii_digit()) {
                anyhow::bail!("Discord recipient must be a numeric channel ID");
            }
            Ok(OutboundRequest {
                channel: "discord".into(),
                recipient: recipient.into(),
                endpoint: format!("https://discord.com/api/v10/channels/{recipient}/messages"),
                body: json!({"content": text, "allowed_mentions": {"parse": []}}),
                authorization: OutboundAuthorization::DiscordBot(token),
            })
        }
        ChannelKind::Whatsapp => {
            let phone_id = env_value(
                config.outbound_phone_number_id_env.as_deref(),
                "outbound_phone_number_id_env",
            )?;
            let version = config.outbound_api_version.as_deref().unwrap_or("v25.0");
            Ok(OutboundRequest {
                channel: "whatsapp".into(),
                recipient: recipient.into(),
                endpoint: format!("https://graph.facebook.com/{version}/{phone_id}/messages"),
                body: json!({
                    "messaging_product": "whatsapp",
                    "recipient_type": "individual",
                    "to": recipient,
                    "type": "text",
                    "text": {"body": text, "preview_url": false}
                }),
                authorization: OutboundAuthorization::Bearer(token),
            })
        }
        ChannelKind::Generic => {
            let endpoint = env_value(config.outbound_endpoint_env.as_deref(), "outbound_endpoint_env")?;
            if !endpoint.starts_with("https://") {
                anyhow::bail!("generic outbound endpoint must use HTTPS");
            }
            Ok(OutboundRequest {
                channel: "generic".into(),
                recipient: recipient.into(),
                endpoint,
                body: json!({"recipient": recipient, "text": text}),
                authorization: OutboundAuthorization::Bearer(token),
            })
        }
        ChannelKind::Line | ChannelKind::Signal => anyhow::bail!(
            "outbound {} delivery is not available in the core; use a reviewed local adapter plugin",
            channel_name(channel)
        ),
    }
}

fn env_value(name: Option<&str>, setting: &str) -> Result<String> {
    let name = name.context(format!("{setting} must name an environment variable"))?;
    env::var(name).with_context(|| format!("set {name} before outbound delivery"))
}

fn channel_name(channel: ChannelKind) -> &'static str {
    match channel {
        ChannelKind::Telegram => "telegram",
        ChannelKind::Discord => "discord",
        ChannelKind::Whatsapp => "whatsapp",
        ChannelKind::Line => "line",
        ChannelKind::Signal => "signal",
        ChannelKind::Generic => "generic",
    }
}

fn summarize_response(response: &str) -> String {
    let compact = response.split_whitespace().collect::<Vec<_>>().join(" ");
    compact.chars().take(200).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MockTransport {
        requests: Mutex<Vec<OutboundRequest>>,
    }

    #[async_trait]
    impl OutboundTransport for MockTransport {
        async fn send(&self, request: OutboundRequest) -> Result<OutboundDelivery> {
            let delivery = OutboundDelivery {
                channel: request.channel.clone(),
                recipient: request.recipient.clone(),
                status: 200,
                response_summary: "mocked".into(),
            };
            self.requests.lock().unwrap().push(request);
            Ok(delivery)
        }
    }

    fn channel_config(recipient: &str, token_env: &str) -> ChannelConfig {
        ChannelConfig {
            allowed_recipients: vec![recipient.into()],
            outbound_token_env: Some(token_env.into()),
            ..ChannelConfig::default()
        }
    }

    #[tokio::test]
    async fn sends_telegram_only_to_a_trusted_recipient() {
        std::env::set_var("TC_TEST_TELEGRAM_TOKEN", "test-token");
        let transport = MockTransport::default();
        let delivery = send_message(
            ChannelKind::Telegram,
            &channel_config("42", "TC_TEST_TELEGRAM_TOKEN"),
            "42",
            "hello",
            &transport,
        )
        .await
        .unwrap();
        std::env::remove_var("TC_TEST_TELEGRAM_TOKEN");
        assert_eq!(delivery.status, 200);
        let request = transport.requests.lock().unwrap().pop().unwrap();
        assert_eq!(request.body["chat_id"], "42");
        assert_eq!(request.body["text"], "hello");
    }

    #[tokio::test]
    async fn refuses_an_untrusted_recipient_before_transport() {
        std::env::set_var("TC_TEST_DISCORD_TOKEN", "test-token");
        let transport = MockTransport::default();
        let result = send_message(
            ChannelKind::Discord,
            &channel_config("123", "TC_TEST_DISCORD_TOKEN"),
            "999",
            "hello",
            &transport,
        )
        .await;
        std::env::remove_var("TC_TEST_DISCORD_TOKEN");
        assert!(result.is_err());
        assert!(transport.requests.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn builds_a_whatsapp_text_request_from_environment_only() {
        std::env::set_var("TC_TEST_WHATSAPP_TOKEN", "test-token");
        std::env::set_var("TC_TEST_WHATSAPP_PHONE", "123456789");
        let transport = MockTransport::default();
        let mut config = channel_config("15555550123", "TC_TEST_WHATSAPP_TOKEN");
        config.outbound_phone_number_id_env = Some("TC_TEST_WHATSAPP_PHONE".into());
        let result = send_message(
            ChannelKind::Whatsapp,
            &config,
            "15555550123",
            "hello",
            &transport,
        )
        .await;
        std::env::remove_var("TC_TEST_WHATSAPP_TOKEN");
        std::env::remove_var("TC_TEST_WHATSAPP_PHONE");
        result.unwrap();
        let request = transport.requests.lock().unwrap().pop().unwrap();
        assert!(request.endpoint.ends_with("/123456789/messages"));
        assert_eq!(request.body["text"]["body"], "hello");
    }
}

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{env, fs, path::PathBuf};

use crate::config::ChannelConfig;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChannelKind {
    Telegram,
    Discord,
    Whatsapp,
    Line,
    Signal,
    Generic,
}

impl std::str::FromStr for ChannelKind {
    type Err = anyhow::Error;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().replace('-', "_").as_str() {
            "telegram" => Ok(Self::Telegram),
            "discord" => Ok(Self::Discord),
            "whatsapp" | "whatsapp_cloud" => Ok(Self::Whatsapp),
            "line" => Ok(Self::Line),
            "signal" => Ok(Self::Signal),
            "generic" | "stdin" => Ok(Self::Generic),
            _ => anyhow::bail!("unsupported bridge channel: {value}"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InboundMessage {
    pub id: String,
    pub channel: String,
    pub sender_id: String,
    pub conversation_id: Option<String>,
    pub text: String,
    pub received_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PairedSender {
    pub channel: String,
    pub sender_id: String,
    pub paired_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct PairingStore {
    path: PathBuf,
}

impl PairingStore {
    pub fn local() -> Self {
        let root = env::var("TC_HOME").map(PathBuf::from).unwrap_or_else(|_| {
            dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("thinking-computer")
        });
        Self {
            path: root.join("paired-senders.json"),
        }
    }

    pub fn pair(&self, channel: &str, sender_id: &str) -> Result<PairedSender> {
        let mut records = self.list()?;
        if let Some(record) = records
            .iter()
            .find(|record| record.channel == channel && record.sender_id == sender_id)
        {
            return Ok(record.clone());
        }
        let record = PairedSender {
            channel: channel.to_string(),
            sender_id: sender_id.to_string(),
            paired_at: Utc::now(),
        };
        records.push(record.clone());
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&self.path, serde_json::to_string_pretty(&records)?)?;
        Ok(record)
    }

    pub fn contains(&self, channel: &str, sender_id: &str) -> Result<bool> {
        Ok(self
            .list()?
            .iter()
            .any(|record| record.channel == channel && record.sender_id == sender_id))
    }

    pub fn list(&self) -> Result<Vec<PairedSender>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        Ok(serde_json::from_str(&fs::read_to_string(&self.path)?)?)
    }
}

pub fn normalize(channel: ChannelKind, body: &str) -> Result<InboundMessage> {
    let payload: Value = serde_json::from_str(body).context("bridge payload is not valid JSON")?;
    let received_at = Utc::now();
    match channel {
        ChannelKind::Telegram => {
            let message = payload
                .get("message")
                .or_else(|| payload.get("business_message"))
                .context("Telegram update has no supported message")?;
            Ok(InboundMessage {
                id: payload
                    .get("update_id")
                    .map(ToString::to_string)
                    .unwrap_or_else(|| {
                        Utc::now()
                            .timestamp_nanos_opt()
                            .unwrap_or_default()
                            .to_string()
                    }),
                channel: "telegram".into(),
                sender_id: value_string(message.pointer("/from/id"))?,
                conversation_id: message.pointer("/chat/id").map(ToString::to_string),
                text: value_string(message.pointer("/text"))?,
                received_at,
            })
        }
        ChannelKind::Discord => {
            let sender = payload
                .pointer("/member/user/id")
                .or_else(|| payload.pointer("/user/id"))
                .context("Discord interaction has no sender id")?;
            let name = value_string(payload.pointer("/data/name"))?;
            let text = payload
                .pointer("/data/options/0/value")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
                .unwrap_or(name);
            Ok(InboundMessage {
                id: value_string(payload.get("id"))?,
                channel: "discord".into(),
                sender_id: value_string(Some(sender))?,
                conversation_id: payload.get("channel_id").map(ToString::to_string),
                text,
                received_at,
            })
        }
        ChannelKind::Whatsapp => {
            let message = payload
                .pointer("/entry/0/changes/0/value/messages/0")
                .context("WhatsApp webhook has no inbound message")?;
            Ok(InboundMessage {
                id: value_string(message.get("id"))?,
                channel: "whatsapp".into(),
                sender_id: value_string(message.get("from"))?,
                conversation_id: message.get("from").map(ToString::to_string),
                text: value_string(message.pointer("/text/body"))?,
                received_at,
            })
        }
        ChannelKind::Line => {
            let event = payload
                .pointer("/events/0")
                .context("LINE webhook has no event")?;
            Ok(InboundMessage {
                id: value_string(event.get("webhookEventId"))?,
                channel: "line".into(),
                sender_id: value_string(event.pointer("/source/userId"))?,
                conversation_id: event.pointer("/source/groupId").map(ToString::to_string),
                text: value_string(event.pointer("/message/text"))?,
                received_at,
            })
        }
        ChannelKind::Signal => Ok(InboundMessage {
            id: value_string(payload.get("id"))?,
            channel: "signal".into(),
            sender_id: value_string(payload.get("sender_id"))?,
            conversation_id: payload
                .get("conversation_id")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),
            text: value_string(payload.get("text"))?,
            received_at,
        }),
        ChannelKind::Generic => Ok(InboundMessage {
            id: value_string(payload.get("id"))?,
            channel: "generic".into(),
            sender_id: value_string(payload.get("sender_id"))?,
            conversation_id: payload
                .get("conversation_id")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),
            text: value_string(payload.get("text"))?,
            received_at,
        }),
    }
}

pub fn sender_is_trusted(config: Option<&ChannelConfig>, message: &InboundMessage) -> bool {
    config
        .map(|channel| {
            channel
                .allowed_senders
                .iter()
                .any(|sender| sender == &message.sender_id)
        })
        .unwrap_or(false)
}

fn value_string(value: Option<&Value>) -> Result<String> {
    match value {
        Some(Value::String(value)) => Ok(value.clone()),
        Some(Value::Number(value)) => Ok(value.to_string()),
        _ => anyhow::bail!("channel payload is missing a required string or number field"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normalizes_a_telegram_message() {
        let message = normalize(
            ChannelKind::Telegram,
            r#"{"update_id":1,"message":{"from":{"id":99},"chat":{"id":22},"text":"hello"}}"#,
        )
        .unwrap();
        assert_eq!(message.sender_id, "99");
        assert_eq!(message.text, "hello");
    }
}

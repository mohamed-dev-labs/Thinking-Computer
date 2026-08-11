use crate::{
    config::{ProviderKind, ResolvedProvider},
    model::{ChatMessage, Role, ToolCall, ToolDefinition},
};
use anyhow::{Context, Result};
use reqwest::{Client, Url};
use serde_json::{json, Value};
use uuid::Uuid;

const ANTHROPIC_VERSION: &str = "2023-06-01";

pub async fn complete(
    provider: &ResolvedProvider,
    messages: &[ChatMessage],
    tools: &[ToolDefinition],
) -> Result<ChatMessage> {
    match provider.kind {
        ProviderKind::Openai => openai(provider, messages, tools).await,
        ProviderKind::Anthropic => anthropic(provider, messages, tools).await,
        ProviderKind::Gemini => gemini(provider, messages, tools).await,
        ProviderKind::Ollama => ollama(provider, messages, tools).await,
        _ if provider.kind.uses_openai_compatible_transport() => {
            openai(provider, messages, tools).await
        }
        _ => anyhow::bail!(
            "provider transport is not implemented: {}",
            provider.kind.as_str()
        ),
    }
}

fn remote_key(provider: &ResolvedProvider) -> Result<&str> {
    provider.api_key.as_deref().context(format!(
        "{} requires an API key; use an environment variable or local configuration",
        provider.kind.as_str()
    ))
}

fn message_text(message: &ChatMessage) -> Value {
    match message.role {
        Role::Tool => {
            json!({"role":"tool","tool_call_id":message.tool_call_id,"content":message.content})
        }
        Role::Assistant if !message.tool_calls.is_empty() => json!({
            "role":"assistant", "content": if message.content.is_empty() { Value::Null } else { Value::String(message.content.clone()) },
            "tool_calls": message.tool_calls.iter().map(|call| json!({"id":call.id,"type":"function","function":{"name":call.name,"arguments":call.arguments.to_string()}})).collect::<Vec<_>>()
        }),
        Role::System => json!({"role":"system","content":message.content}),
        Role::User => json!({"role":"user","content":message.content}),
        Role::Assistant => json!({"role":"assistant","content":message.content}),
    }
}

fn openai_tools(tools: &[ToolDefinition]) -> Vec<Value> {
    tools.iter().map(|tool| json!({"type":"function","function":{"name":tool.name,"description":tool.description,"parameters":tool.parameters}})).collect()
}

async fn json_response(response: reqwest::Response) -> Result<Value> {
    let status = response.status();
    let body = response.text().await?;
    if !status.is_success() {
        anyhow::bail!("provider request failed ({status}): {body}");
    }
    serde_json::from_str(&body).context("provider returned invalid JSON")
}

async fn openai(
    provider: &ResolvedProvider,
    messages: &[ChatMessage],
    tools: &[ToolDefinition],
) -> Result<ChatMessage> {
    let endpoint = provider
        .base_url
        .clone()
        .context("OpenAI-compatible provider has no chat-completions endpoint")?;
    let body = json!({"model":provider.model,"messages":messages.iter().map(message_text).collect::<Vec<_>>(),"tools":openai_tools(tools),"tool_choice":"auto"});
    let mut request = Client::new()
        .post(endpoint)
        .bearer_auth(remote_key(provider)?)
        .json(&body);
    for (name, value) in &provider.headers {
        request = request.header(name, value);
    }
    let value = json_response(request.send().await?).await?;
    let message = value
        .pointer("/choices/0/message")
        .context("OpenAI response has no message")?;
    let tool_calls = message
        .get("tool_calls")
        .and_then(Value::as_array)
        .unwrap_or(&Vec::new())
        .iter()
        .map(|call| ToolCall {
            id: call
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            name: call
                .pointer("/function/name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            arguments: serde_json::from_str(
                call.pointer("/function/arguments")
                    .and_then(Value::as_str)
                    .unwrap_or("{}"),
            )
            .unwrap_or_else(|_| json!({})),
        })
        .collect();
    Ok(ChatMessage {
        role: Role::Assistant,
        content: message
            .get("content")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        tool_calls,
        tool_call_id: None,
        tool_name: None,
    })
}

fn anthropic_message(message: &ChatMessage) -> Option<Value> {
    match message.role {
        Role::System => None,
        Role::User => Some(json!({"role":"user","content":message.content})),
        Role::Assistant => {
            let mut content = Vec::new();
            if !message.content.is_empty() {
                content.push(json!({"type":"text","text":message.content}));
            }
            content.extend(message.tool_calls.iter().map(|call| json!({"type":"tool_use","id":call.id,"name":call.name,"input":call.arguments})));
            Some(json!({"role":"assistant","content":content}))
        }
        Role::Tool => Some(
            json!({"role":"user","content":[{"type":"tool_result","tool_use_id":message.tool_call_id,"content":message.content}]}),
        ),
    }
}

async fn anthropic(
    provider: &ResolvedProvider,
    messages: &[ChatMessage],
    tools: &[ToolDefinition],
) -> Result<ChatMessage> {
    let endpoint = provider
        .base_url
        .clone()
        .unwrap_or_else(|| "https://api.anthropic.com/v1/messages".into());
    let system = messages
        .iter()
        .filter(|message| message.role == Role::System)
        .map(|message| message.content.clone())
        .collect::<Vec<_>>()
        .join("\n");
    let body = json!({
        "model":provider.model, "max_tokens":2048, "system":system,
        "messages":messages.iter().filter_map(anthropic_message).collect::<Vec<_>>(),
        "tools":tools.iter().map(|tool| json!({"name":tool.name,"description":tool.description,"input_schema":tool.parameters})).collect::<Vec<_>>(),
        "tool_choice":{"type":"auto","disable_parallel_tool_use":true}
    });
    let value = json_response(
        Client::new()
            .post(endpoint)
            .header("x-api-key", remote_key(provider)?)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .json(&body)
            .send()
            .await?,
    )
    .await?;
    let blocks = value
        .get("content")
        .and_then(Value::as_array)
        .context("Anthropic response has no content")?;
    let content = blocks
        .iter()
        .filter(|block| block.get("type").and_then(Value::as_str) == Some("text"))
        .filter_map(|block| block.get("text").and_then(Value::as_str))
        .collect::<Vec<_>>()
        .join("\n");
    let tool_calls = blocks
        .iter()
        .filter(|block| block.get("type").and_then(Value::as_str) == Some("tool_use"))
        .map(|block| ToolCall {
            id: block
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            name: block
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            arguments: block.get("input").cloned().unwrap_or_else(|| json!({})),
        })
        .collect();
    Ok(ChatMessage {
        role: Role::Assistant,
        content,
        tool_calls,
        tool_call_id: None,
        tool_name: None,
    })
}

fn gemini_contents(messages: &[ChatMessage]) -> Vec<Value> {
    messages.iter().filter(|message| message.role != Role::System).map(|message| match message.role {
        Role::User => json!({"role":"user","parts":[{"text":message.content}]}),
        Role::Assistant => {
            let mut parts = Vec::new();
            if !message.content.is_empty() { parts.push(json!({"text":message.content})); }
            parts.extend(message.tool_calls.iter().map(|call| json!({"functionCall":{"name":call.name,"args":call.arguments}})));
            json!({"role":"model","parts":parts})
        }
        Role::Tool => json!({"role":"user","parts":[{"functionResponse":{"name":message.tool_name,"response":{"result":message.content}}}]}),
        Role::System => unreachable!(),
    }).collect()
}

async fn gemini(
    provider: &ResolvedProvider,
    messages: &[ChatMessage],
    tools: &[ToolDefinition],
) -> Result<ChatMessage> {
    let mut endpoint = Url::parse(&provider.base_url.clone().unwrap_or_else(|| {
        format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            provider.model
        )
    }))?;
    endpoint
        .query_pairs_mut()
        .append_pair("key", remote_key(provider)?);
    let instruction = messages
        .iter()
        .filter(|message| message.role == Role::System)
        .map(|message| message.content.clone())
        .collect::<Vec<_>>()
        .join("\n");
    let body = json!({
        "system_instruction":{"parts":[{"text":instruction}]}, "contents":gemini_contents(messages),
        "tools":[{"functionDeclarations":tools.iter().map(|tool| json!({"name":tool.name,"description":tool.description,"parameters":tool.parameters})).collect::<Vec<_>>() }]
    });
    let value = json_response(Client::new().post(endpoint).json(&body).send().await?).await?;
    let parts = value
        .pointer("/candidates/0/content/parts")
        .and_then(Value::as_array)
        .context("Gemini response has no content parts")?;
    let content = parts
        .iter()
        .filter_map(|part| part.get("text").and_then(Value::as_str))
        .collect::<Vec<_>>()
        .join("\n");
    let tool_calls = parts
        .iter()
        .filter_map(|part| part.get("functionCall"))
        .map(|call| ToolCall {
            id: Uuid::new_v4().to_string(),
            name: call
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            arguments: call.get("args").cloned().unwrap_or_else(|| json!({})),
        })
        .collect();
    Ok(ChatMessage {
        role: Role::Assistant,
        content,
        tool_calls,
        tool_call_id: None,
        tool_name: None,
    })
}

async fn ollama(
    provider: &ResolvedProvider,
    messages: &[ChatMessage],
    tools: &[ToolDefinition],
) -> Result<ChatMessage> {
    let endpoint = format!(
        "{}/api/chat",
        provider
            .base_url
            .as_deref()
            .unwrap_or("http://127.0.0.1:11434")
            .trim_end_matches('/')
    );
    let body = json!({"model":provider.model,"stream":false,"messages":messages.iter().map(message_text).collect::<Vec<_>>(),"tools":openai_tools(tools)});
    let value = json_response(Client::new().post(endpoint).json(&body).send().await?).await?;
    let message = value
        .get("message")
        .context("Ollama response has no message")?;
    let tool_calls = message
        .get("tool_calls")
        .and_then(Value::as_array)
        .unwrap_or(&Vec::new())
        .iter()
        .map(|call| ToolCall {
            id: Uuid::new_v4().to_string(),
            name: call
                .pointer("/function/name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            arguments: call
                .pointer("/function/arguments")
                .cloned()
                .unwrap_or_else(|| json!({})),
        })
        .collect();
    Ok(ChatMessage {
        role: Role::Assistant,
        content: message
            .get("content")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        tool_calls,
        tool_call_id: None,
        tool_name: None,
    })
}

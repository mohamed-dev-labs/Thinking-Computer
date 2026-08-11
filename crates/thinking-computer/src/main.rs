use anyhow::Context;
use anyhow::Result;
use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::post,
    Router,
};
use clap::{Parser, Subcommand};
use rustyline::DefaultEditor;
use serde_json::{json, Value};
use std::{
    io::{self, BufRead, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tc_core::{
    config::ProviderKind, normalize_bridge_message, plugin::discover_plugins, recipient_is_trusted,
    send_outbound_message, sender_is_trusted, system_summary, Agent, AgentMemory, AppConfig,
    Approval, ApprovalRequest, Capability, ChannelKind, HttpOutboundTransport, PairingStore,
    PermissionPolicy, PluginManifest, PluginStore, PluginTool, RegistrationTarget, ScheduleStore,
    SessionStore, SkillManifest, SkillStore, ToolExecutor, VmCapabilityProfile,
};

mod tui;

#[derive(Debug, Parser)]
#[command(
    name = "thinking-computer",
    version,
    about = "A local-first personal agent for your terminal"
)]
struct Cli {
    #[arg(long, global = true, env = "TC_CONFIG")]
    config: Option<PathBuf>,
    #[arg(long, global = true, env = "TC_WORKSPACE")]
    workspace: Option<PathBuf>,
    #[arg(
        long,
        global = true,
        help = "Auto-approve guarded tool actions for this run"
    )]
    yes: bool,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    Repl {
        #[arg(short, long)]
        provider: Option<String>,
        #[arg(short, long)]
        model: Option<String>,
    },
    Chat {
        #[arg(short, long)]
        provider: Option<String>,
        #[arg(short, long)]
        model: Option<String>,
        #[arg(long)]
        session: Option<String>,
        #[arg(required = true, trailing_var_arg = true)]
        prompt: Vec<String>,
    },
    Init,
    Config,
    Services,
    Tui,
    Plugins {
        #[command(subcommand)]
        command: PluginCommand,
    },
    Skills {
        #[command(subcommand)]
        command: SkillCommand,
    },
    Schedule {
        #[command(subcommand)]
        command: ScheduleCommand,
    },
    Memory {
        #[command(subcommand)]
        command: MemoryCommand,
    },
    Bridge {
        #[command(subcommand)]
        command: BridgeCommand,
    },
    Webhook {
        #[command(subcommand)]
        command: WebhookCommand,
    },
    /// Read JSON requests from stdin and return JSON responses for the Python/Hermes adapter.
    Rpc,
    Doctor,
}

#[derive(Debug, Subcommand)]
enum PluginCommand {
    List,
    Create {
        #[arg(long)]
        name: String,
        #[arg(long, default_value = "0.1.0")]
        version: String,
        #[arg(long = "tool", required = true)]
        tools: Vec<String>,
    },
}

#[derive(Debug, Subcommand)]
enum SkillCommand {
    List,
    Create {
        #[arg(long)]
        name: String,
        #[arg(long, default_value = "0.1.0")]
        version: String,
        #[arg(long)]
        description: String,
        #[arg(long)]
        instructions: String,
        #[arg(long = "capability")]
        capabilities: Vec<String>,
    },
}

#[derive(Debug, Subcommand)]
enum ScheduleCommand {
    List,
    Add {
        #[arg(long)]
        name: String,
        #[arg(long)]
        cron: String,
        #[arg(long)]
        provider: Option<String>,
        #[arg(long)]
        model: Option<String>,
        #[arg(required = true, trailing_var_arg = true)]
        prompt: Vec<String>,
    },
    Remove {
        #[arg(long)]
        name: String,
    },
    Export {
        #[arg(long, default_value = "linux")]
        target: String,
    },
}

#[derive(Debug, Subcommand)]
enum MemoryCommand {
    Profile,
    Recall {
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
}

#[derive(Debug, Subcommand)]
enum BridgeCommand {
    Pair {
        #[arg(long)]
        channel: String,
        #[arg(long)]
        sender: String,
    },
    Inspect {
        #[arg(long)]
        channel: String,
        #[arg(long)]
        payload: PathBuf,
    },
    Dispatch {
        #[arg(long)]
        channel: String,
        #[arg(long)]
        payload: PathBuf,
        #[arg(short, long)]
        provider: Option<String>,
        #[arg(short, long)]
        model: Option<String>,
        #[arg(long)]
        session: Option<String>,
    },
    Send {
        #[arg(long)]
        channel: String,
        #[arg(long)]
        recipient: String,
        #[arg(long)]
        message: String,
    },
}

#[derive(Debug, Subcommand)]
enum WebhookCommand {
    /// Starts only when explicitly invoked. Received events are verified and audited, not auto-dispatched.
    Listen {
        #[arg(long)]
        channel: String,
        #[arg(long, default_value = "127.0.0.1:8787")]
        bind: String,
    },
}

#[derive(Clone)]
struct WebhookState {
    kind: ChannelKind,
    channel: String,
    secret: String,
    allowed_senders: Vec<String>,
    replay: Arc<Mutex<tc_core::ReplayGuard>>,
}

struct TerminalApproval;
impl Approval for TerminalApproval {
    fn approve(&self, request: &ApprovalRequest) -> Result<bool> {
        use std::io::{self, Write};
        print!(
            "\nPermission required [{:?}]\n{}\nApprove? [y/N] ",
            request.capability, request.summary
        );
        io::stdout().flush()?;
        let mut answer = String::new();
        io::stdin().read_line(&mut answer)?;
        Ok(matches!(
            answer.trim().to_ascii_lowercase().as_str(),
            "y" | "yes"
        ))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = AppConfig::load_or_default(cli.config.as_deref())?;
    let default_command = Command::Repl {
        provider: None,
        model: None,
    };
    match cli.command.as_ref().unwrap_or(&default_command) {
        Command::Init => {
            let path = cli.config.clone().unwrap_or_else(AppConfig::config_path);
            if path.exists() {
                anyhow::bail!("configuration already exists at {}", path.display());
            }
            AppConfig::write_example(&path)?;
            println!("Created {}", path.display());
            let plugin = install_bundled_sample_plugin(&state_root())?;
            println!("Installed sample plugin at {}", plugin.display());
        }
        Command::Config => {
            let provider = config.resolve_provider(None, None)?;
            println!("default provider: {}", provider.kind.as_str());
            println!("model: {}", provider.model);
            println!(
                "API key: {}",
                if provider.api_key.is_some() {
                    "configured (redacted)"
                } else {
                    "not configured"
                }
            );
            println!(
                "workspace: {}",
                cli.workspace
                    .clone()
                    .or(config.workspace.clone())
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "current directory".into())
            );
        }
        Command::Services => {
            if config.services.is_empty() {
                println!("No optional services configured. Add [services.<name>] to config.toml.");
            }
            for name in config.services.keys() {
                let service = config.resolve_service(name)?;
                println!(
                    "{} | protocol={} | endpoint={} | API key={}",
                    service.name,
                    service.protocol,
                    service.base_url.as_deref().unwrap_or("not set"),
                    if service.api_key.is_some() {
                        "configured (redacted)"
                    } else {
                        "not configured"
                    }
                );
            }
        }
        Command::Tui => tui::run()?,
        Command::Plugins { command } => plugins(command)?,
        Command::Skills { command } => skills(command)?,
        Command::Schedule { command } => schedule(command)?,
        Command::Memory { command } => memory(command, cli.yes)?,
        Command::Bridge { command } => bridge(command, &config, &cli).await?,
        Command::Webhook { command } => webhook(command, &config).await?,
        Command::Rpc => rpc(&config, &cli).await?,
        Command::Doctor => {
            println!("platform: {}", system_summary());
            println!(
                "config: {}",
                cli.config
                    .clone()
                    .unwrap_or_else(AppConfig::config_path)
                    .display()
            );
            println!("session storage: {}", state_root().display());
            println!("Node.js plugins: optional; required only when invoking plugin tools");
        }
        Command::Chat {
            provider,
            model,
            session,
            prompt,
        } => {
            let answer = agent_for(
                &config,
                &cli,
                provider.as_deref(),
                model.as_deref(),
                session.clone(),
            )
            .await?
            .run(&prompt.join(" "))
            .await?;
            println!("\n{answer}");
        }
        Command::Repl { provider, model } => {
            repl(&config, &cli, provider.as_deref(), model.as_deref()).await?
        }
    }
    Ok(())
}

async fn webhook(command: &WebhookCommand, config: &AppConfig) -> Result<()> {
    match command {
        WebhookCommand::Listen { channel, bind } => {
            let kind: ChannelKind = channel.parse()?;
            let name = channel_name(kind).to_string();
            let channel_config = config.channels.get(&name).context(
                "configure [channels.<name>] with allowed_senders and webhook_secret_env before listening",
            )?;
            let secret_env = channel_config
                .webhook_secret_env
                .as_deref()
                .context("webhook_secret_env is required for a listener")?;
            let secret = std::env::var(secret_env)
                .with_context(|| format!("set {secret_env} before listening"))?;
            let state = WebhookState {
                kind,
                channel: name,
                secret,
                allowed_senders: channel_config.allowed_senders.clone(),
                replay: Arc::new(Mutex::new(tc_core::ReplayGuard::new(10_000))),
            };
            let listener = tokio::net::TcpListener::bind(bind)
                .await
                .with_context(|| format!("cannot bind webhook listener on {bind}"))?;
            println!(
                "Listening for verified {} webhooks on http://{} (no auto-dispatch).",
                state.channel, bind
            );
            axum::serve(
                listener,
                Router::new()
                    .route("/webhook", post(receive_webhook))
                    .with_state(state),
            )
            .await?;
        }
    }
    Ok(())
}

fn skills(command: &SkillCommand) -> Result<()> {
    let store = SkillStore::local();
    match command {
        SkillCommand::List => {
            println!("{}", serde_json::to_string_pretty(&store.list()?)?);
        }
        SkillCommand::Create {
            name,
            version,
            description,
            instructions,
            capabilities,
        } => {
            let skill = store.create(SkillManifest {
                name: name.clone(),
                version: version.clone(),
                description: description.clone(),
                instructions: instructions.clone(),
                capabilities: capabilities.clone(),
                created_at: chrono::Utc::now(),
            })?;
            println!("{}", serde_json::to_string_pretty(&skill)?);
        }
    }
    Ok(())
}

fn channel_name(kind: ChannelKind) -> &'static str {
    match kind {
        ChannelKind::Telegram => "telegram",
        ChannelKind::Discord => "discord",
        ChannelKind::Whatsapp => "whatsapp",
        ChannelKind::Line => "line",
        ChannelKind::Signal => "signal",
        ChannelKind::Generic => "generic",
    }
}

async fn receive_webhook(
    State(state): State<WebhookState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, StatusCode> {
    verify_webhook_request(&state, &headers, &body)?;
    let message = normalize_bridge_message(
        state.kind,
        std::str::from_utf8(&body).map_err(|_| StatusCode::BAD_REQUEST)?,
    )
    .map_err(|_| StatusCode::BAD_REQUEST)?;
    let trusted_by_config = state
        .allowed_senders
        .iter()
        .any(|sender| sender == &message.sender_id);
    let trusted_by_pairing = PairingStore::local()
        .contains(&message.channel, &message.sender_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if !trusted_by_config && !trusted_by_pairing {
        return Err(StatusCode::FORBIDDEN);
    }
    let replay_id = format!("{}:{}", state.channel, message.id);
    if !state
        .replay
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .accept_once(replay_id)
    {
        return Err(StatusCode::CONFLICT);
    }
    AgentMemory::local()
        .and_then(|memory| {
            memory.append_audit(
                "webhook_accepted",
                &format!(
                    "channel={}; sender={}; message_id={}",
                    message.channel, message.sender_id, message.id
                ),
                true,
            )
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!({
        "accepted": true,
        "message_id": message.id,
        "dispatched": false,
        "next_step": "Use bridge dispatch after local review."
    })))
}

fn verify_webhook_request(
    state: &WebhookState,
    headers: &HeaderMap,
    body: &[u8],
) -> Result<(), StatusCode> {
    let header = |name: &str| {
        headers
            .get(name)
            .and_then(|value| value.to_str().ok())
            .ok_or(StatusCode::UNAUTHORIZED)
    };
    match state.kind {
        ChannelKind::Telegram => {
            if header("x-telegram-bot-api-secret-token")? == state.secret {
                Ok(())
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        ChannelKind::Discord => tc_core::verify_discord_ed25519(
            &state.secret,
            header("x-signature-ed25519")?,
            header("x-signature-timestamp")?,
            body,
        )
        .map_err(|_| StatusCode::UNAUTHORIZED),
        _ => {
            let signature = headers
                .get("x-hub-signature-256")
                .or_else(|| headers.get("x-thinking-computer-signature"))
                .and_then(|value| value.to_str().ok())
                .ok_or(StatusCode::UNAUTHORIZED)?;
            tc_core::verify_hmac_sha256(&state.secret, body, signature)
                .map_err(|_| StatusCode::UNAUTHORIZED)
        }
    }
}

fn memory(command: &MemoryCommand, assume_yes: bool) -> Result<()> {
    let memory = AgentMemory::local()?;
    match command {
        MemoryCommand::Profile => {
            if !assume_yes && !TerminalApproval.approve(&ApprovalRequest { capability: Capability::InspectSystem, summary: "Capture OS, CPU, memory, disk, process, and installed-tool indicators for the current VM".into() })? { anyhow::bail!("VM profile capture was denied by the user"); }
            let profile = VmCapabilityProfile::collect();
            memory.save_capability_profile(&profile)?;
            println!("{}", serde_json::to_string_pretty(&profile)?);
        }
        MemoryCommand::Recall { limit } => {
            let value = json!({"capability_profile": memory.load_capability_profile()?, "knowledge": memory.recall(*limit)?});
            println!("{}", serde_json::to_string_pretty(&value)?);
        }
    }
    Ok(())
}

async fn bridge(command: &BridgeCommand, config: &AppConfig, cli: &Cli) -> Result<()> {
    match command {
        BridgeCommand::Pair { channel, sender } => {
            let kind: ChannelKind = channel.parse()?;
            let record = PairingStore::local().pair(channel_name(kind), sender)?;
            println!("{}", serde_json::to_string_pretty(&record)?);
        }
        BridgeCommand::Inspect { channel, payload } => {
            let kind: ChannelKind = channel.parse()?;
            let message = normalize_bridge_message(kind, &std::fs::read_to_string(payload)?)?;
            let policy = config.channels.get(&message.channel);
            let trusted = sender_is_trusted(policy, &message)
                || PairingStore::local().contains(&message.channel, &message.sender_id)?;
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &json!({"message": message, "trusted_sender": trusted, "next_step": if trusted {"review and explicitly dispatch through the Rust engine"} else {"deny: sender is not in the local allowlist"}})
                )?
            );
        }
        BridgeCommand::Dispatch {
            channel,
            payload,
            provider,
            model,
            session,
        } => {
            let kind: ChannelKind = channel.parse()?;
            let message = normalize_bridge_message(kind, &std::fs::read_to_string(payload)?)?;
            let trusted = sender_is_trusted(config.channels.get(&message.channel), &message)
                || PairingStore::local().contains(&message.channel, &message.sender_id)?;
            if !trusted {
                anyhow::bail!(
                    "inbound message denied: sender {} is not in the {} allowlist",
                    message.sender_id,
                    message.channel
                );
            }
            if !cli.yes
                && !TerminalApproval.approve(&ApprovalRequest {
                    capability: Capability::BridgeDispatch,
                    summary: format!(
                    "Dispatch inbound {} message from trusted sender {} to the Rust agent engine",
                    message.channel, message.sender_id
                ),
                })?
            {
                anyhow::bail!("inbound message dispatch was denied by the user");
            }
            let answer = agent_for(
                config,
                cli,
                provider.as_deref(),
                model.as_deref(),
                session.clone(),
            )
            .await?
            .run(&message.text)
            .await?;
            AgentMemory::local()?.append_audit(
                "bridge_dispatch",
                &format!(
                    "channel={}; sender={}; message_id={}",
                    message.channel, message.sender_id, message.id
                ),
                true,
            )?;
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &json!({"message_id": message.id, "channel": message.channel, "answer": answer})
                )?
            );
        }
        BridgeCommand::Send {
            channel,
            recipient,
            message,
        } => {
            let kind: ChannelKind = channel.parse()?;
            let name = channel_name(kind);
            let channel_config = config
                .channels
                .get(name)
                .context("configure this channel before sending messages")?;
            if !recipient_is_trusted(Some(channel_config), recipient) {
                anyhow::bail!(
                    "outbound message denied: recipient {} is not in the {} allowlist",
                    recipient,
                    name
                );
            }
            if !cli.yes
                && !TerminalApproval.approve(&ApprovalRequest {
                    capability: Capability::BridgeDispatch,
                    summary: format!(
                        "Send {} characters to trusted {} recipient {}",
                        message.chars().count(),
                        name,
                        recipient
                    ),
                })?
            {
                anyhow::bail!("outbound message delivery was denied by the user");
            }
            let delivery = send_outbound_message(
                kind,
                channel_config,
                recipient,
                message,
                &HttpOutboundTransport::default(),
            )
            .await?;
            AgentMemory::local()?.append_audit(
                "bridge_send",
                &format!(
                    "channel={}; recipient={}; chars={}; status={}",
                    delivery.channel,
                    delivery.recipient,
                    message.chars().count(),
                    delivery.status
                ),
                true,
            )?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "channel": delivery.channel,
                    "recipient": delivery.recipient,
                    "status": delivery.status,
                    "response": delivery.response_summary,
                }))?
            );
        }
    }
    Ok(())
}

fn schedule(command: &ScheduleCommand) -> Result<()> {
    let store = ScheduleStore::local()?;
    match command {
        ScheduleCommand::List => {
            let tasks = store.list()?;
            if tasks.is_empty() {
                println!("No local schedule definitions found.");
            }
            for task in tasks {
                println!(
                    "{} | {} | {} | enabled={}",
                    task.name,
                    task.cron,
                    task.provider.unwrap_or_else(|| "default".into()),
                    task.enabled
                );
            }
        }
        ScheduleCommand::Add {
            name,
            cron,
            provider,
            model,
            prompt,
        } => {
            let task = store.add(
                name,
                cron,
                &prompt.join(" "),
                provider.clone(),
                model.clone(),
            )?;
            println!("Saved local schedule definition {}. It has not been registered with the operating system.", task.name);
            println!(
                "To register it on a Unix VM, review and add this line to your crontab:\n{}",
                store.cron_command(&task)
            );
        }
        ScheduleCommand::Remove { name } => {
            if store.remove(name)? {
                println!("Removed local schedule definition {name}.");
            } else {
                println!("No local schedule named {name}.");
            }
        }
        ScheduleCommand::Export { target } => {
            let target: RegistrationTarget = target.parse()?;
            for task in store.list()? {
                println!("{}", store.registration_template(&task, target));
            }
        }
    }
    Ok(())
}

async fn rpc(config: &AppConfig, cli: &Cli) -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let request: Value = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(error) => {
                writeln!(
                    stdout,
                    "{}",
                    json!({"ok": false, "error": format!("invalid JSON request: {error}")})
                )?;
                stdout.flush()?;
                continue;
            }
        };
        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let result = async {
            let prompt = request
                .get("prompt")
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow::anyhow!("prompt must be a string"))?;
            let provider = request.get("provider").and_then(Value::as_str);
            let model = request.get("model").and_then(Value::as_str);
            let session = request
                .get("session")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            let answer = agent_for(config, cli, provider, model, session)
                .await?
                .run(prompt)
                .await?;
            Ok::<Value, anyhow::Error>(json!({"text": answer}))
        }
        .await;
        let response = match result {
            Ok(value) => json!({"id": id, "ok": true, "result": value}),
            Err(error) => json!({"id": id, "ok": false, "error": error.to_string()}),
        };
        writeln!(stdout, "{response}")?;
        stdout.flush()?;
    }
    Ok(())
}

async fn agent_for(
    config: &AppConfig,
    cli: &Cli,
    provider: Option<&str>,
    model: Option<&str>,
    session: Option<String>,
) -> Result<Agent> {
    let provider = config.resolve_provider(provider, model)?;
    if provider.kind != ProviderKind::Ollama && provider.api_key.is_none() {
        anyhow::bail!(
            "{} has no API key. Set {} or configure it in the local config file.",
            provider.kind.as_str(),
            provider.kind.env_key().unwrap_or("the provider key")
        );
    }
    let workspace = cli
        .workspace
        .clone()
        .or_else(|| config.workspace.clone())
        .unwrap_or(std::env::current_dir()?);
    let mut policy = PermissionPolicy::with_read_access();
    policy.assume_yes = cli.yes;
    let tools = ToolExecutor::new(workspace, policy, Arc::new(TerminalApproval))?;
    Ok(Agent::new(
        provider,
        tools,
        SessionStore::local(session)?,
        config.max_steps,
    ))
}

async fn repl(
    config: &AppConfig,
    cli: &Cli,
    provider: Option<&str>,
    model: Option<&str>,
) -> Result<()> {
    let agent = agent_for(config, cli, provider, model, None).await?;
    let mut editor = DefaultEditor::new()?;
    println!(
        "Thinking Computer REPL — session {}. Type /help for commands, /exit to leave.",
        agent.session_id()
    );
    loop {
        match editor.readline("tc> ") {
            Ok(line) => {
                let task = line.trim();
                if task.is_empty() {
                    continue;
                }
                editor.add_history_entry(task)?;
                match task {
                    "/exit" | "/quit" => break,
                    "/help" => {
                        println!("/help, /exit — enter a task to call the configured model.")
                    }
                    _ => match agent.run(task).await {
                        Ok(answer) => println!("\n{answer}\n"),
                        Err(error) => eprintln!("\nError: {error}\n"),
                    },
                }
            }
            Err(rustyline::error::ReadlineError::Interrupted) => continue,
            Err(rustyline::error::ReadlineError::Eof) => break,
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

fn state_root() -> PathBuf {
    std::env::var("TC_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("thinking-computer")
        })
}
fn install_bundled_sample_plugin(root: &std::path::Path) -> Result<PathBuf> {
    let plugin = root.join("plugins").join("hello-plugin");
    if !plugin.exists() {
        std::fs::create_dir_all(&plugin)?;
        std::fs::write(
            plugin.join("thinking-computer-plugin.json"),
            include_str!("../../../plugins/hello-plugin/thinking-computer-plugin.json"),
        )?;
        std::fs::write(
            plugin.join("index.mjs"),
            include_str!("../../../plugins/hello-plugin/index.mjs"),
        )?;
    }
    Ok(plugin)
}
fn list_plugins() -> Result<()> {
    let root = state_root().join("plugins");
    let plugins = discover_plugins(&root)?;
    if plugins.is_empty() {
        println!("No local plugins found in {}", root.display());
    }
    for (path, manifest) in plugins {
        println!(
            "{} {} — {} tool(s) [{}]",
            manifest.name,
            manifest.version,
            manifest.tools.len(),
            path.display()
        );
    }
    Ok(())
}

fn plugins(command: &PluginCommand) -> Result<()> {
    match command {
        PluginCommand::List => list_plugins(),
        PluginCommand::Create {
            name,
            version,
            tools,
        } => {
            let manifest = PluginManifest {
                name: name.clone(),
                version: version.clone(),
                entry: "index.mjs".into(),
                tools: tools
                    .iter()
                    .map(|name| PluginTool {
                        name: name.clone(),
                        description: "Generated template tool; implement only after local review."
                            .into(),
                        parameters: json!({"type": "object", "properties": {}}),
                        capabilities: vec![],
                    })
                    .collect(),
            };
            let result = PluginStore::local().create(manifest.clone());
            let success = result.is_ok();
            AgentMemory::local()?.append_audit(
                "plugin_create",
                &format!(
                    "name={}; version={}; tools={}",
                    manifest.name,
                    manifest.version,
                    manifest.tools.len()
                ),
                success,
            )?;
            let path = result?;
            println!(
                "Created {}. The generated tool template has no privileged behavior; review and implement it before invocation.",
                path.display()
            );
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bundled_plugin_is_discoverable_from_normal_state_directory() {
        let temp = tempfile::tempdir().unwrap();
        install_bundled_sample_plugin(temp.path()).unwrap();
        let plugins = discover_plugins(temp.path().join("plugins")).unwrap();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].1.name, "hello-plugin");
    }
}

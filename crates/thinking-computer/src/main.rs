use anyhow::Result;
use clap::{Parser, Subcommand};
use rustyline::DefaultEditor;
use std::{path::PathBuf, sync::Arc};
use tc_core::{Agent, Approval, ApprovalRequest, AppConfig, PermissionPolicy, SessionStore, ToolExecutor, config::ProviderKind, plugin::discover_plugins, system_summary};

#[derive(Debug, Parser)]
#[command(name = "thinking-computer", version, about = "A local-first personal agent for your terminal")]
struct Cli {
    #[arg(long, global = true, env = "TC_CONFIG")]
    config: Option<PathBuf>,
    #[arg(long, global = true, env = "TC_WORKSPACE")]
    workspace: Option<PathBuf>,
    #[arg(long, global = true, help = "Auto-approve guarded tool actions for this run")]
    yes: bool,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    Repl { #[arg(short, long)] provider: Option<String>, #[arg(short, long)] model: Option<String> },
    Chat { #[arg(short, long)] provider: Option<String>, #[arg(short, long)] model: Option<String>, #[arg(long)] session: Option<String>, #[arg(required = true, trailing_var_arg = true)] prompt: Vec<String> },
    Init,
    Config,
    Plugins { #[command(subcommand)] command: PluginCommand },
    Doctor,
}

#[derive(Debug, Subcommand)]
enum PluginCommand { List }

struct TerminalApproval;
impl Approval for TerminalApproval {
    fn approve(&self, request: &ApprovalRequest) -> Result<bool> {
        use std::io::{self, Write};
        print!("\nPermission required [{:?}]\n{}\nApprove? [y/N] ", request.capability, request.summary);
        io::stdout().flush()?;
        let mut answer = String::new();
        io::stdin().read_line(&mut answer)?;
        Ok(matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes"))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = AppConfig::load_or_default(cli.config.as_deref())?;
    let default_command = Command::Repl { provider: None, model: None };
    match cli.command.as_ref().unwrap_or(&default_command) {
        Command::Init => {
            let path = cli.config.clone().unwrap_or_else(AppConfig::config_path);
            if path.exists() { anyhow::bail!("configuration already exists at {}", path.display()); }
            AppConfig::write_example(&path)?;
            println!("Created {}", path.display());
            let plugin = install_bundled_sample_plugin(&state_root())?;
            println!("Installed sample plugin at {}", plugin.display());
        }
        Command::Config => {
            let provider = config.resolve_provider(None, None)?;
            println!("default provider: {}", provider.kind.as_str());
            println!("model: {}", provider.model);
            println!("API key: {}", if provider.api_key.is_some() { "configured (redacted)" } else { "not configured" });
            println!("workspace: {}", cli.workspace.clone().or(config.workspace.clone()).map(|path| path.display().to_string()).unwrap_or_else(|| "current directory".into()));
        }
        Command::Plugins { command: PluginCommand::List } => list_plugins()?,
        Command::Doctor => {
            println!("platform: {}", system_summary());
            println!("config: {}", cli.config.clone().unwrap_or_else(AppConfig::config_path).display());
            println!("session storage: {}", state_root().display());
            println!("Node.js plugins: optional; required only when invoking plugin tools");
        }
        Command::Chat { provider, model, session, prompt } => {
            let answer = agent_for(&config, &cli, provider.as_deref(), model.as_deref(), session.clone()).await?.run(&prompt.join(" ")).await?;
            println!("\n{answer}");
        }
        Command::Repl { provider, model } => repl(&config, &cli, provider.as_deref(), model.as_deref()).await?,
    }
    Ok(())
}

async fn agent_for(config: &AppConfig, cli: &Cli, provider: Option<&str>, model: Option<&str>, session: Option<String>) -> Result<Agent> {
    let provider = config.resolve_provider(provider, model)?;
    if provider.kind != ProviderKind::Ollama && provider.api_key.is_none() { anyhow::bail!("{} has no API key. Set {} or configure it in the local config file.", provider.kind.as_str(), provider.kind.env_key().unwrap_or("the provider key")); }
    let workspace = cli.workspace.clone().or_else(|| config.workspace.clone()).unwrap_or(std::env::current_dir()?);
    let mut policy = PermissionPolicy::with_read_access();
    policy.assume_yes = cli.yes;
    let tools = ToolExecutor::new(workspace, policy, Arc::new(TerminalApproval))?;
    Ok(Agent::new(provider, tools, SessionStore::local(session)?, config.max_steps))
}

async fn repl(config: &AppConfig, cli: &Cli, provider: Option<&str>, model: Option<&str>) -> Result<()> {
    let agent = agent_for(config, cli, provider, model, None).await?;
    let mut editor = DefaultEditor::new()?;
    println!("Thinking Computer REPL — session {}. Type /help for commands, /exit to leave.", agent.session_id());
    loop {
        match editor.readline("tc> ") {
            Ok(line) => {
                let task = line.trim();
                if task.is_empty() { continue; }
                editor.add_history_entry(task)?;
                match task {
                    "/exit" | "/quit" => break,
                    "/help" => println!("/help, /exit — enter a task to call the configured model."),
                    _ => match agent.run(task).await { Ok(answer) => println!("\n{answer}\n"), Err(error) => eprintln!("\nError: {error}\n") },
                }
            }
            Err(rustyline::error::ReadlineError::Interrupted) => continue,
            Err(rustyline::error::ReadlineError::Eof) => break,
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

fn state_root() -> PathBuf { std::env::var("TC_HOME").map(PathBuf::from).unwrap_or_else(|_| dirs::data_local_dir().unwrap_or_else(|| PathBuf::from(".")).join("thinking-computer")) }
fn install_bundled_sample_plugin(root: &std::path::Path) -> Result<PathBuf> {
    let plugin = root.join("plugins").join("hello-plugin");
    if !plugin.exists() {
        std::fs::create_dir_all(&plugin)?;
        std::fs::write(plugin.join("thinking-computer-plugin.json"), include_str!("../../../plugins/hello-plugin/thinking-computer-plugin.json"))?;
        std::fs::write(plugin.join("index.mjs"), include_str!("../../../plugins/hello-plugin/index.mjs"))?;
    }
    Ok(plugin)
}
fn list_plugins() -> Result<()> {
    let root = state_root().join("plugins");
    let plugins = discover_plugins(&root)?;
    if plugins.is_empty() { println!("No local plugins found in {}", root.display()); }
    for (path, manifest) in plugins { println!("{} {} — {} tool(s) [{}]", manifest.name, manifest.version, manifest.tools.len(), path.display()); }
    Ok(())
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

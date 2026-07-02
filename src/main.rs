use std::io::Write;

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::tty::IsTty;
use todo_in_cli::{
    agent,
    cli::{
        AgentCommand, ApiCommand, Cli, Command, GithubCommand, RoadmapCommand, SyncKind,
        TodoCommand,
    },
    github, llm,
    storage::Store,
    tui,
};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Command::Tui {
        provider: todo_in_cli::cli::ProviderKind::Openai,
        no_tmux_follow_active_pane: false,
    }) {
        Command::Tui {
            provider,
            no_tmux_follow_active_pane,
        } => {
            let project = {
                let mut store = Store::open_default_locked()?;
                let project = store.ensure_current_project()?;
                store.save()?;
                project
            };
            tui::run(project, provider, !no_tmux_follow_active_pane)?;
        }
        Command::Todo { command } => match command {
            TodoCommand::Add { title } => {
                let mut store = Store::open_default_locked()?;
                let project = store.ensure_current_project()?;
                let todo = store.add_todo(&project.id, title)?;
                store.save()?;
                println!("created todo {}: {}", todo.id, todo.title);
            }
            TodoCommand::Edit { id, title } => {
                let mut store = Store::open_default_locked()?;
                let project = store.ensure_current_project()?;
                let todo = store.update_todo_title(&project.id, &id, title)?;
                store.save()?;
                println!("updated todo {}: {}", todo.id, todo.title);
            }
            TodoCommand::List => {
                let mut store = Store::open_default()?;
                let project = store.ensure_current_project()?;
                for todo in store.todos_for_project(&project.id) {
                    let status = if todo.completed { "done" } else { "open" };
                    println!("{} [{}] {}", todo.id, status, todo.title);
                }
            }
            TodoCommand::Done { id } => {
                let mut store = Store::open_default_locked()?;
                let project = store.ensure_current_project()?;
                store.complete_todo(&project.id, &id)?;
                store.save()?;
                println!("completed todo {id}");
            }
            TodoCommand::Delete { id } => {
                let mut store = Store::open_default_locked()?;
                let project = store.ensure_current_project()?;
                let todo = store.delete_todo(&project.id, &id)?;
                store.save()?;
                println!("deleted todo {}: {}", todo.id, todo.title);
            }
        },
        Command::Roadmap { command } => match command {
            RoadmapCommand::Add { title } => {
                let mut store = Store::open_default_locked()?;
                let project = store.ensure_current_project()?;
                let item = store.add_roadmap_item(&project.id, title)?;
                store.save()?;
                println!("created roadmap item {}: {}", item.id, item.title);
            }
            RoadmapCommand::Edit { id, title, status } => {
                let mut store = Store::open_default_locked()?;
                let project = store.ensure_current_project()?;
                let item = store.update_roadmap_item(&project.id, &id, title, status)?;
                store.save()?;
                println!(
                    "updated roadmap item {} [{}] {}",
                    item.id, item.status, item.title
                );
            }
            RoadmapCommand::List => {
                let mut store = Store::open_default()?;
                let project = store.ensure_current_project()?;
                for item in store.roadmap_for_project(&project.id) {
                    println!("{} [{}] {}", item.id, item.status, item.title);
                }
            }
            RoadmapCommand::Delete { id } => {
                let mut store = Store::open_default_locked()?;
                let project = store.ensure_current_project()?;
                let item = store.delete_roadmap_item(&project.id, &id)?;
                store.save()?;
                println!("deleted roadmap item {}: {}", item.id, item.title);
            }
        },
        Command::Chat { provider, message } => {
            let mut store = Store::open_default()?;
            let project = store.ensure_current_project()?;
            let client = llm::provider_from_env(provider)?;
            let response = client
                .chat(llm::ChatRequest {
                    project_name: project.name.clone(),
                    message: message.clone(),
                })
                .await?;
            let mut store = Store::open_default_locked()?;
            let project = store.ensure_current_project()?;
            store.add_chat_message(&project.id, "user", message)?;
            store.add_chat_message(&project.id, "assistant", response.message.clone())?;
            store.save()?;
            println!("{}", response.message);
        }
        Command::Agent { command } => match command {
            AgentCommand::Propose { json } => {
                let mut store = Store::open_default_locked()?;
                let project = store.ensure_current_project()?;
                let actions = agent::parse_actions(&json)?;
                let created = store.queue_agent_actions(&project.id, actions)?;
                store.save()?;
                for action in created {
                    println!("queued action {}: {}", action.id, action.tool.summary());
                }
            }
            AgentCommand::List => {
                let mut store = Store::open_default()?;
                let project = store.ensure_current_project()?;
                for action in store.agent_actions_for_project(&project.id) {
                    println!(
                        "{} [{}] {}",
                        action.id,
                        action.status.as_str(),
                        action.tool.summary()
                    );
                }
            }
            AgentCommand::Approve { id } => {
                require_human_approval(&id)?;
                let mut store = Store::open_default_locked()?;
                let project = store.ensure_current_project()?;
                let outcome = store.approve_agent_action(&project.id, &id)?;
                store.save()?;
                println!("{outcome}");
            }
            AgentCommand::Reject { id, reason } => {
                let mut store = Store::open_default_locked()?;
                let project = store.ensure_current_project()?;
                store.reject_agent_action(&project.id, &id, reason)?;
                store.save()?;
                println!("rejected action {id}");
            }
        },
        Command::Api { command } => match command {
            ApiCommand::Snapshot => {
                let mut store = Store::open_default()?;
                let project = store.ensure_current_project()?;
                println!("{}", todo_in_cli::api::snapshot_json(&store, &project.id)?);
            }
            ApiCommand::Manifest => {
                println!("{}", todo_in_cli::api::manifest_json()?);
            }
        },
        Command::Github { command } => match command {
            GithubCommand::Sync {
                kind,
                dry_run,
                pull,
            } => {
                let kind = match kind {
                    SyncKind::Todos => github::SyncKind::Todos,
                    SyncKind::Roadmap => github::SyncKind::Roadmap,
                    SyncKind::All => github::SyncKind::All,
                };
                let report = github::sync_issues(kind, dry_run, pull)?;
                for line in report.lines {
                    println!("{line}");
                }
            }
            GithubCommand::Pull { dry_run } => {
                let report = github::pull_issues(dry_run)?;
                for line in report.lines {
                    println!("{line}");
                }
            }
        },
    }

    Ok(())
}

fn require_human_approval(id: &str) -> Result<()> {
    if std::env::var("TODO_IN_CLI_ALLOW_NONINTERACTIVE_APPROVAL").as_deref() == Ok("1") {
        return Ok(());
    }

    if !std::io::stdin().is_tty() || !std::io::stderr().is_tty() {
        anyhow::bail!(
            "agent approval requires an interactive terminal; plugins may propose actions but cannot approve them"
        );
    }

    eprint!("Approve agent action {id}? Type 'approve' to continue: ");
    std::io::stderr()
        .flush()
        .context("failed to flush prompt")?;

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .context("failed to read approval confirmation")?;

    if input.trim() != "approve" {
        anyhow::bail!("approval cancelled");
    }

    Ok(())
}

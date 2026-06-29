mod cli;
mod llm;
mod models;
mod storage;
mod tui;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command, RoadmapCommand, TodoCommand};
use storage::Store;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Command::Tui) {
        Command::Tui => {
            let mut store = Store::open_default()?;
            let project = store.ensure_current_project()?;
            tui::run(&store, project)?;
        }
        Command::Todo { command } => match command {
            TodoCommand::Add { title } => {
                let mut store = Store::open_default_locked()?;
                let project = store.ensure_current_project()?;
                let todo = store.add_todo(&project.id, title)?;
                store.save()?;
                println!("created todo {}: {}", todo.id, todo.title);
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
        },
        Command::Roadmap { command } => match command {
            RoadmapCommand::Add { title } => {
                let mut store = Store::open_default_locked()?;
                let project = store.ensure_current_project()?;
                let item = store.add_roadmap_item(&project.id, title)?;
                store.save()?;
                println!("created roadmap item {}: {}", item.id, item.title);
            }
            RoadmapCommand::List => {
                let mut store = Store::open_default()?;
                let project = store.ensure_current_project()?;
                for item in store.roadmap_for_project(&project.id) {
                    println!("{} [{}] {}", item.id, item.status, item.title);
                }
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
    }

    Ok(())
}

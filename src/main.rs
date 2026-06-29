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
    let mut store = Store::open_default()?;
    let project = store.ensure_current_project()?;

    match cli.command.unwrap_or(Command::Tui) {
        Command::Tui => tui::run(&store, project)?,
        Command::Todo { command } => match command {
            TodoCommand::Add { title } => {
                let todo = store.add_todo(&project.id, title)?;
                println!("created todo {}: {}", todo.id, todo.title);
            }
            TodoCommand::List => {
                for todo in store.todos_for_project(&project.id) {
                    let status = if todo.completed { "done" } else { "open" };
                    println!("{} [{}] {}", todo.id, status, todo.title);
                }
            }
            TodoCommand::Done { id } => {
                store.complete_todo(&project.id, &id)?;
                println!("completed todo {id}");
            }
        },
        Command::Roadmap { command } => match command {
            RoadmapCommand::Add { title } => {
                let item = store.add_roadmap_item(&project.id, title)?;
                println!("created roadmap item {}: {}", item.id, item.title);
            }
            RoadmapCommand::List => {
                for item in store.roadmap_for_project(&project.id) {
                    println!("{} [{}] {}", item.id, item.status, item.title);
                }
            }
        },
        Command::Chat { provider, message } => {
            let client = llm::provider_from_env(provider)?;
            let response = client
                .chat(llm::ChatRequest {
                    project_name: project.name.clone(),
                    message: message.clone(),
                })
                .await?;
            store.add_chat_message(&project.id, "user", message)?;
            store.add_chat_message(&project.id, "assistant", response.message.clone())?;
            println!("{}", response.message);
        }
    }

    store.save()?;
    Ok(())
}

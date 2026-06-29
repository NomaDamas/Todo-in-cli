use anyhow::Result;
use serde::Serialize;

use crate::storage::Store;

#[derive(Serialize)]
struct Manifest {
    name: &'static str,
    version: &'static str,
    commands: Vec<CommandSchema>,
}

#[derive(Serialize)]
struct CommandSchema {
    name: &'static str,
    description: &'static str,
}

pub fn manifest_json() -> Result<String> {
    let manifest = Manifest {
        name: "todo-in-cli",
        version: env!("CARGO_PKG_VERSION"),
        commands: vec![
            CommandSchema {
                name: "api snapshot",
                description: "Read project-scoped todos, roadmap items, chat messages, and agent actions as JSON.",
            },
            CommandSchema {
                name: "agent propose",
                description: "Queue structured agent actions for explicit user approval.",
            },
            CommandSchema {
                name: "github sync",
                description: "Publish unsynced local todos and roadmap items to GitHub Issues.",
            },
        ],
    };
    Ok(serde_json::to_string_pretty(&manifest)?)
}

pub fn snapshot_json(store: &Store, project_id: &str) -> Result<String> {
    Ok(serde_json::to_string_pretty(
        &store.project_snapshot(project_id)?,
    )?)
}

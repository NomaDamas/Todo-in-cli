use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use uuid::Uuid;

use crate::models::{AppState, ChatMessage, Project, RoadmapItem, Todo};

pub struct Store {
    path: PathBuf,
    state: AppState,
}

impl Store {
    pub fn open_default() -> Result<Self> {
        let home = env::var_os("TODO_IN_CLI_HOME")
            .map(PathBuf::from)
            .or_else(|| dirs::home_dir().map(|path| path.join(".todo-in-cli")))
            .context("could not determine home directory")?;
        fs::create_dir_all(&home)
            .with_context(|| format!("failed to create state directory {}", home.display()))?;
        Self::open(home.join("state.json"))
    }

    pub fn open(path: PathBuf) -> Result<Self> {
        let state = if path.exists() {
            let raw = fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            serde_json::from_str(&raw)
                .with_context(|| format!("failed to parse {}", path.display()))?
        } else {
            AppState::default()
        };

        Ok(Self { path, state })
    }

    pub fn save(&self) -> Result<()> {
        let raw = serde_json::to_string_pretty(&self.state)?;
        fs::write(&self.path, raw)
            .with_context(|| format!("failed to write {}", self.path.display()))
    }

    pub fn ensure_current_project(&mut self) -> Result<Project> {
        let root = current_project_root()?;
        let root_string = root.to_string_lossy().to_string();

        if let Some(project) = self
            .state
            .projects
            .iter()
            .find(|project| project.root == root_string)
        {
            return Ok(project.clone());
        }

        let name = root
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("workspace")
            .to_string();
        let project = Project {
            id: short_id(),
            name,
            root: root_string,
            created_at: Utc::now(),
        };
        self.state.projects.push(project.clone());
        Ok(project)
    }

    pub fn add_todo(&mut self, project_id: &str, title: String) -> Result<Todo> {
        ensure_non_empty(&title, "todo title")?;
        let todo = Todo {
            id: short_id(),
            project_id: project_id.to_string(),
            title,
            completed: false,
            created_at: Utc::now(),
            completed_at: None,
        };
        self.state.todos.push(todo.clone());
        Ok(todo)
    }

    pub fn complete_todo(&mut self, project_id: &str, id: &str) -> Result<()> {
        let todo = self
            .state
            .todos
            .iter_mut()
            .find(|todo| todo.project_id == project_id && todo.id == id)
            .ok_or_else(|| anyhow!("todo not found: {id}"))?;
        todo.completed = true;
        todo.completed_at = Some(Utc::now());
        Ok(())
    }

    pub fn todos_for_project(&self, project_id: &str) -> Vec<Todo> {
        self.state
            .todos
            .iter()
            .filter(|todo| todo.project_id == project_id)
            .cloned()
            .collect()
    }

    pub fn add_roadmap_item(&mut self, project_id: &str, title: String) -> Result<RoadmapItem> {
        ensure_non_empty(&title, "roadmap title")?;
        let item = RoadmapItem {
            id: short_id(),
            project_id: project_id.to_string(),
            title,
            status: "planned".to_string(),
            created_at: Utc::now(),
        };
        self.state.roadmap.push(item.clone());
        Ok(item)
    }

    pub fn roadmap_for_project(&self, project_id: &str) -> Vec<RoadmapItem> {
        self.state
            .roadmap
            .iter()
            .filter(|item| item.project_id == project_id)
            .cloned()
            .collect()
    }

    pub fn add_chat_message(
        &mut self,
        project_id: &str,
        role: &str,
        content: String,
    ) -> Result<()> {
        ensure_non_empty(&content, "chat message")?;
        self.state.chat_messages.push(ChatMessage {
            id: short_id(),
            project_id: project_id.to_string(),
            role: role.to_string(),
            content,
            created_at: Utc::now(),
        });
        Ok(())
    }

    pub fn chat_for_project(&self, project_id: &str) -> Vec<ChatMessage> {
        self.state
            .chat_messages
            .iter()
            .filter(|message| message.project_id == project_id)
            .cloned()
            .collect()
    }
}

fn current_project_root() -> Result<PathBuf> {
    if let Ok(output) = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        && output.status.success()
    {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            return Ok(PathBuf::from(path));
        }
    }

    env::current_dir().context("failed to determine current directory")
}

fn ensure_non_empty(value: &str, label: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(anyhow!("{label} cannot be empty"));
    }
    Ok(())
}

fn short_id() -> String {
    Uuid::new_v4().simple().to_string()[..8].to_string()
}

#[allow(dead_code)]
fn _is_inside(path: &Path, root: &Path) -> bool {
    path.starts_with(root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_todos() {
        let mut store = Store {
            path: PathBuf::from("unused"),
            state: AppState::default(),
        };

        let error = store.add_todo("project", "   ".to_string()).unwrap_err();
        assert!(error.to_string().contains("todo title cannot be empty"));
    }

    #[test]
    fn completes_project_scoped_todo() {
        let mut store = Store {
            path: PathBuf::from("unused"),
            state: AppState::default(),
        };

        let todo = store.add_todo("project", "ship".to_string()).unwrap();
        store.complete_todo("project", &todo.id).unwrap();

        let todos = store.todos_for_project("project");
        assert!(todos[0].completed);
        assert!(todos[0].completed_at.is_some());
    }
}

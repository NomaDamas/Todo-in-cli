use std::{
    env, fs,
    fs::{File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use fs2::FileExt;
use uuid::Uuid;

use crate::models::{
    AgentAction, AgentActionStatus, AgentTool, AppState, ChatMessage, Project, RoadmapItem, Todo,
};

pub struct Store {
    path: PathBuf,
    state: AppState,
    _lock: Option<File>,
}

impl Store {
    pub fn open_default() -> Result<Self> {
        Self::open(default_state_path()?)
    }

    pub fn open_default_locked() -> Result<Self> {
        let path = default_state_path()?;
        let lock_path = path.with_extension("lock");
        let lock = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(&lock_path)
            .with_context(|| format!("failed to open lock file {}", lock_path.display()))?;
        lock.lock_exclusive()
            .with_context(|| format!("failed to lock {}", lock_path.display()))?;
        let mut store = Self::open(path)?;
        store._lock = Some(lock);
        Ok(store)
    }

    pub fn open(path: PathBuf) -> Result<Self> {
        let state = read_state(&path)?;
        Ok(Self {
            path,
            state,
            _lock: None,
        })
    }

    pub fn save(&self) -> Result<()> {
        let parent = self
            .path
            .parent()
            .with_context(|| format!("state path has no parent: {}", self.path.display()))?;
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create state directory {}", parent.display()))?;

        if self.path.exists() {
            let backup = self.path.with_extension("json.bak");
            fs::copy(&self.path, &backup)
                .with_context(|| format!("failed to create backup {}", backup.display()))?;
        }

        let temp = self.path.with_extension("json.tmp");
        let raw = serde_json::to_string_pretty(&self.state)?;
        {
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&temp)
                .with_context(|| format!("failed to open temp state {}", temp.display()))?;
            file.write_all(raw.as_bytes())
                .with_context(|| format!("failed to write temp state {}", temp.display()))?;
            file.sync_all()
                .with_context(|| format!("failed to sync temp state {}", temp.display()))?;
        }
        fs::rename(&temp, &self.path).with_context(|| {
            format!(
                "failed to atomically replace {} with {}",
                self.path.display(),
                temp.display()
            )
        })?;

        if let Ok(dir) = File::open(parent) {
            let _ = dir.sync_all();
        }

        Ok(())
    }
}

fn default_state_path() -> Result<PathBuf> {
    let home = env::var_os("TODO_IN_CLI_HOME")
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|path| path.join(".todo-in-cli")))
        .context("could not determine home directory")?;
    fs::create_dir_all(&home)
        .with_context(|| format!("failed to create state directory {}", home.display()))?;
    Ok(home.join("state.json"))
}

fn read_state(path: &Path) -> Result<AppState> {
    if !path.exists() {
        return Ok(AppState::default());
    }

    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    match serde_json::from_str(&raw) {
        Ok(state) => Ok(state),
        Err(error) => {
            let backup = path.with_extension("json.bak");
            if backup.exists() {
                let raw = fs::read_to_string(&backup)
                    .with_context(|| format!("failed to read backup {}", backup.display()))?;
                return serde_json::from_str(&raw).with_context(|| {
                    format!(
                        "failed to parse {} and backup {}",
                        path.display(),
                        backup.display()
                    )
                });
            }
            Err(error).with_context(|| format!("failed to parse {}", path.display()))
        }
    }
}

impl Store {
    pub fn projects(&self) -> Vec<Project> {
        self.state.projects.clone()
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
            github_issue: None,
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
            github_issue: None,
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

    pub fn queue_agent_actions(
        &mut self,
        project_id: &str,
        tools: Vec<AgentTool>,
    ) -> Result<Vec<AgentAction>> {
        let created: Vec<AgentAction> = tools
            .into_iter()
            .map(|tool| AgentAction {
                id: short_id(),
                project_id: project_id.to_string(),
                tool,
                status: AgentActionStatus::Pending,
                result: None,
                rejection_reason: None,
                created_at: Utc::now(),
                decided_at: None,
            })
            .collect();
        self.state.agent_actions.extend(created.clone());
        Ok(created)
    }

    pub fn agent_actions_for_project(&self, project_id: &str) -> Vec<AgentAction> {
        self.state
            .agent_actions
            .iter()
            .filter(|action| action.project_id == project_id)
            .cloned()
            .collect()
    }

    pub fn approve_agent_action(&mut self, project_id: &str, id: &str) -> Result<String> {
        let index = self
            .state
            .agent_actions
            .iter()
            .position(|action| action.project_id == project_id && action.id == id)
            .ok_or_else(|| anyhow!("agent action not found: {id}"))?;

        if !matches!(
            self.state.agent_actions[index].status,
            AgentActionStatus::Pending
        ) {
            return Err(anyhow!("agent action {id} is not pending"));
        }

        let tool = self.state.agent_actions[index].tool.clone();
        let result = match tool {
            AgentTool::CreateTodo { title } => {
                let todo = self.add_todo(project_id, title)?;
                format!("created todo {}", todo.id)
            }
            AgentTool::CompleteTodo { id } => {
                self.complete_todo(project_id, &id)?;
                format!("completed todo {id}")
            }
            AgentTool::CreateRoadmapItem { title } => {
                let item = self.add_roadmap_item(project_id, title)?;
                format!("created roadmap item {}", item.id)
            }
            AgentTool::SummarizeProject => {
                let open = self
                    .todos_for_project(project_id)
                    .iter()
                    .filter(|todo| !todo.completed)
                    .count();
                let roadmap = self.roadmap_for_project(project_id).len();
                format!("project has {open} open todos and {roadmap} roadmap items")
            }
        };

        let action = &mut self.state.agent_actions[index];
        action.status = AgentActionStatus::Approved;
        action.result = Some(result.clone());
        action.decided_at = Some(Utc::now());
        Ok(result)
    }

    pub fn reject_agent_action(
        &mut self,
        project_id: &str,
        id: &str,
        reason: Option<String>,
    ) -> Result<()> {
        let action = self
            .state
            .agent_actions
            .iter_mut()
            .find(|action| action.project_id == project_id && action.id == id)
            .ok_or_else(|| anyhow!("agent action not found: {id}"))?;

        if !matches!(action.status, AgentActionStatus::Pending) {
            return Err(anyhow!("agent action {id} is not pending"));
        }

        action.status = AgentActionStatus::Rejected;
        action.rejection_reason = reason;
        action.decided_at = Some(Utc::now());
        Ok(())
    }

    pub fn unsynced_todos(&self, project_id: &str) -> Vec<Todo> {
        self.state
            .todos
            .iter()
            .filter(|todo| todo.project_id == project_id && todo.github_issue.is_none())
            .cloned()
            .collect()
    }

    pub fn unsynced_roadmap(&self, project_id: &str) -> Vec<RoadmapItem> {
        self.state
            .roadmap
            .iter()
            .filter(|item| item.project_id == project_id && item.github_issue.is_none())
            .cloned()
            .collect()
    }

    pub fn link_todo_issue(&mut self, project_id: &str, todo_id: &str, issue: u64) -> Result<()> {
        let todo = self
            .state
            .todos
            .iter_mut()
            .find(|todo| todo.project_id == project_id && todo.id == todo_id)
            .ok_or_else(|| anyhow!("todo not found: {todo_id}"))?;
        todo.github_issue = Some(issue);
        Ok(())
    }

    pub fn link_roadmap_issue(
        &mut self,
        project_id: &str,
        item_id: &str,
        issue: u64,
    ) -> Result<()> {
        let item = self
            .state
            .roadmap
            .iter_mut()
            .find(|item| item.project_id == project_id && item.id == item_id)
            .ok_or_else(|| anyhow!("roadmap item not found: {item_id}"))?;
        item.github_issue = Some(issue);
        Ok(())
    }

    pub fn project_snapshot(&self, project_id: &str) -> Result<ProjectSnapshot> {
        let project = self
            .state
            .projects
            .iter()
            .find(|project| project.id == project_id)
            .cloned()
            .ok_or_else(|| anyhow!("project not found: {project_id}"))?;

        Ok(ProjectSnapshot {
            project,
            todos: self.todos_for_project(project_id),
            roadmap: self.roadmap_for_project(project_id),
            chat_messages: self.chat_for_project(project_id),
            agent_actions: self.agent_actions_for_project(project_id),
        })
    }
}

#[derive(serde::Serialize)]
pub struct ProjectSnapshot {
    pub project: Project,
    pub todos: Vec<Todo>,
    pub roadmap: Vec<RoadmapItem>,
    pub chat_messages: Vec<ChatMessage>,
    pub agent_actions: Vec<AgentAction>,
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
            _lock: None,
        };

        let error = store.add_todo("project", "   ".to_string()).unwrap_err();
        assert!(error.to_string().contains("todo title cannot be empty"));
    }

    #[test]
    fn completes_project_scoped_todo() {
        let mut store = Store {
            path: PathBuf::from("unused"),
            state: AppState::default(),
            _lock: None,
        };

        let todo = store.add_todo("project", "ship".to_string()).unwrap();
        store.complete_todo("project", &todo.id).unwrap();

        let todos = store.todos_for_project("project");
        assert!(todos[0].completed);
        assert!(todos[0].completed_at.is_some());
    }

    #[test]
    fn approves_agent_action_and_applies_mutation() {
        let mut store = Store {
            path: PathBuf::from("unused"),
            state: AppState::default(),
            _lock: None,
        };

        let action = store
            .queue_agent_actions(
                "project",
                vec![AgentTool::CreateTodo {
                    title: "agent todo".to_string(),
                }],
            )
            .unwrap()
            .remove(0);
        let result = store.approve_agent_action("project", &action.id).unwrap();

        assert!(result.contains("created todo"));
        assert_eq!(store.todos_for_project("project")[0].title, "agent todo");
        assert!(matches!(
            store.agent_actions_for_project("project")[0].status,
            AgentActionStatus::Approved
        ));
    }

    #[test]
    fn links_github_issue_to_todo() {
        let mut store = Store {
            path: PathBuf::from("unused"),
            state: AppState::default(),
            _lock: None,
        };

        let todo = store.add_todo("project", "sync me".to_string()).unwrap();
        store.link_todo_issue("project", &todo.id, 9).unwrap();

        assert_eq!(store.todos_for_project("project")[0].github_issue, Some(9));
        assert!(store.unsynced_todos("project").is_empty());
    }

    #[test]
    fn recovers_from_backup_when_state_is_corrupt() {
        let dir = env::temp_dir().join(format!("todo-in-cli-test-{}", short_id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("state.json");
        let backup = dir.join("state.json.bak");
        fs::write(&path, "{").unwrap();
        fs::write(
            &backup,
            r#"{"projects":[],"todos":[],"roadmap":[],"chat_messages":[]}"#,
        )
        .unwrap();

        let store = Store::open(path).unwrap();

        assert!(store.state.projects.is_empty());
    }
}

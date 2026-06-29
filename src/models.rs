use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub root: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Todo {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub completed: bool,
    #[serde(default)]
    pub github_issue: Option<u64>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RoadmapItem {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub status: String,
    #[serde(default)]
    pub github_issue: Option<u64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChatMessage {
    pub id: String,
    pub project_id: String,
    pub role: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "tool", rename_all = "snake_case")]
pub enum AgentTool {
    CreateTodo { title: String },
    CompleteTodo { id: String },
    CreateRoadmapItem { title: String },
    SummarizeProject,
}

impl AgentTool {
    pub fn summary(&self) -> String {
        match self {
            Self::CreateTodo { title } => format!("create todo: {title}"),
            Self::CompleteTodo { id } => format!("complete todo: {id}"),
            Self::CreateRoadmapItem { title } => format!("create roadmap item: {title}"),
            Self::SummarizeProject => "summarize project".to_string(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentActionStatus {
    Pending,
    Approved,
    Rejected,
}

impl AgentActionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AgentAction {
    pub id: String,
    pub project_id: String,
    pub tool: AgentTool,
    pub status: AgentActionStatus,
    pub result: Option<String>,
    pub rejection_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub decided_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct AppState {
    pub projects: Vec<Project>,
    pub todos: Vec<Todo>,
    pub roadmap: Vec<RoadmapItem>,
    pub chat_messages: Vec<ChatMessage>,
    #[serde(default)]
    pub agent_actions: Vec<AgentAction>,
}

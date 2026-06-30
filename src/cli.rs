use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Open the terminal dashboard.
    Tui {
        #[arg(long, value_enum, default_value_t = ProviderKind::Openai)]
        provider: ProviderKind,
    },
    /// Manage project-scoped todos.
    Todo {
        #[command(subcommand)]
        command: TodoCommand,
    },
    /// Manage project roadmap items.
    Roadmap {
        #[command(subcommand)]
        command: RoadmapCommand,
    },
    /// Send a planning chat message to an LLM provider.
    Chat {
        #[arg(long, value_enum, default_value_t = ProviderKind::Openai)]
        provider: ProviderKind,
        message: String,
    },
    /// Queue, inspect, approve, and reject assistant-proposed actions.
    Agent {
        #[command(subcommand)]
        command: AgentCommand,
    },
    /// Machine-readable API for future Codex/Claude-style plugins.
    Api {
        #[command(subcommand)]
        command: ApiCommand,
    },
    /// Sync local planning items to GitHub Issues through gh.
    Github {
        #[command(subcommand)]
        command: GithubCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum TodoCommand {
    Add { title: String },
    List,
    Done { id: String },
}

#[derive(Debug, Subcommand)]
pub enum RoadmapCommand {
    Add { title: String },
    List,
}

#[derive(Debug, Subcommand)]
pub enum AgentCommand {
    /// Queue one or more proposed actions from JSON.
    Propose { json: String },
    /// List proposed and audited agent actions.
    List,
    /// Approve a pending action and apply its mutation.
    Approve { id: String },
    /// Reject a pending action without mutating project state.
    Reject {
        id: String,
        #[arg(long)]
        reason: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum ApiCommand {
    /// Print project state as stable JSON.
    Snapshot,
    /// Print plugin/API command manifest as stable JSON.
    Manifest,
}

#[derive(Debug, Subcommand)]
pub enum GithubCommand {
    /// Publish unsynced local todos and roadmap items to GitHub Issues.
    Sync {
        #[arg(long, value_enum, default_value_t = SyncKind::All)]
        kind: SyncKind,
        #[arg(long)]
        dry_run: bool,
        /// Also pull matching GitHub Issues back into local state.
        #[arg(long)]
        pull: bool,
    },
    /// Pull matching GitHub Issues back into local state.
    Pull {
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum SyncKind {
    All,
    Todos,
    Roadmap,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum ProviderKind {
    Openai,
    Claude,
    Gemini,
    Grok,
}

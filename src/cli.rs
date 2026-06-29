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
    Tui,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum ProviderKind {
    Openai,
    Claude,
    Gemini,
    Grok,
}

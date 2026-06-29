# Todo In CLI

Rust terminal workspace for project-scoped todos, roadmaps, and AI-assisted planning.

## Status

This repository is starting with an MVP implementation:

- Project-aware todos and roadmap items.
- tmux-friendly `ratatui` dashboard.
- CLI commands for local workflows.
- Provider abstraction for GPT, Claude, Gemini, and Grok chat.
- Agent approval flow for safe assistant-proposed mutations.
- Codex/Claude-style plugin integration surfaces.
- GitHub Issue sync through the authenticated `gh` CLI.

## Quick Start

```sh
cargo run -- todo add "Ship the TUI MVP"
cargo run -- roadmap add "v0.1: local todo and roadmap dashboard"
cargo run -- tui
```

## Commands

```sh
todo-in-cli tui
todo-in-cli todo add "Write acceptance tests"
todo-in-cli todo list
todo-in-cli todo done <todo-id>
todo-in-cli roadmap add "Provider abstraction"
todo-in-cli roadmap list
todo-in-cli chat --provider openai "Draft the next milestone"
todo-in-cli agent propose '{"tool":"create_todo","title":"Review approval flow"}'
todo-in-cli agent list
todo-in-cli agent approve <action-id> --user-confirmed
todo-in-cli api manifest
todo-in-cli api snapshot
todo-in-cli github sync --dry-run
todo-in-cli github sync --kind todos
```

## LLM Provider Environment Variables

```sh
OPENAI_API_KEY=...
ANTHROPIC_API_KEY=...
GEMINI_API_KEY=...
XAI_API_KEY=...
```

## Storage

Local state is stored under:

```text
~/.todo-in-cli/state.json
```

Set `TODO_IN_CLI_HOME` to override the storage directory.

## Agent Approval Flow

Agentic mutations are approval-gated. Assistants and plugins can queue structured actions, but state changes only happen after explicit user approval.

```sh
todo-in-cli agent propose '[{"tool":"create_todo","title":"Add release checklist"},{"tool":"create_roadmap_item","title":"v0.2 agent tools"}]'
todo-in-cli agent list
todo-in-cli agent approve <action-id> --user-confirmed
todo-in-cli agent reject <action-id> --reason "not needed"
```

Supported tools:

- `create_todo`
- `complete_todo`
- `create_roadmap_item`
- `summarize_project`

## Plugin And Agent Host Integration

The crate exposes reusable modules through `src/lib.rs`, and the CLI exposes stable JSON surfaces. Plugin hosts should propose actions, but they should not call approval commands; approvals are reserved for human-facing terminal or TUI surfaces.

```sh
todo-in-cli api manifest
todo-in-cli api snapshot
```

Codex CLI, Claude Code, or MCP-style plugin hosts should call these commands instead of scraping terminal UI output. See [docs/plugin-integration.md](docs/plugin-integration.md).

## GitHub Issue Sync

GitHub sync uses the authenticated `gh` CLI in the current git repository:

```sh
gh auth login
todo-in-cli github sync --dry-run
todo-in-cli github sync --kind all
```

Synced local items store their GitHub issue number to prevent duplicate publishes. See [docs/github-sync.md](docs/github-sync.md).

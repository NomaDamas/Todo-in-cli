# Todo In CLI

Rust terminal workspace for project-scoped todos, roadmaps, and AI-assisted planning.

## Status

This repository is starting with an MVP implementation:

- Project-aware todos and roadmap items.
- tmux-friendly `ratatui` dashboard.
- Mouse-click project switching in the TUI project pane.
- TUI Codex mode toggle with `x`.
- Terminal-friendly Markdown subset rendering in TUI text.
- CLI commands for local workflows.
- Provider abstraction for GPT, Claude, Gemini, and Grok chat.
- Agent approval flow for safe assistant-proposed mutations.
- Codex/Claude-style plugin integration surfaces.
- GitHub Issue sync through the authenticated `gh` CLI.

## Quick Start

```sh
cargo run -- todo add "Ship the TUI MVP" && cargo run -- roadmap add "v0.1: local todo and roadmap dashboard" && cargo run -- tui
```

## Commands

```sh
todo-in-cli tui
todo-in-cli tui --provider claude
todo-in-cli tui --tmux-follow-active-pane
todo-in-cli todo add "Write acceptance tests"
todo-in-cli todo edit <todo-id> "Write regression tests"
todo-in-cli todo list
todo-in-cli todo done <todo-id>
todo-in-cli todo delete <todo-id>
todo-in-cli roadmap add "Provider abstraction"
todo-in-cli roadmap edit <roadmap-id> "Provider abstraction and retries" --status in-progress
todo-in-cli roadmap list
todo-in-cli roadmap delete <roadmap-id>
todo-in-cli chat --provider openai "Draft the next milestone"
todo-in-cli agent propose '{"tool":"create_todo","title":"Review approval flow"}'
todo-in-cli agent list
todo-in-cli agent approve <action-id>
todo-in-cli api manifest
todo-in-cli api snapshot
todo-in-cli github sync --dry-run
todo-in-cli github sync --kind todos
todo-in-cli github sync --pull
todo-in-cli github pull --dry-run
```

## TUI Controls

```text
Tab / Shift-Tab  Move focus between panes
Mouse click      Focus a pane
tmux pane click  Follow that pane's current directory as the active project
Project click    Switch the active project in the in-app Project pane
Todos pane       Press a, type a todo title, Enter saves
Roadmap pane     Press a, type a roadmap title, Enter saves
x                Toggle Codex mode indicator
q / Esc / Ctrl-C Exit, Esc cancels an active add prompt
Chat pane        Type directly, Enter sends to the selected provider
```

When running inside tmux, `todo-in-cli tui` follows the active tmux pane by default. Click a
different tmux pane that is open in another project directory, and the TUI switches to that
project's todos, roadmap, and chat. The pane running `todo-in-cli tui` is ignored so clicking
back into the dashboard does not reset the selected project.

Inside the dashboard, clicking Todos, Roadmap, or Chat only changes focus. If tmux does not
change the active pane on mouse click, enable mouse support with `tmux set -g mouse on`.

The TUI renders a terminal-safe Markdown subset:

- headings: `#`, `##`, `###`
- bullets: `- item`
- numbered lists: `1. item`
- blockquotes: `> quote`
- bold: `**text**`
- inline code: `` `command` ``
- links: `[label](url)`
- strikethrough: `~~text~~`

When Codex mode is enabled with `x`, chat messages create a Codex handoff by default. To execute a local Codex CLI command from the TUI, explicitly set:

```sh
TODO_IN_CLI_CODEX_EXEC=1
TODO_IN_CLI_CODEX_COMMAND=codex
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
todo-in-cli agent approve <action-id>
todo-in-cli agent reject <action-id> --reason "not needed"
```

Supported tools:

- `create_todo`
- `complete_todo`
- `create_roadmap_item`
- `summarize_project`

## Plugin And Agent Host Integration

The crate exposes reusable modules through `src/lib.rs`, and the CLI exposes stable JSON surfaces. Plugin hosts should propose actions, but they cannot approve actions non-interactively; approvals require a human-facing terminal confirmation prompt.

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
todo-in-cli github sync --pull
todo-in-cli github pull
```

Synced local items store their GitHub issue number to prevent duplicate publishes. See [docs/github-sync.md](docs/github-sync.md).

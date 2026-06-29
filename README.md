# Todo In CLI

Rust terminal workspace for project-scoped todos, roadmaps, and AI-assisted planning.

## Status

This repository is starting with an MVP implementation:

- Project-aware todos and roadmap items.
- tmux-friendly `ratatui` dashboard.
- CLI commands for local workflows.
- Provider abstraction for GPT, Claude, Gemini, and Grok chat.
- Future-work tracks for Codex/Claude plugin integration and GitHub Issue sync.

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

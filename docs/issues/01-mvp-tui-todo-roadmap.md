# MVP: Rust TUI Todo And Roadmap Workspace

## Goal
Build the first production-usable Rust CLI/TUI that detects the current project, stores project-scoped todos and roadmap items, and renders them in a tmux-friendly terminal dashboard.

## User Value
Users can open the CLI inside a project directory and immediately see the work queue, roadmap, and project context without leaving the terminal.

## Scope
- Initialize a Rust binary crate.
- Add CLI commands for todo and roadmap CRUD.
- Add project identity detection from git root or current directory.
- Persist local state safely.
- Add a `ratatui` dashboard with keyboard and mouse pane focus.
- Make layout readable in tmux panes and narrow terminals.

## Acceptance Criteria
- `todo-in-cli todo add "message"` creates a todo under the current project.
- `todo-in-cli todo list` shows todos for the current project only.
- `todo-in-cli roadmap add "milestone"` creates a roadmap item.
- `todo-in-cli roadmap list` shows roadmap items for the current project only.
- `todo-in-cli tui` opens a terminal dashboard with Project, Todos, Roadmap, and Chat panels.
- Mouse clicks change active panels without crashing.
- The app exits cleanly with `q`, `Esc`, or `Ctrl-C`.
- State survives process restarts.

## Non-Goals
- Cloud sync.
- Multi-user collaboration.
- Full LLM tool execution.

## Validation
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- Manual smoke test for `todo add/list`, `roadmap add/list`, and `tui`.

## Risks
- Terminal mouse event behavior varies across terminals and tmux versions.
- Narrow panes can truncate content; the UI must degrade gracefully.

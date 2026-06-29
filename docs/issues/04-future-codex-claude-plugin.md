# Future Work: Codex CLI And Claude Code Plugin Integration

## Goal
Design this app so it can later be embedded into Codex CLI or Claude Code style plugin systems.

## User Value
The same todo/roadmap workspace can become a planning surface inside coding agents instead of remaining only a standalone CLI.

## Future Scope
- Extract core state and planning APIs into a reusable library crate.
- Add a stable JSON-RPC or MCP-compatible command surface.
- Define plugin metadata and lifecycle hooks.
- Support external agent calls to read project state, propose tasks, and append roadmap items.
- Keep the TUI as one frontend over the same core.

## Acceptance Criteria
- Core logic is callable without terminal UI dependencies.
- Agent-facing commands have stable request/response schemas.
- Plugin integration can run without storing provider secrets in plugin metadata.
- Documentation explains how Codex/Claude-style hosts should invoke the tool.

## Non-Goals
- Implementing the actual plugin in the MVP PR.
- Depending on any one proprietary agent host.

## Risks
- Plugin APIs evolve quickly.
- Overfitting to one host could make the core less reusable.

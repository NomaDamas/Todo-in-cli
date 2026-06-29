# Plugin Integration

`todo-in-cli` is designed as a standalone terminal app and as a reusable planning backend for agent hosts.

## Stable Surfaces

Use these commands from Codex CLI, Claude Code, MCP servers, or similar hosts:

```sh
todo-in-cli api manifest
todo-in-cli api snapshot
todo-in-cli agent propose '<json-action-or-array>'
```

## Action Schema

```json
[
  {
    "tool": "create_todo",
    "title": "Write release notes"
  },
  {
    "tool": "create_roadmap_item",
    "title": "v0.3 GitHub sync hardening"
  },
  {
    "tool": "complete_todo",
    "id": "abcd1234"
  },
  {
    "tool": "summarize_project"
  }
]
```

## Contract

- Hosts can read state through `api snapshot`.
- Hosts can propose actions through `agent propose`.
- Hosts must not expect proposed actions to mutate state.
- Users approve or reject actions explicitly through human-facing CLI/TUI flows.
- Hosts must not call `agent approve`; the CLI requires `--user-confirmed` as an explicit guardrail.
- Approved and rejected actions remain in local audit state.
- Secrets stay in environment variables and are never stored in plugin metadata.

## Rust API Boundary

The crate exposes reusable modules through `src/lib.rs`:

- `agent`: structured action parsing and validation.
- `api`: manifest and project snapshot JSON.
- `github`: GitHub Issue sync orchestration.
- `storage`: local state, audit log, and persistence.
- `llm`: provider abstraction for GPT, Claude, Gemini, and Grok.

The TUI is only one frontend over these modules.

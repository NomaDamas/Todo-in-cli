# Agentic Planning: Todo And Roadmap Tool Approval Flow

## Goal
Allow the chat assistant to propose structured todo and roadmap changes, then apply them only after explicit user approval.

## User Value
Users can ask the assistant to break down a project plan and safely convert approved suggestions into tracked work.

## Scope
- Define internal tool schemas: `create_todo`, `complete_todo`, `create_roadmap_item`, `summarize_project`.
- Add a pending action queue.
- Render proposed actions in the TUI.
- Require explicit approval before mutating state.
- Persist accepted and rejected agent actions for auditability.

## Acceptance Criteria
- The assistant can return proposed todo/roadmap actions in a structured format.
- The app displays proposed actions separately from committed state.
- Users can approve or reject proposed actions.
- Approved actions update local state and are recorded in an action log.
- Rejected actions are recorded without state mutation.

## Non-Goals
- Fully autonomous background agents.
- Shell command execution by the assistant.
- Writing source code files.

## Validation
- Unit tests for action parsing, approval, rejection, and persistence.
- Manual TUI approval flow smoke test.

## Risks
- LLM output can be malformed; parser must reject invalid or partial actions safely.
- Users need a clear distinction between proposed and committed changes.

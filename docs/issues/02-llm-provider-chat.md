# LLM Chat: GPT, Claude, Gemini, And Grok Provider Layer

## Goal
Add an LLM abstraction that supports GPT, Claude, Gemini, and Grok from one chat interface.

## User Value
Users can ask project planning questions from the terminal and choose their preferred model provider without changing workflows.

## Scope
- Define a provider-neutral chat request/response model.
- Add provider selection by CLI flag and environment variable.
- Implement HTTP clients for OpenAI-compatible, Anthropic, Gemini, and xAI/Grok APIs.
- Store chat history locally per project.
- Keep secrets out of persisted state and logs.

## Acceptance Criteria
- `todo-in-cli chat --provider openai "summarize this project"` sends a request when `OPENAI_API_KEY` is set.
- `--provider claude`, `--provider gemini`, and `--provider grok` have explicit API key validation and actionable errors.
- Failed provider calls return clear user-facing messages without panics.
- Chat messages are persisted per project.
- Provider code is isolated behind a trait so future providers can be added without touching the TUI.

## Non-Goals
- Streaming responses in the first version.
- Autonomous file edits.
- Provider-specific advanced tool-calling.

## Validation
- Unit tests for provider config and missing-key errors.
- Mockable provider trait.
- Manual smoke test with at least one real provider key.

## Risks
- Provider API shapes differ and change over time.
- Model names should be configurable instead of hard-coded into business logic.

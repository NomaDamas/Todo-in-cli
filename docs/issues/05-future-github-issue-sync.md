# Future Work: GitHub Issue Sync

## Goal
Connect local todos and roadmap items to GitHub Issues so terminal planning can sync with repository project management.

## User Value
Users can plan locally in the terminal while keeping GitHub Issues updated for team visibility.

## Future Scope
- Authenticate through GitHub CLI or a token.
- Link local todo IDs to GitHub issue numbers.
- Create issues from todos and roadmap items.
- Pull issue status back into local state.
- Support labels, milestones, assignees, and backlinks.
- Add conflict handling when local and remote state diverge.

## Acceptance Criteria
- Users can run a sync command in a git repository with a GitHub remote.
- Local items can be published to GitHub Issues.
- Existing issue links are preserved across syncs.
- Sync errors are actionable and do not corrupt local state.

## Non-Goals
- Full GitHub Projects support in the first sync version.
- Background daemon sync.

## Validation
- Unit tests for issue mapping and conflict states.
- Integration smoke test against a test repository.

## Risks
- GitHub rate limits and auth failures.
- Different teams use labels/milestones differently, so mapping must be configurable.

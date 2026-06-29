# GitHub Issue Sync

GitHub sync publishes unsynced local todos and roadmap items to GitHub Issues.

## Prerequisites

```sh
gh auth login
```

Run the command from a git repository that the authenticated GitHub account can access.

## Commands

```sh
todo-in-cli github sync --dry-run
todo-in-cli github sync --kind todos
todo-in-cli github sync --kind roadmap
todo-in-cli github sync --kind all
```

## Behavior

- Todos become issues titled `[todo] <title>`.
- Roadmap items become issues titled `[roadmap] <title>`.
- The local item stores the created issue number.
- Already linked items are skipped on later syncs.
- `--dry-run` prints intended publishes without creating issues.

## Failure Safety

The sync command uses the existing locked, atomic local persistence path. If GitHub creation fails, the local item remains unsynced and can be retried.

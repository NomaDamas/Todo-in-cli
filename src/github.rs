use std::process::Command;

use anyhow::{Context, Result, anyhow};

use crate::storage::Store;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SyncKind {
    All,
    Todos,
    Roadmap,
}

pub struct SyncReport {
    pub lines: Vec<String>,
}

pub fn sync_issues(kind: SyncKind, dry_run: bool) -> Result<SyncReport> {
    let mut lines = Vec::new();
    let mut store = if dry_run {
        Store::open_default()?
    } else {
        Store::open_default_locked()?
    };
    let project = store.ensure_current_project()?;

    if matches!(kind, SyncKind::All | SyncKind::Todos) {
        for todo in store.unsynced_todos(&project.id) {
            let title = format!("[todo] {}", todo.title);
            let body = format!(
                "Created from todo-in-cli.\n\nLocal todo id: `{}`\nStatus: `{}`",
                todo.id,
                if todo.completed { "done" } else { "open" }
            );
            if dry_run {
                lines.push(format!("dry-run todo {} -> {title}", todo.id));
            } else {
                let issue = create_issue(&title, &body)?;
                store.link_todo_issue(&project.id, &todo.id, issue)?;
                store.save()?;
                lines.push(format!("synced todo {} -> issue #{issue}", todo.id));
            }
        }
    }

    if matches!(kind, SyncKind::All | SyncKind::Roadmap) {
        for item in store.unsynced_roadmap(&project.id) {
            let title = format!("[roadmap] {}", item.title);
            let body = format!(
                "Created from todo-in-cli.\n\nLocal roadmap id: `{}`\nStatus: `{}`",
                item.id, item.status
            );
            if dry_run {
                lines.push(format!("dry-run roadmap {} -> {title}", item.id));
            } else {
                let issue = create_issue(&title, &body)?;
                store.link_roadmap_issue(&project.id, &item.id, issue)?;
                store.save()?;
                lines.push(format!("synced roadmap {} -> issue #{issue}", item.id));
            }
        }
    }

    if lines.is_empty() {
        lines.push("nothing to sync".to_string());
    }

    Ok(SyncReport { lines })
}

fn create_issue(title: &str, body: &str) -> Result<u64> {
    let mut command = Command::new("gh");
    command.args(["issue", "create", "--title", title, "--body", body]);

    let output = command
        .output()
        .context("failed to execute gh issue create")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("gh issue create failed: {}", stderr.trim()));
    }

    parse_issue_number(&String::from_utf8_lossy(&output.stdout))
}

pub fn parse_issue_number(output: &str) -> Result<u64> {
    let trimmed = output.trim();
    let last = trimmed
        .rsplit('/')
        .next()
        .ok_or_else(|| anyhow!("could not parse issue URL: {trimmed}"))?;
    last.parse::<u64>()
        .with_context(|| format!("could not parse issue number from {trimmed}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_issue_number_from_url() {
        let number = parse_issue_number("https://github.com/org/repo/issues/42\n").unwrap();
        assert_eq!(number, 42);
    }
}

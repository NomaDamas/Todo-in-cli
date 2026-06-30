use std::process::Command;

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;

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

pub fn sync_issues(kind: SyncKind, dry_run: bool, pull: bool) -> Result<SyncReport> {
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

    if pull {
        lines.extend(pull_issues(dry_run)?.lines);
    }

    Ok(SyncReport { lines })
}

pub fn pull_issues(dry_run: bool) -> Result<SyncReport> {
    let issues = list_issues()?;
    let mut lines = Vec::new();
    let mut store = if dry_run {
        Store::open_default()?
    } else {
        Store::open_default_locked()?
    };
    let project = store.ensure_current_project()?;

    for issue in issues {
        let Some((kind, title)) = parse_synced_title(&issue.title) else {
            continue;
        };
        let closed = issue.state.eq_ignore_ascii_case("closed");
        match kind {
            IssueKind::Todo => {
                if dry_run {
                    lines.push(format!(
                        "dry-run pull todo issue #{} [{}] {}",
                        issue.number, issue.state, title
                    ));
                } else if store.set_todo_completed_by_issue(&project.id, issue.number, closed)? {
                    lines.push(format!("updated todo from issue #{}", issue.number));
                    store.save()?;
                } else {
                    let todo =
                        store.add_todo_from_github(&project.id, title, issue.number, closed)?;
                    lines.push(format!(
                        "created todo {} from issue #{}",
                        todo.id, issue.number
                    ));
                    store.save()?;
                }
            }
            IssueKind::Roadmap => {
                let status = if closed { "done" } else { "planned" }.to_string();
                if dry_run {
                    lines.push(format!(
                        "dry-run pull roadmap issue #{} [{}] {}",
                        issue.number, issue.state, title
                    ));
                } else if store.set_roadmap_status_by_issue(&project.id, issue.number, status)? {
                    lines.push(format!("updated roadmap from issue #{}", issue.number));
                    store.save()?;
                } else {
                    let item =
                        store.add_roadmap_from_github(&project.id, title, issue.number, closed)?;
                    lines.push(format!(
                        "created roadmap {} from issue #{}",
                        item.id, issue.number
                    ));
                    store.save()?;
                }
            }
        }
    }

    if lines.is_empty() {
        lines.push("nothing to pull".to_string());
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

fn list_issues() -> Result<Vec<GhIssue>> {
    let output = Command::new("gh")
        .args([
            "issue",
            "list",
            "--state",
            "all",
            "--limit",
            "200",
            "--json",
            "number,title,state",
        ])
        .output()
        .context("failed to execute gh issue list")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("gh issue list failed: {}", stderr.trim()));
    }

    serde_json::from_slice(&output.stdout).context("failed to parse gh issue list JSON")
}

#[derive(Debug, Deserialize)]
struct GhIssue {
    number: u64,
    title: String,
    state: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum IssueKind {
    Todo,
    Roadmap,
}

fn parse_synced_title(title: &str) -> Option<(IssueKind, String)> {
    title
        .strip_prefix("[todo] ")
        .map(|title| (IssueKind::Todo, title.to_string()))
        .or_else(|| {
            title
                .strip_prefix("[roadmap] ")
                .map(|title| (IssueKind::Roadmap, title.to_string()))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_issue_number_from_url() {
        let number = parse_issue_number("https://github.com/org/repo/issues/42\n").unwrap();
        assert_eq!(number, 42);
    }

    #[test]
    fn parses_synced_issue_titles() {
        assert_eq!(
            parse_synced_title("[todo] Ship").unwrap(),
            (IssueKind::Todo, "Ship".to_string())
        );
        assert!(parse_synced_title("Other").is_none());
    }
}

use std::{env, process::Command};

use anyhow::{Context, Result, anyhow};

use crate::models::Project;

pub fn chat(project: &Project, message: &str) -> Result<String> {
    let prompt = format!(
        "Project: {}\nRoot: {}\n\nUser planning request:\n{}",
        project.name, project.root, message
    );

    if env::var("TODO_IN_CLI_CODEX_EXEC").as_deref() != Ok("1") {
        return Ok(format!(
            "Codex handoff prepared. Set TODO_IN_CLI_CODEX_EXEC=1 to execute local Codex CLI.\n\n```text\n{}\n```",
            prompt
        ));
    }

    let command = env::var("TODO_IN_CLI_CODEX_COMMAND").unwrap_or_else(|_| "codex".to_string());
    let output = Command::new(&command)
        .args(["exec", "--skip-git-repo-check", &prompt])
        .current_dir(&project.root)
        .output()
        .with_context(|| format!("failed to execute {command}"))?;

    if !output.status.success() {
        return Err(anyhow!(
            "{command} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

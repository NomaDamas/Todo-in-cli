use anyhow::{Context, Result, anyhow};

use crate::models::AgentTool;

pub fn parse_actions(raw: &str) -> Result<Vec<AgentTool>> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("agent action JSON cannot be empty"));
    }

    if trimmed.starts_with('[') {
        let actions: Vec<AgentTool> =
            serde_json::from_str(trimmed).context("failed to parse agent action array")?;
        ensure_actions(actions)
    } else {
        let action: AgentTool =
            serde_json::from_str(trimmed).context("failed to parse agent action")?;
        ensure_actions(vec![action])
    }
}

fn ensure_actions(actions: Vec<AgentTool>) -> Result<Vec<AgentTool>> {
    if actions.is_empty() {
        return Err(anyhow!("at least one agent action is required"));
    }

    for action in &actions {
        match action {
            AgentTool::CreateTodo { title } | AgentTool::CreateRoadmapItem { title } => {
                if title.trim().is_empty() {
                    return Err(anyhow!("agent action title cannot be empty"));
                }
            }
            AgentTool::CompleteTodo { id } => {
                if id.trim().is_empty() {
                    return Err(anyhow!("agent action todo id cannot be empty"));
                }
            }
            AgentTool::SummarizeProject => {}
        }
    }

    Ok(actions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_action() {
        let actions = parse_actions(r#"{"tool":"create_todo","title":"ship"}"#).unwrap();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].summary(), "create todo: ship");
    }

    #[test]
    fn rejects_empty_action_title() {
        let error = parse_actions(r#"{"tool":"create_todo","title":" "}"#).unwrap_err();
        assert!(error.to_string().contains("title cannot be empty"));
    }
}

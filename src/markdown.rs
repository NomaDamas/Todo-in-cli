use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

pub fn render_lines(input: &str) -> Vec<Line<'static>> {
    let lines: Vec<Line<'static>> = input
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            if let Some(text) = trimmed.strip_prefix("### ") {
                return styled_line(text, Color::Magenta, Modifier::BOLD);
            }
            if let Some(text) = trimmed.strip_prefix("## ") {
                return styled_line(text, Color::Cyan, Modifier::BOLD);
            }
            if let Some(text) = trimmed.strip_prefix("# ") {
                return styled_line(text, Color::Yellow, Modifier::BOLD);
            }
            if let Some(text) = trimmed.strip_prefix("- ") {
                let mut spans = vec![Span::styled("• ", Style::default().fg(Color::Yellow))];
                spans.extend(inline_segments(text));
                return Line::from(spans);
            }
            Line::from(inline_segments(trimmed))
        })
        .collect();

    if lines.is_empty() {
        vec![Line::from("")]
    } else {
        lines
    }
}

pub fn render_inline(input: &str) -> Line<'static> {
    Line::from(inline_segments(input))
}

fn styled_line(text: &str, color: Color, modifier: Modifier) -> Line<'static> {
    Line::from(Span::styled(
        text.to_string(),
        Style::default().fg(color).add_modifier(modifier),
    ))
}

fn inline_segments(input: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut rest = input;

    while !rest.is_empty() {
        let bold = rest.find("**");
        let code = rest.find('`');
        let next = match (bold, code) {
            (Some(bold), Some(code)) if bold < code => ("bold", bold),
            (Some(_), Some(code)) => ("code", code),
            (Some(bold), None) => ("bold", bold),
            (None, Some(code)) => ("code", code),
            (None, None) => {
                spans.push(Span::raw(rest.to_string()));
                break;
            }
        };

        if next.1 > 0 {
            spans.push(Span::raw(rest[..next.1].to_string()));
        }

        if next.0 == "bold" {
            let after = &rest[next.1 + 2..];
            if let Some(end) = after.find("**") {
                spans.push(Span::styled(
                    after[..end].to_string(),
                    Style::default().add_modifier(Modifier::BOLD),
                ));
                rest = &after[end + 2..];
            } else {
                spans.push(Span::raw(rest[next.1..].to_string()));
                break;
            }
        } else {
            let after = &rest[next.1 + 1..];
            if let Some(end) = after.find('`') {
                spans.push(Span::styled(
                    after[..end].to_string(),
                    Style::default().fg(Color::Green),
                ));
                rest = &after[end + 1..];
            } else {
                spans.push(Span::raw(rest[next.1..].to_string()));
                break;
            }
        }
    }

    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_heading_and_body() {
        let lines = render_lines("# Title\n- **todo** `cmd`");
        assert_eq!(lines.len(), 2);
        assert!(lines[1].spans.len() > 2);
    }
}

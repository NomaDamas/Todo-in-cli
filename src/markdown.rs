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
            if let Some((number, text)) = numbered_item(trimmed) {
                let mut spans = vec![Span::styled(
                    format!("{number}. "),
                    Style::default().fg(Color::Yellow),
                )];
                spans.extend(inline_segments(text));
                return Line::from(spans);
            }
            if let Some(text) = trimmed.strip_prefix("> ") {
                let mut spans = vec![Span::styled("│ ", Style::default().fg(Color::DarkGray))];
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
        let strike = rest.find("~~");
        let link = rest.find('[');
        let next = [
            bold.map(|index| ("bold", index)),
            code.map(|index| ("code", index)),
            strike.map(|index| ("strike", index)),
            link.map(|index| ("link", index)),
        ]
        .into_iter()
        .flatten()
        .min_by_key(|(_, index)| *index);

        let Some((kind, index)) = next else {
            spans.push(Span::raw(rest.to_string()));
            break;
        };

        if index > 0 {
            spans.push(Span::raw(rest[..index].to_string()));
        }

        if kind == "bold" {
            let after = &rest[index + 2..];
            if let Some(end) = after.find("**") {
                spans.push(Span::styled(
                    after[..end].to_string(),
                    Style::default().add_modifier(Modifier::BOLD),
                ));
                rest = &after[end + 2..];
            } else {
                spans.push(Span::raw(rest[index..].to_string()));
                break;
            }
        } else if kind == "code" {
            let after = &rest[index + 1..];
            if let Some(end) = after.find('`') {
                spans.push(Span::styled(
                    after[..end].to_string(),
                    Style::default().fg(Color::Green),
                ));
                rest = &after[end + 1..];
            } else {
                spans.push(Span::raw(rest[index..].to_string()));
                break;
            }
        } else if kind == "strike" {
            let after = &rest[index + 2..];
            if let Some(end) = after.find("~~") {
                spans.push(Span::styled(
                    after[..end].to_string(),
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::CROSSED_OUT),
                ));
                rest = &after[end + 2..];
            } else {
                spans.push(Span::raw(rest[index..].to_string()));
                break;
            }
        } else if let Some((label, url, consumed)) = parse_link(&rest[index..]) {
            spans.push(Span::styled(
                format!("{label} ({url})"),
                Style::default().fg(Color::Blue),
            ));
            rest = &rest[index + consumed..];
        } else {
            spans.push(Span::raw(rest[index..=index].to_string()));
            rest = &rest[index + 1..];
        }
    }

    spans
}

fn numbered_item(input: &str) -> Option<(&str, &str)> {
    let (number, rest) = input.split_once(". ")?;
    number
        .chars()
        .all(|ch| ch.is_ascii_digit())
        .then_some((number, rest))
}

fn parse_link(input: &str) -> Option<(&str, &str, usize)> {
    let label_end = input.find("](")?;
    if !input.starts_with('[') {
        return None;
    }
    let after_label = label_end + 2;
    let url_end = input[after_label..].find(')')? + after_label;
    let label = &input[1..label_end];
    let url = &input[after_label..url_end];
    if label.is_empty() || url.is_empty() {
        return None;
    }
    Some((label, url, url_end + 1))
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

    #[test]
    fn renders_expanded_markdown_subset() {
        let lines = render_lines("> [Docs](https://example.com)\n1. ~~old~~ **new**");
        assert_eq!(lines.len(), 2);
        assert!(lines[0].spans.len() > 1);
        assert!(lines[1].spans.len() > 2);
    }
}

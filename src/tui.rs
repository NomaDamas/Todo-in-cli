use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, MouseButton, MouseEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::{
    cli::ProviderKind,
    codex, llm, markdown,
    models::{AgentActionStatus, ChatMessage, Project, RoadmapItem, Todo},
    storage::Store,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Pane {
    Project,
    Todos,
    Roadmap,
    Chat,
}

pub fn run(project: Project, provider: ProviderKind) -> Result<()> {
    let mut stdout = std::io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, event::EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let result = run_loop(&mut terminal, project, provider);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    project: Project,
    provider: ProviderKind,
) -> Result<()> {
    let mut current_project = project;
    let mut codex_enabled = false;
    let mut chat_input = String::new();
    let mut status = "Chat pane: type and press Enter. x toggles Codex.".to_string();
    let mut active = Pane::Todos;
    let mut layout = DashboardLayout::default();

    loop {
        let view_store = Store::open_default()?;
        let projects = view_store.projects();
        if let Some(project) = projects
            .iter()
            .find(|project| project.id == current_project.id)
            .cloned()
            .or_else(|| projects.first().cloned())
        {
            current_project = project;
        }
        let todos = view_store.todos_for_project(&current_project.id);
        let roadmap = view_store.roadmap_for_project(&current_project.id);
        let chat = view_store.chat_for_project(&current_project.id);
        let pending_actions = view_store
            .agent_actions_for_project(&current_project.id)
            .iter()
            .filter(|action| matches!(action.status, AgentActionStatus::Pending))
            .count();

        terminal.draw(|frame| {
            layout = draw_dashboard(
                frame,
                DashboardView {
                    active,
                    projects: &projects,
                    project: &current_project,
                    todos: &todos,
                    roadmap: &roadmap,
                    chat_messages: &chat,
                    chat_input: &chat_input,
                    status: &status,
                    pending_actions,
                    codex_enabled,
                },
            );
        })?;

        if !event::poll(Duration::from_millis(250))? {
            continue;
        }

        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => break,
                KeyCode::Char('x') => codex_enabled = !codex_enabled,
                KeyCode::Enter if active == Pane::Chat && !chat_input.trim().is_empty() => {
                    status = "sending chat message...".to_string();
                    terminal.draw(|frame| {
                        layout = draw_dashboard(
                            frame,
                            DashboardView {
                                active,
                                projects: &projects,
                                project: &current_project,
                                todos: &todos,
                                roadmap: &roadmap,
                                chat_messages: &chat,
                                chat_input: &chat_input,
                                status: &status,
                                pending_actions,
                                codex_enabled,
                            },
                        );
                    })?;

                    match send_chat_message(provider, codex_enabled, &current_project, &chat_input)
                    {
                        Ok(message) => {
                            status = message;
                            chat_input.clear();
                        }
                        Err(error) => status = format!("chat failed: {error}"),
                    }
                }
                KeyCode::Backspace if active == Pane::Chat => {
                    chat_input.pop();
                }
                KeyCode::Char(ch) if active == Pane::Chat => {
                    chat_input.push(ch);
                }
                KeyCode::Tab => active = next_pane(active),
                KeyCode::BackTab => active = previous_pane(active),
                _ => {}
            },
            Event::Mouse(mouse) if mouse.kind == MouseEventKind::Down(MouseButton::Left) => {
                active = layout.pane_at(mouse.column, mouse.row).unwrap_or(active);
                if active == Pane::Project
                    && let Some(index) = layout.project_index_at(mouse.column, mouse.row)
                    && let Some(project) = projects.get(index)
                {
                    current_project = project.clone();
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn draw_dashboard(frame: &mut ratatui::Frame, view: DashboardView<'_>) -> DashboardLayout {
    let root = frame.area();
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(7)])
        .split(root);
    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(28),
            Constraint::Percentage(36),
            Constraint::Percentage(36),
        ])
        .split(vertical[0]);

    let layout = DashboardLayout {
        project: top[0],
        todos: top[1],
        roadmap: top[2],
        chat: vertical[1],
    };

    frame.render_widget(
        project_panel(view.projects, view.project, view.active == Pane::Project),
        layout.project,
    );
    frame.render_widget(
        todo_panel(view.todos, view.active == Pane::Todos),
        layout.todos,
    );
    frame.render_widget(
        roadmap_panel(view.roadmap, view.active == Pane::Roadmap),
        layout.roadmap,
    );
    frame.render_widget(
        chat_panel(
            view.chat_messages,
            view.chat_input,
            view.status,
            view.pending_actions,
            view.codex_enabled,
            view.active == Pane::Chat,
        ),
        layout.chat,
    );

    layout
}

struct DashboardView<'a> {
    active: Pane,
    projects: &'a [Project],
    project: &'a Project,
    todos: &'a [Todo],
    roadmap: &'a [RoadmapItem],
    chat_messages: &'a [ChatMessage],
    chat_input: &'a str,
    status: &'a str,
    pending_actions: usize,
    codex_enabled: bool,
}

fn project_panel(projects: &[Project], selected: &Project, active: bool) -> List<'static> {
    let mut items = Vec::new();

    if projects.is_empty() {
        items.push(ListItem::new("No projects yet."));
    } else {
        for project in projects {
            let marker = if project.id == selected.id { ">" } else { " " };
            let style = if project.id == selected.id {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            items.push(ListItem::new(Line::from(vec![Span::styled(
                truncate_project_label(marker, &project.name),
                style,
            )])));
        }
    }

    items.extend([
        ListItem::new(""),
        ListItem::new(format!("root: {}", selected.root)),
        ListItem::new(""),
        ListItem::new("x: Codex toggle"),
        ListItem::new("Tab: next pane, q: quit"),
    ]);

    List::new(items).block(panel_block("Projects - click to switch", active))
}

fn truncate_project_label(marker: &str, name: &str) -> String {
    const MAX_LABEL_CHARS: usize = 28;
    let label = format!("{marker} {name}");
    if label.chars().count() <= MAX_LABEL_CHARS {
        return label;
    }

    let mut truncated: String = label.chars().take(MAX_LABEL_CHARS - 1).collect();
    truncated.push('…');
    truncated
}

fn todo_panel(todos: &[Todo], active: bool) -> List<'static> {
    let items = if todos.is_empty() {
        vec![ListItem::new("No todos yet. Use `todo-in-cli todo add`.")]
    } else {
        todos
            .iter()
            .map(|todo| {
                let marker = if todo.completed { "[x]" } else { "[ ]" };
                ListItem::new(markdown::render_inline(&format!(
                    "{marker} {} {}",
                    todo.id, todo.title
                )))
            })
            .collect()
    };
    List::new(items).block(panel_block("Todos", active))
}

fn roadmap_panel(roadmap: &[RoadmapItem], active: bool) -> List<'static> {
    let items = if roadmap.is_empty() {
        vec![ListItem::new(
            "No roadmap items yet. Use `todo-in-cli roadmap add`.",
        )]
    } else {
        roadmap
            .iter()
            .map(|item| {
                ListItem::new(markdown::render_inline(&format!(
                    "{} [{}] {}",
                    item.id, item.status, item.title
                )))
            })
            .collect()
    };
    List::new(items).block(panel_block("Roadmap", active))
}

fn chat_panel(
    messages: &[ChatMessage],
    input: &str,
    status: &str,
    pending_actions: usize,
    codex_enabled: bool,
    active: bool,
) -> Paragraph<'static> {
    let codex = if codex_enabled { "**on**" } else { "`off`" };
    let mut lines = markdown::render_lines(&format!(
        "Codex mode: {codex} `(x toggles)`  Pending approvals: **{pending_actions}**"
    ));
    lines.push(Line::from(""));
    for message in messages.iter().rev().take(4).rev() {
        lines.extend(markdown::render_lines(&format!(
            "**{}**: {}",
            message.role, message.content
        )));
    }
    lines.push(Line::from(""));
    lines.extend(markdown::render_lines(status));
    lines.push(markdown::render_inline(&format!("> {input}")));

    Paragraph::new(lines)
        .block(panel_block("Agent Chat", active))
        .wrap(Wrap { trim: true })
}

fn send_chat_message(
    provider: ProviderKind,
    codex_enabled: bool,
    project: &Project,
    input: &str,
) -> Result<String> {
    let user_message = input.trim().to_string();
    let response = if codex_enabled {
        codex::chat(project, &user_message)?
    } else {
        let project_name = project.name.clone();
        let request_message = user_message.clone();
        std::thread::spawn(move || -> Result<String> {
            let runtime = tokio::runtime::Runtime::new()?;
            runtime.block_on(async move {
                let client = llm::provider_from_env(provider)?;
                let response = client
                    .chat(llm::ChatRequest {
                        project_name,
                        message: request_message,
                    })
                    .await?;
                Ok(response.message)
            })
        })
        .join()
        .map_err(|_| anyhow::anyhow!("chat worker panicked"))??
    };

    let mut store = Store::open_default_locked()?;
    store.add_chat_message(&project.id, "user", user_message)?;
    store.add_chat_message(&project.id, "assistant", response)?;
    store.save()?;
    Ok("chat response saved".to_string())
}

fn panel_block(title: &'static str, active: bool) -> Block<'static> {
    let style = if active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(style)
}

fn next_pane(pane: Pane) -> Pane {
    match pane {
        Pane::Project => Pane::Todos,
        Pane::Todos => Pane::Roadmap,
        Pane::Roadmap => Pane::Chat,
        Pane::Chat => Pane::Project,
    }
}

fn previous_pane(pane: Pane) -> Pane {
    match pane {
        Pane::Project => Pane::Chat,
        Pane::Todos => Pane::Project,
        Pane::Roadmap => Pane::Todos,
        Pane::Chat => Pane::Roadmap,
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct DashboardLayout {
    project: Rect,
    todos: Rect,
    roadmap: Rect,
    chat: Rect,
}

impl DashboardLayout {
    fn pane_at(&self, column: u16, row: u16) -> Option<Pane> {
        [
            (Pane::Project, self.project),
            (Pane::Todos, self.todos),
            (Pane::Roadmap, self.roadmap),
            (Pane::Chat, self.chat),
        ]
        .into_iter()
        .find_map(|(pane, rect)| contains(rect, column, row).then_some(pane))
    }

    fn project_index_at(&self, column: u16, row: u16) -> Option<usize> {
        if !contains(self.project, column, row) {
            return None;
        }

        let content_top = self.project.y.saturating_add(1);
        if row < content_top {
            return None;
        }

        Some(usize::from(row.saturating_sub(content_top)))
    }
}

fn contains(rect: Rect, column: u16, row: u16) -> bool {
    column >= rect.x
        && column < rect.x.saturating_add(rect.width)
        && row >= rect.y
        && row < rect.y.saturating_add(rect.height)
}

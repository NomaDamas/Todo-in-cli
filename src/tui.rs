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
    models::{AgentActionStatus, Project, RoadmapItem, Todo},
    storage::Store,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Pane {
    Project,
    Todos,
    Roadmap,
    Chat,
}

pub fn run(store: &Store, project: Project) -> Result<()> {
    let mut stdout = std::io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, event::EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let result = run_loop(&mut terminal, store, project);

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
    store: &Store,
    project: Project,
) -> Result<()> {
    let mut active = Pane::Todos;
    let mut layout = DashboardLayout::default();

    loop {
        let todos = store.todos_for_project(&project.id);
        let roadmap = store.roadmap_for_project(&project.id);
        let chat = store.chat_for_project(&project.id);
        let pending_actions = store
            .agent_actions_for_project(&project.id)
            .iter()
            .filter(|action| matches!(action.status, AgentActionStatus::Pending))
            .count();

        terminal.draw(|frame| {
            layout = draw_dashboard(
                frame,
                active,
                &project,
                &todos,
                &roadmap,
                chat.len(),
                pending_actions,
            );
        })?;

        if !event::poll(Duration::from_millis(250))? {
            continue;
        }

        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => break,
                KeyCode::Tab => active = next_pane(active),
                KeyCode::BackTab => active = previous_pane(active),
                _ => {}
            },
            Event::Mouse(mouse) if mouse.kind == MouseEventKind::Down(MouseButton::Left) => {
                active = layout.pane_at(mouse.column, mouse.row).unwrap_or(active);
            }
            _ => {}
        }
    }

    Ok(())
}

fn draw_dashboard(
    frame: &mut ratatui::Frame,
    active: Pane,
    project: &Project,
    todos: &[Todo],
    roadmap: &[RoadmapItem],
    chat_count: usize,
    pending_actions: usize,
) -> DashboardLayout {
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
        project_panel(project, active == Pane::Project),
        layout.project,
    );
    frame.render_widget(todo_panel(todos, active == Pane::Todos), layout.todos);
    frame.render_widget(
        roadmap_panel(roadmap, active == Pane::Roadmap),
        layout.roadmap,
    );
    frame.render_widget(
        chat_panel(chat_count, pending_actions, active == Pane::Chat),
        layout.chat,
    );

    layout
}

fn project_panel(project: &Project, active: bool) -> Paragraph<'static> {
    let lines = vec![
        Line::from(vec![Span::styled(
            project.name.clone(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(format!("root: {}", project.root)),
        Line::from(""),
        Line::from("Tab: next pane"),
        Line::from("Mouse: focus pane"),
        Line::from("q/Esc: quit"),
    ];
    Paragraph::new(lines)
        .block(panel_block("Project", active))
        .wrap(Wrap { trim: true })
}

fn todo_panel(todos: &[Todo], active: bool) -> List<'static> {
    let items = if todos.is_empty() {
        vec![ListItem::new("No todos yet. Use `todo-in-cli todo add`.")]
    } else {
        todos
            .iter()
            .map(|todo| {
                let marker = if todo.completed { "[x]" } else { "[ ]" };
                ListItem::new(format!("{marker} {} {}", todo.id, todo.title))
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
            .map(|item| ListItem::new(format!("{} [{}] {}", item.id, item.status, item.title)))
            .collect()
    };
    List::new(items).block(panel_block("Roadmap", active))
}

fn chat_panel(chat_count: usize, pending_actions: usize, active: bool) -> Paragraph<'static> {
    Paragraph::new(vec![
        Line::from("Agent chat is available from CLI in this MVP:"),
        Line::from("todo-in-cli chat --provider openai \"plan next steps\""),
        Line::from(format!("Persisted project chat messages: {chat_count}")),
        Line::from(format!("Pending approval actions: {pending_actions}")),
    ])
    .block(panel_block("Agent Chat", active))
    .wrap(Wrap { trim: true })
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
}

fn contains(rect: Rect, column: u16, row: u16) -> bool {
    column >= rect.x
        && column < rect.x.saturating_add(rect.width)
        && row >= rect.y
        && row < rect.y.saturating_add(rect.height)
}

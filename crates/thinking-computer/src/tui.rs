use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use std::{io, time::Duration};
use tc_core::{system_summary, SkillStore};

#[derive(Clone, Copy, PartialEq, Eq)]
enum View {
    Status,
    Skills,
    Task,
}

impl View {
    fn label(self) -> &'static str {
        match self {
            Self::Status => "STATUS",
            Self::Skills => "SKILLS",
            Self::Task => "NEW TASK",
        }
    }
    fn next(self) -> Self {
        match self {
            Self::Status => Self::Skills,
            Self::Skills => Self::Task,
            Self::Task => Self::Status,
        }
    }
    fn previous(self) -> Self {
        self.next().next()
    }
}

fn status_lines() -> Vec<Line<'static>> {
    let skills = SkillStore::local().list().unwrap_or_default();
    let enabled = skills.iter().filter(|skill| skill.enabled).count();
    vec![
        Line::from(vec![Span::styled(
            "THINKING COMPUTER",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("mono-pixel local agent console"),
        Line::from(""),
        Line::from(format!("SYSTEM  {}", system_summary())),
        Line::from(format!("SKILLS  {enabled}/{} enabled", skills.len())),
        Line::from("MEMORY  local-only; secrets are excluded"),
        Line::from("POLICY  approvals remain in the Rust engine"),
        Line::from(""),
        Line::from("Use [3] NEW TASK to compose an approved agent request."),
    ]
}

fn skill_lines() -> Vec<Line<'static>> {
    let skills = SkillStore::local().list().unwrap_or_default();
    if skills.is_empty() {
        return vec![Line::from(
            "No local Skills. Create one with `thinking-computer skills create`.",
        )];
    }
    skills
        .into_iter()
        .map(|skill| {
            let state = if skill.enabled { "ENABLED" } else { "DISABLED" };
            Line::from(format!(
                "{state:8}  {}  v{} — {}",
                skill.name, skill.version, skill.description
            ))
        })
        .collect()
}

fn task_lines(task: &str) -> Vec<Line<'static>> {
    let prompt = if task.is_empty() {
        "task> ".to_string()
    } else {
        format!("task> {task}")
    };
    vec![
        Line::from("Compose a task. Press Enter to return to the CLI and run it through the normal Rust permission flow."),
        Line::from(""),
        Line::from(vec![Span::styled(prompt, Style::default().fg(Color::White))]),
    ]
}

pub fn run() -> Result<Option<String>> {
    enable_raw_mode()?;
    let mut output = io::stdout();
    execute!(output, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(output);
    let mut terminal = Terminal::new(backend)?;
    let mut view = View::Status;
    let mut task = String::new();
    let result = loop {
        terminal.draw(|frame| {
            let areas = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(10),
                    Constraint::Length(3),
                ])
                .split(frame.area());
            let navigation = Paragraph::new(format!(
                " [1] STATUS  [2] SKILLS  [3] NEW TASK    ACTIVE: {} ",
                view.label()
            ))
            .style(Style::default().fg(Color::LightMagenta))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" [ THINKING COMPUTER ] "),
            );
            frame.render_widget(navigation, areas[0]);
            let content = match view {
                View::Status => status_lines(),
                View::Skills => skill_lines(),
                View::Task => task_lines(&task),
            };
            let panel = Paragraph::new(content)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::DarkGray))
                        .title(format!(" [ {} ] ", view.label())),
                )
                .alignment(Alignment::Left)
                .style(Style::default().bg(Color::Black).fg(Color::White))
                .wrap(Wrap { trim: true });
            frame.render_widget(panel, areas[1]);
            let help = match view {
                View::Task => "[Enter] run task   [Backspace] edit   [←/→] navigate   [q/Esc] quit",
                _ => "[1/2/3] navigate   [←/→] navigate   [r] refresh   [q/Esc] quit",
            };
            frame.render_widget(
                Paragraph::new(help)
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::Gray)),
                areas[2],
            );
        })?;
        if event::poll(Duration::from_millis(120))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break Ok(None),
                    KeyCode::Left => view = view.previous(),
                    KeyCode::Right => view = view.next(),
                    KeyCode::Char('1') => view = View::Status,
                    KeyCode::Char('2') => view = View::Skills,
                    KeyCode::Char('3') => view = View::Task,
                    KeyCode::Char(character) if view == View::Task => task.push(character),
                    KeyCode::Backspace if view == View::Task => {
                        task.pop();
                    }
                    KeyCode::Enter if view == View::Task && !task.trim().is_empty() => {
                        break Ok(Some(task.trim().to_string()));
                    }
                    _ => {}
                }
            }
        }
    };
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_the_local_agent_identity() {
        assert!(status_lines()
            .iter()
            .any(|line| line.to_string().contains("THINKING COMPUTER")));
    }

    #[test]
    fn exposes_a_bounded_task_handoff() {
        assert!(task_lines("review workspace")
            .iter()
            .any(|line| line.to_string().contains("task> review workspace")));
    }
}

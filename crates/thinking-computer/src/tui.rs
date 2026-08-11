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

fn status_lines() -> Vec<Line<'static>> {
    let skill_count = SkillStore::local()
        .list()
        .map(|skills| skills.len())
        .unwrap_or(0);
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
        Line::from(format!("SKILLS  {skill_count} local manifest(s)")),
        Line::from("MEMORY  local-only; secrets are excluded"),
        Line::from("POLICY  approvals remain in the Rust engine"),
    ]
}

pub fn run() -> Result<()> {
    enable_raw_mode()?;
    let mut output = io::stdout();
    execute!(output, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(output);
    let mut terminal = Terminal::new(backend)?;
    let result = loop {
        terminal.draw(|frame| {
            let areas = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([Constraint::Min(10), Constraint::Length(3)])
                .split(frame.area());
            let panel = Paragraph::new(status_lines())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::DarkGray))
                        .title(" [ AGENT STATUS ] "),
                )
                .alignment(Alignment::Left)
                .style(Style::default().bg(Color::Black).fg(Color::White))
                .wrap(Wrap { trim: true });
            frame.render_widget(panel, areas[0]);
            let footer = Paragraph::new(
                "[q] quit   [Esc] quit   Use `thinking-computer chat` for agent tasks",
            )
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
            frame.render_widget(footer, areas[1]);
        })?;
        if event::poll(Duration::from_millis(120))? {
            if let Event::Key(key) = event::read()? {
                if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                    break Ok(());
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
}

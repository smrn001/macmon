use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph};
use ratatui::Frame;

use super::format::bytes;
use crate::models::process::ProcessDetails;

pub fn render(frame: &mut Frame, area: Rect, details: &ProcessDetails) {
    let executable = if details.executable.is_empty() {
        "unknown".to_string()
    } else {
        details.executable.clone()
    };
    let rows = vec![
        row("Name", details.name.clone()),
        row("PID", details.pid.to_string()),
        row("Parent", details.parent.to_string()),
        row("User", details.user.clone()),
        row("CPU", format!("{:.1}%", details.cpu)),
        row("Memory", bytes(details.memory)),
        row("Threads", details.threads.to_string()),
        row("State", state_str(details.state).to_string()),
        Line::from(""),
        Line::from(Span::styled(" Executable", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(Span::raw(format!(" {executable}"))),
    ];
    let paragraph = Paragraph::new(rows).block(Block::bordered().title(" PROCESS "));
    frame.render_widget(paragraph, area);
}

pub fn render_confirm(frame: &mut Frame, area: Rect, details: &ProcessDetails) {
    let width = area.width.min(48);
    let height = 5;
    let popup = Rect::new(
        area.x + (area.width.saturating_sub(width)) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    );
    let lines = vec![
        Line::from(Span::raw(format!(
            "Kill process {} ({})?",
            details.pid, details.name
        ))),
        Line::from(""),
        Line::from(Span::styled(
            " [y] Yes   [n] No ",
            Style::default().add_modifier(Modifier::BOLD),
        )),
    ];
    frame.render_widget(Clear, popup);
    let confirm = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(Block::bordered().title(" CONFIRM "));
    frame.render_widget(confirm, popup);
}

fn row(label: &str, value: String) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!(" {label:<9}"),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(value),
    ])
}

fn state_str(state: char) -> &'static str {
    match state {
        'I' => "Idle",
        'R' => "Running",
        'S' => "Sleeping",
        'T' => "Stopped",
        'Z' => "Zombie",
        _ => "Unknown",
    }
}

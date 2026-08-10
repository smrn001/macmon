use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Cell, Row, Table, TableState};
use ratatui::Frame;

use super::format::bytes;
use crate::models::process::ProcessInfo;

pub fn render(frame: &mut Frame, area: Rect, processes: &[ProcessInfo], selected: usize) {
    let header = Row::new(vec![
        Cell::from("PID"),
        Cell::from("CPU"),
        Cell::from("MEM"),
        Cell::from("ST"),
        Cell::from("THRD"),
        Cell::from("NAME"),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD));

    let rows = processes.iter().map(|p| {
        Row::new(vec![
            Cell::from(p.pid.to_string()),
            Cell::from(format!("{:>5.1}%", p.cpu)),
            Cell::from(format!("{:>7}", bytes(p.memory))),
            Cell::from(p.state.to_string()),
            Cell::from(p.threads.to_string()),
            Cell::from(p.name.as_str()),
        ])
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Length(4),
            Constraint::Length(6),
            Constraint::Min(10),
        ],
    )
    .header(header)
    .block(Block::bordered().title(" PROCESSES "))
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let mut state = TableState::new().with_selected(Some(selected));
    frame.render_stateful_widget(table, area, &mut state);
}

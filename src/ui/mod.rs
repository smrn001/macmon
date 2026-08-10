use ratatui::layout::{Alignment, Layout as RatatuiLayout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;

use crate::app::App;

use self::header::Header;
use self::layout::Layout;

pub mod cpu;
pub mod format;
pub mod header;
pub mod layout;
pub mod lines;
pub mod memory;

pub fn render(frame: &mut Frame, app: &App) {
    let layout = Layout::new(frame.area());
    let cpu = app.cpu_usage.as_ref().map(|u| u.total);
    let mem = app.memory.as_ref().map(|m| m.used_ratio() * 100.0);
    frame.render_widget(Header::new(cpu, mem), layout.header);
    render_body(frame, layout.body, app);
    render_footer(frame, layout.footer);
}

fn render_body(frame: &mut Frame, area: Rect, app: &App) {
    let [cpu, memory] = RatatuiLayout::horizontal([
        ratatui::layout::Constraint::Percentage(55),
        ratatui::layout::Constraint::Percentage(45),
    ])
    .areas(area);
    match &app.cpu_usage {
        Some(usage) => frame.render_widget(cpu::CpuPanel::new(usage), cpu),
        None => render_placeholder(frame, cpu, " CPU "),
    }
    match &app.memory {
        Some(info) => frame.render_widget(memory::MemoryPanel::new(info), memory),
        None => render_placeholder(frame, memory, " MEMORY "),
    }
}

fn render_placeholder(frame: &mut Frame, area: Rect, title: &str) {
    let text = "collecting…";
    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(Block::bordered().title(title));
    frame.render_widget(paragraph, area);
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let hint = Span::styled(" q Quit ", Style::default().add_modifier(Modifier::DIM));
    frame.render_widget(hint, area);
}

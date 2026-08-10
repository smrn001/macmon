use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Widget};

use super::format::{bytes, percent};
use super::lines;
use crate::models::memory::MemoryInfo;

pub struct MemoryPanel<'a> {
    memory: &'a MemoryInfo,
}

impl<'a> MemoryPanel<'a> {
    pub fn new(memory: &'a MemoryInfo) -> Self {
        Self { memory }
    }
}

impl Widget for MemoryPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let width = area.width.saturating_sub(2);
        let memory = self.memory;

        let used_pct = memory.used_ratio() * 100.0;
        let lines = vec![
            lines::row("Physical".to_string(), bytes(memory.physical), width),
            lines::row(
                "Used".to_string(),
                format!("{}  ({})", bytes(memory.used()), percent(used_pct)),
                width,
            ),
            lines::row("Cached".to_string(), bytes(memory.cached()), width),
            lines::row("Free".to_string(), bytes(memory.free), width),
            lines::row("Compressed".to_string(), bytes(memory.compressed), width),
            lines::row(
                "Swap".to_string(),
                format!("{} / {}", bytes(memory.swap_used), bytes(memory.swap_total)),
                width,
            ),
            Line::raw(""),
            lines::bar_row(
                "Usage".to_string(),
                memory.used_ratio(),
                format!("{:>4}", percent(used_pct)),
                width,
            ),
        ];

        Paragraph::new(lines)
            .block(Block::bordered().title(" MEMORY "))
            .render(area, buf);
    }
}

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Widget};

use super::format::percent;
use super::lines;
use crate::models::cpu::CpuUsage;

pub struct CpuPanel<'a> {
    usage: &'a CpuUsage,
}

impl<'a> CpuPanel<'a> {
    pub fn new(usage: &'a CpuUsage) -> Self {
        Self { usage }
    }
}

impl Widget for CpuPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let width = area.width.saturating_sub(2);

        let mut lines = vec![
            lines::row("Total".to_string(), percent(self.usage.total), width),
            Line::from(Span::styled(
                format!(
                    "User {}  System {}  Idle {}",
                    percent(self.usage.user),
                    percent(self.usage.system),
                    percent(self.usage.idle)
                ),
                Style::default().add_modifier(Modifier::DIM),
            )),
            Line::raw(""),
        ];

        if self.usage.cores.is_empty() {
            lines.push(Line::raw("no core data"));
        } else {
            for (i, pct) in self.usage.cores.iter().enumerate() {
                let label = format!("Core {:>2}", i + 1);
                let pct_text = format!("{:>4}", format!("{pct:.0}%"));
                lines.push(lines::bar_row(label, pct / 100.0, pct_text, width));
            }
        }

        Paragraph::new(lines)
            .block(Block::bordered().title(" CPU "))
            .render(area, buf);
    }
}

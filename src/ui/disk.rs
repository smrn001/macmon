use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Widget};

use super::format::{bytes, percent};
use super::lines;
use crate::models::disk::DiskInfo;

pub struct DiskPanel<'a> {
    disk: &'a DiskInfo,
}

impl<'a> DiskPanel<'a> {
    pub fn new(disk: &'a DiskInfo) -> Self {
        Self { disk }
    }
}

impl Widget for DiskPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let width = area.width.saturating_sub(2);
        let disk = self.disk;

        let used_pct = disk.used_ratio() * 100.0;
        let mut lines = vec![
            lines::row("Capacity".to_string(), bytes(disk.capacity), width),
            lines::row(
                "Used".to_string(),
                format!("{}  ({})", bytes(disk.used), percent(used_pct)),
                width,
            ),
            lines::row("Available".to_string(), bytes(disk.available), width),
            lines::row(
                "Read".to_string(),
                format!("{}/s", bytes(disk.read_rate as u64)),
                width,
            ),
            lines::row(
                "Write".to_string(),
                format!("{}/s", bytes(disk.write_rate as u64)),
                width,
            ),
        ];
        if area.height > 8 {
            lines.push(Line::raw(""));
            lines.push(lines::bar_row(
                "Usage".to_string(),
                disk.used_ratio(),
                format!("{:>4}", percent(used_pct)),
                width,
            ));
        }

        Paragraph::new(lines)
            .block(Block::bordered().title(" DISK "))
            .render(area, buf);
    }
}

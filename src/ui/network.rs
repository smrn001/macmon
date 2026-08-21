use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Paragraph, Widget};

use super::format::bytes;
use super::lines;
use crate::models::network::NetworkUsage;

pub struct NetworkPanel<'a> {
    usage: &'a NetworkUsage,
}

impl<'a> NetworkPanel<'a> {
    pub fn new(usage: &'a NetworkUsage) -> Self {
        Self { usage }
    }
}

impl Widget for NetworkPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let width = area.width.saturating_sub(2);
        let usage = self.usage;

        let lines = vec![
            lines::row(
                "Download".to_string(),
                format!("↓ {}/s", bytes(usage.download_rate as u64)),
                width,
            ),
            lines::row(
                "Upload".to_string(),
                format!("↑ {}/s", bytes(usage.upload_rate as u64)),
                width,
            ),
            lines::row("Total ↓".to_string(), bytes(usage.total_downloaded), width),
            lines::row("Total ↑".to_string(), bytes(usage.total_uploaded), width),
        ];

        Paragraph::new(lines)
            .block(Block::bordered().title(" NETWORK "))
            .render(area, buf);
    }
}

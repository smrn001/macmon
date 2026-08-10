use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

pub struct Header;

impl Widget for Header {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let left = " macmon ";
        let right = "CPU --%  RAM --% ";
        let padding = area
            .width
            .saturating_sub((left.len() + right.len()) as u16);
        let line = Line::from(vec![
            Span::styled(left, Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" ".repeat(padding as usize)),
            Span::styled(right, Style::default().add_modifier(Modifier::DIM)),
        ]);
        line.render(area, buf);
    }
}

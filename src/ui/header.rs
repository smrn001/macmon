use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

pub struct Header {
    cpu: Option<f64>,
    mem: Option<f64>,
}

impl Header {
    pub fn new(cpu: Option<f64>, mem: Option<f64>) -> Self {
        Self { cpu, mem }
    }
}

impl Widget for Header {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let left = " macmon ";
        let cpu = self
            .cpu
            .map(|v| format!("{v:.0}%"))
            .unwrap_or_else(|| "--".into());
        let mem = self
            .mem
            .map(|v| format!("{v:.0}%"))
            .unwrap_or_else(|| "--".into());
        let right = format!("CPU {cpu}  RAM {mem} ");
        let padding = area.width.saturating_sub((left.len() + right.len()) as u16);
        let line = Line::from(vec![
            Span::styled(left, Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" ".repeat(padding as usize)),
            Span::styled(right, Style::default().add_modifier(Modifier::DIM)),
        ]);
        line.render(area, buf);
    }
}

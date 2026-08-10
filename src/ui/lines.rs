use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

pub fn row(left: String, right: String, width: u16) -> Line<'static> {
    let pad = width.saturating_sub((left.len() + right.len()) as u16);
    Line::from(vec![
        Span::raw(left),
        Span::raw(" ".repeat(pad as usize)),
        Span::raw(right),
    ])
}

pub fn bar_row(label: String, ratio: f64, pct_text: String, width: u16) -> Line<'static> {
    let bar_width = width.saturating_sub((label.len() + pct_text.len() + 2) as u16);
    let filled = (ratio * bar_width as f64).round() as u16;
    let mut spans = vec![Span::raw(label), Span::raw(" ")];
    if bar_width > 0 {
        let mut bar = "█".repeat(filled as usize);
        bar.push_str(&"░".repeat(bar_width.saturating_sub(filled) as usize));
        spans.push(Span::styled(bar, Style::default().add_modifier(Modifier::DIM)));
        spans.push(Span::raw(" "));
    }
    spans.push(Span::raw(pct_text));
    Line::from(spans)
}

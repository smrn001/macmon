use ratatui::layout::{Constraint, Layout as RatatuiLayout, Rect};

pub struct Layout {
    pub header: Rect,
    pub body: Rect,
    pub footer: Rect,
}

impl Layout {
    pub fn new(area: Rect) -> Self {
        let [header, body, footer] = RatatuiLayout::vertical([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .areas(area);
        Self {
            header,
            body,
            footer,
        }
    }
}

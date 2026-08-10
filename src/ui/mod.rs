use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Paragraph};
use ratatui::{Alignment, Frame, VerticalAlignment};

use self::header::Header;
use self::layout::Layout;

pub mod header;
pub mod layout;

pub fn render(frame: &mut Frame) {
    let layout = Layout::new(frame.area());
    frame.render_widget(Header, layout.header);
    render_body(frame, layout.body);
    render_footer(frame, layout.footer);
}

fn render_body(frame: &mut Frame, area: Rect) {
    let text = "Monitoring panels arrive in Milestone 2\n\nCPU · Memory · Processes · Network · Disk";
    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .vertical_alignment(VerticalAlignment::Middle)
        .block(Block::bordered().title(" MACMON "));
    frame.render_widget(paragraph, area);
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let hint = Span::styled(" q Quit ", Style::default().add_modifier(Modifier::DIM));
    frame.render_widget(hint, area);
}

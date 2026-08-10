mod app;
mod event;
mod models;
mod system;
mod ui;

use std::io;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = app::App::new().run(&mut terminal);
    ratatui::restore();
    result
}

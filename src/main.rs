mod app;
mod event;
mod models;
mod system;
mod ui;

use std::io;

fn main() -> io::Result<()> {
    if let Some(flag) = std::env::args().nth(1) {
        match flag.as_str() {
            "--version" | "-V" => {
                println!("macmon {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "--help" | "-h" => {
                println!(
                    "macmon {} — ultra-lightweight system monitor for macOS\n\n\
                     Usage: macmon\n\n\
                     Runs an interactive terminal UI. Press q inside the app to quit.",
                    env!("CARGO_PKG_VERSION")
                );
                return Ok(());
            }
            other => {
                eprintln!("unknown argument: {other}\ntry 'macmon --help'");
                std::process::exit(2);
            }
        }
    }

    let mut terminal = ratatui::init();
    let result = app::App::new().run(&mut terminal);
    ratatui::restore();
    result
}

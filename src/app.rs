use std::io;

use crossterm::event::KeyCode;
use ratatui::{DefaultTerminal, Frame};

use crate::event::{self, Event};

pub struct App {
    should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self { should_quit: false }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_events(terminal)?;
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        crate::ui::render(frame);
    }

    fn handle_events(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        match event::read()? {
            Event::Key(key) => self.on_key(key),
            Event::Resize => terminal.clear()?,
            Event::Tick => {}
        }
        Ok(())
    }

    fn on_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            _ => {}
        }
    }
}

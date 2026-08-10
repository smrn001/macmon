use std::io;
use std::time::Duration;

use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, KeyEventKind};

const TICK_RATE: Duration = Duration::from_millis(250);

pub enum Event {
    Key(KeyEvent),
    Resize(u16, u16),
    Tick,
}

pub fn read() -> io::Result<Event> {
    if !event::poll(TICK_RATE)? {
        return Ok(Event::Tick);
    }
    match event::read()? {
        CrosstermEvent::Key(key) if key.kind == KeyEventKind::Press => Ok(Event::Key(key)),
        CrosstermEvent::Resize(width, height) => Ok(Event::Resize(width, height)),
        _ => Ok(Event::Tick),
    }
}

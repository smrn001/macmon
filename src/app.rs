use std::io;
use std::time::{Duration, Instant};

use crossterm::event::KeyCode;
use ratatui::{DefaultTerminal, Frame};

use crate::event::{self, Event};
use crate::models::cpu::CpuUsage;
use crate::models::memory::MemoryInfo;
use crate::system::cpu::CpuSampler;

const CPU_REFRESH: Duration = Duration::from_millis(500);
const MEMORY_REFRESH: Duration = Duration::from_millis(1000);

pub struct App {
    should_quit: bool,
    cpu: CpuSampler,
    pub(crate) cpu_usage: Option<CpuUsage>,
    pub(crate) memory: Option<MemoryInfo>,
    last_cpu: Instant,
    last_memory: Instant,
}

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            cpu: CpuSampler::new(),
            cpu_usage: None,
            memory: None,
            last_cpu: Instant::now(),
            last_memory: Instant::now(),
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.refresh();
        while !self.should_quit {
            terminal.draw(|frame| self.render(frame))?;
            match event::read()? {
                Event::Key(key) => self.on_key(key),
                Event::Resize => terminal.clear()?,
                Event::Tick => self.refresh(),
            }
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        crate::ui::render(frame, self);
    }

    fn refresh(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_cpu) >= CPU_REFRESH {
            if let Some(usage) = self.cpu.sample() {
                self.cpu_usage = Some(usage);
            }
            self.last_cpu = now;
        }
        if now.duration_since(self.last_memory) >= MEMORY_REFRESH {
            self.memory = crate::system::memory::sample();
            self.last_memory = now;
        }
    }

    fn on_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            _ => {}
        }
    }
}

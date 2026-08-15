use std::io;
use std::time::{Duration, Instant};

use crossterm::event::KeyCode;
use ratatui::{DefaultTerminal, Frame};

use crate::event::{self, Event};
use crate::models::cpu::CpuUsage;
use crate::models::memory::MemoryInfo;
use crate::models::process::{ProcessDetails, ProcessInfo, SortBy};
use crate::system::cpu::CpuSampler;
use crate::system::processes::ProcessSampler;

const CPU_REFRESH: Duration = Duration::from_millis(500);
const MEMORY_REFRESH: Duration = Duration::from_millis(1000);
const PROCESS_REFRESH: Duration = Duration::from_millis(1000);

pub(crate) enum View {
    List,
    Details {
        details: ProcessDetails,
        confirm_kill: bool,
    },
}

pub struct App {
    should_quit: bool,
    cpu: CpuSampler,
    pub(crate) cpu_usage: Option<CpuUsage>,
    pub(crate) memory: Option<MemoryInfo>,
    last_cpu: Instant,
    last_memory: Instant,
    processes: ProcessSampler,
    pub(crate) process_list: Vec<ProcessInfo>,
    pub(crate) selection: usize,
    pub(crate) sort_by: SortBy,
    last_processes: Instant,
    pub(crate) view: View,
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
            processes: ProcessSampler::new(),
            process_list: Vec::new(),
            selection: 0,
            sort_by: SortBy::Cpu,
            last_processes: Instant::now(),
            view: View::List,
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
        if now.duration_since(self.last_processes) >= PROCESS_REFRESH {
            if let Some(mut list) = self.processes.sample() {
                Self::sort_processes(&mut list, self.sort_by);
                self.process_list = list;
                self.clamp_selection();
            }
            if let View::Details { details, .. } = &mut self.view {
                if let Some(p) = self.process_list.iter().find(|p| p.pid == details.pid) {
                    details.cpu = p.cpu;
                    details.memory = p.memory;
                }
            }
            self.last_processes = now;
        }
    }

    fn sort_processes(list: &mut [ProcessInfo], sort_by: SortBy) {
        match sort_by {
            SortBy::Cpu => list.sort_by(|a, b| b.cpu.total_cmp(&a.cpu)),
            SortBy::Memory => list.sort_by(|a, b| b.memory.cmp(&a.memory)),
            SortBy::Pid => list.sort_by_key(|p| p.pid),
            SortBy::Name => list.sort_by(|a, b| a.name.cmp(&b.name)),
        }
    }

    fn set_sort(&mut self, sort_by: SortBy) {
        self.sort_by = sort_by;
        Self::sort_processes(&mut self.process_list, self.sort_by);
        self.clamp_selection();
    }

    fn move_selection(&mut self, delta: isize) {
        if self.process_list.is_empty() {
            return;
        }
        let len = self.process_list.len();
        let next = self.selection as isize + delta;
        self.selection = next.clamp(0, len as isize - 1) as usize;
    }

    fn clamp_selection(&mut self) {
        if self.process_list.is_empty() {
            self.selection = 0;
        } else {
            self.selection = self.selection.min(self.process_list.len() - 1);
        }
    }

    fn on_key(&mut self, key: crossterm::event::KeyEvent) {
        let in_details = matches!(self.view, View::Details { .. });
        let in_confirm = matches!(self.view, View::Details { confirm_kill: true, .. });
        if in_confirm {
            self.on_key_confirm(key);
        } else if in_details {
            self.on_key_details(key);
        } else {
            self.on_key_list(key);
        }
    }

    fn on_key_list(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Up | KeyCode::Char('k') => self.move_selection(-1),
            KeyCode::Down | KeyCode::Char('j') => self.move_selection(1),
            KeyCode::Char('c') => self.set_sort(SortBy::Cpu),
            KeyCode::Char('m') => self.set_sort(SortBy::Memory),
            KeyCode::Char('p') => self.set_sort(SortBy::Pid),
            KeyCode::Char('a') => self.set_sort(SortBy::Name),
            KeyCode::Enter => self.open_details(),
            _ => {}
        }
    }

    fn on_key_details(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Char('b') | KeyCode::Esc => self.view = View::List,
            KeyCode::Char('k') => {
                if let View::Details { confirm_kill, .. } = &mut self.view {
                    *confirm_kill = true;
                }
            }
            KeyCode::Char('q') => self.should_quit = true,
            _ => {}
        }
    }

    fn on_key_confirm(&mut self, key: crossterm::event::KeyEvent) {
        let pid = match &self.view {
            View::Details { details, .. } => details.pid,
            _ => return,
        };
        match key.code {
            KeyCode::Char('y') => {
                crate::system::processes::kill(pid, libc::SIGTERM);
                self.view = View::List;
            }
            KeyCode::Char('n') | KeyCode::Esc => {
                if let View::Details { confirm_kill, .. } = &mut self.view {
                    *confirm_kill = false;
                }
            }
            _ => {}
        }
    }

    fn open_details(&mut self) {
        let Some(process) = self.process_list.get(self.selection) else {
            return;
        };
        let Some(mut details) = crate::system::processes::details(process.pid) else {
            return;
        };
        details.cpu = process.cpu;
        details.memory = process.memory;
        self.view = View::Details {
            details,
            confirm_kill: false,
        };
    }
}

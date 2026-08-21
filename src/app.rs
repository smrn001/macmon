use std::io;
use std::time::{Duration, Instant};

use crossterm::event::KeyCode;
use ratatui::{DefaultTerminal, Frame};

use crate::event::{self, Event};
use crate::models::cpu::CpuUsage;
use crate::models::disk::DiskInfo;
use crate::models::memory::MemoryInfo;
use crate::models::network::NetworkUsage;
use crate::models::process::{ProcessDetails, ProcessInfo, SortBy};
use crate::system::cpu::CpuSampler;
use crate::system::disk::DiskSampler;
use crate::system::network::NetworkSampler;
use crate::system::processes::ProcessSampler;

const CPU_REFRESH: Duration = Duration::from_millis(500);
const MEMORY_REFRESH: Duration = Duration::from_millis(1000);
const PROCESS_REFRESH: Duration = Duration::from_millis(1000);
const NETWORK_REFRESH: Duration = Duration::from_millis(1000);
const DISK_REFRESH: Duration = Duration::from_millis(1000);

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
    network: NetworkSampler,
    pub(crate) network_usage: Option<NetworkUsage>,
    last_network: Instant,
    disk: DiskSampler,
    pub(crate) disk_info: Option<DiskInfo>,
    last_disk: Instant,
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
            network: NetworkSampler::new(),
            network_usage: None,
            last_network: Instant::now(),
            disk: DiskSampler::new(),
            disk_info: None,
            last_disk: Instant::now(),
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
        let mut dirty = true;
        while !self.should_quit {
            if dirty {
                terminal.draw(|frame| self.render(frame))?;
                dirty = false;
            }
            match event::read()? {
                Event::Key(key) => {
                    self.on_key(key);
                    dirty = true;
                }
                Event::Resize => {
                    terminal.clear()?;
                    dirty = true;
                }
                Event::Tick => dirty |= self.refresh(),
            }
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        crate::ui::render(frame, self);
    }

    fn refresh(&mut self) -> bool {
        let now = Instant::now();
        let mut updated = false;
        if now.duration_since(self.last_cpu) >= CPU_REFRESH {
            if let Some(usage) = self.cpu.sample() {
                self.cpu_usage = Some(usage);
                updated = true;
            }
            self.last_cpu = now;
        }
        if now.duration_since(self.last_memory) >= MEMORY_REFRESH {
            let previous = self.memory.take();
            self.memory = crate::system::memory::sample();
            if self.memory != previous {
                updated = true;
            }
            self.last_memory = now;
        }
        if now.duration_since(self.last_network) >= NETWORK_REFRESH {
            if let Some(usage) = self.network.sample() {
                self.network_usage = Some(usage);
                updated = true;
            }
            self.last_network = now;
        }
        if now.duration_since(self.last_disk) >= DISK_REFRESH {
            if let Some(info) = self.disk.sample() {
                self.disk_info = Some(info);
                updated = true;
            }
            self.last_disk = now;
        }
        if now.duration_since(self.last_processes) >= PROCESS_REFRESH {
            if let Some(mut list) = self.processes.sample() {
                Self::sort_processes(&mut list, self.sort_by);
                self.process_list = list;
                self.clamp_selection();
                updated = true;
            }
            if let View::Details { details, .. } = &mut self.view
                && let Some(p) = self.process_list.iter().find(|p| p.pid == details.pid)
                && (details.cpu != p.cpu || details.memory != p.memory)
            {
                details.cpu = p.cpu;
                details.memory = p.memory;
                updated = true;
            }
            self.last_processes = now;
        }
        updated
    }

    fn sort_processes(list: &mut [ProcessInfo], sort_by: SortBy) {
        match sort_by {
            SortBy::Cpu => list.sort_by(|a, b| b.cpu.total_cmp(&a.cpu)),
            SortBy::Memory => list.sort_by_key(|p| std::cmp::Reverse(p.memory)),
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
        let in_confirm = matches!(
            self.view,
            View::Details {
                confirm_kill: true,
                ..
            }
        );
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

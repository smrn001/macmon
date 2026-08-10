#[derive(Clone, Debug)]
pub struct ProcessInfo {
    pub pid: i32,
    pub name: String,
    pub state: char,
    pub threads: i32,
    pub cpu_time: u64,
    pub cpu: f64,
    pub memory: u64,
}

#[derive(Clone, Copy)]
pub enum SortBy {
    Cpu,
    Memory,
    Pid,
    Name,
}

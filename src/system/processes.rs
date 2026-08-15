use std::ffi::CStr;
use std::time::Instant;

use libc::{proc_listpids, proc_pidinfo, proc_pidpath, proc_taskallinfo, PROC_PIDTASKALLINFO};

use crate::models::process::{ProcessDetails, ProcessInfo};

const MAX_PIDS: usize = 4096;
const MAX_PATH: u32 = 4096;
const PROC_ALL_PIDS: u32 = 1;

pub struct ProcessSampler {
    previous: Vec<(i32, u64)>,
    last_sample: Option<Instant>,
}

impl ProcessSampler {
    pub fn new() -> Self {
        Self {
            previous: Vec::new(),
            last_sample: None,
        }
    }

    pub fn sample(&mut self) -> Option<Vec<ProcessInfo>> {
        let now = Instant::now();
        let elapsed = self
            .last_sample
            .map(|t| now.duration_since(t).as_secs_f64())
            .unwrap_or(1.0);

        let mut procs = read_processes()?;
        for proc in &mut procs {
            let previous = self
                .previous
                .iter()
                .find(|(pid, _)| *pid == proc.pid)
                .map(|(_, time)| *time)
                .unwrap_or(proc.cpu_time);
            let delta = proc.cpu_time.saturating_sub(previous);
            proc.cpu = (delta as f64 / 1e9) / elapsed * 100.0;
        }

        self.previous = procs.iter().map(|p| (p.pid, p.cpu_time)).collect();
        self.last_sample = Some(now);
        Some(procs)
    }
}

impl Default for ProcessSampler {
    fn default() -> Self {
        Self::new()
    }
}

fn read_processes() -> Option<Vec<ProcessInfo>> {
    unsafe {
        let mut pids = vec![0i32; MAX_PIDS];
        let bytes = proc_listpids(
            PROC_ALL_PIDS,
            0,
            pids.as_mut_ptr() as *mut libc::c_void,
            (MAX_PIDS * std::mem::size_of::<i32>()) as libc::c_int,
        );
        if bytes <= 0 {
            return None;
        }
        let count = (bytes as usize) / std::mem::size_of::<i32>();
        let mut procs = Vec::with_capacity(count);
        for &pid in pids.iter().take(count) {
            if pid <= 0 {
                continue;
            }
            if let Some(info) = process_info(pid) {
                procs.push(info);
            }
        }
        Some(procs)
    }
}

fn process_info(pid: i32) -> Option<ProcessInfo> {
    unsafe {
        let mut info = std::mem::zeroed::<proc_taskallinfo>();
        let size = std::mem::size_of::<proc_taskallinfo>();
        let result = proc_pidinfo(
            pid,
            PROC_PIDTASKALLINFO,
            0,
            &mut info as *mut proc_taskallinfo as *mut libc::c_void,
            size as libc::c_int,
        );
        if result as usize != size {
            return None;
        }

        let pbsd = info.pbsd;
        let ptinfo = info.ptinfo;
        Some(ProcessInfo {
            pid,
            name: bytes_to_string(&pbsd.pbi_name),
            state: state_char(pbsd.pbi_status),
            threads: ptinfo.pti_threadnum,
            cpu_time: ptinfo.pti_total_user + ptinfo.pti_total_system,
            cpu: 0.0,
            memory: ptinfo.pti_resident_size,
        })
    }
}

fn bytes_to_string(bytes: &[libc::c_char]) -> String {
    let raw: &[u8] = unsafe {
        std::slice::from_raw_parts(bytes.as_ptr() as *const u8, bytes.len())
    };
    let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
    String::from_utf8_lossy(&raw[..end]).into_owned()
}

pub fn details(pid: i32) -> Option<ProcessDetails> {
    unsafe {
        let mut info = std::mem::zeroed::<proc_taskallinfo>();
        let size = std::mem::size_of::<proc_taskallinfo>();
        let result = proc_pidinfo(
            pid,
            PROC_PIDTASKALLINFO,
            0,
            &mut info as *mut proc_taskallinfo as *mut libc::c_void,
            size as libc::c_int,
        );
        if result as usize != size {
            return None;
        }

        let pbsd = info.pbsd;
        let ptinfo = info.ptinfo;
        Some(ProcessDetails {
            pid,
            name: bytes_to_string(&pbsd.pbi_name),
            parent: pbsd.pbi_ppid as i32,
            user: username(pbsd.pbi_uid),
            state: state_char(pbsd.pbi_status),
            cpu: 0.0,
            memory: ptinfo.pti_resident_size,
            threads: ptinfo.pti_threadnum,
            executable: executable_path(pid),
        })
    }
}

pub fn kill(pid: i32, signal: i32) -> bool {
    unsafe { libc::kill(pid, signal) == 0 }
}

fn executable_path(pid: i32) -> String {
    unsafe {
        let mut path = [0u8; MAX_PATH as usize];
        let len = proc_pidpath(pid, path.as_mut_ptr() as *mut libc::c_void, MAX_PATH);
        if len <= 0 {
            return String::new();
        }
        let end = path[..len as usize]
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(len as usize);
        String::from_utf8_lossy(&path[..end]).into_owned()
    }
}

fn username(uid: libc::uid_t) -> String {
    unsafe {
        let pw = libc::getpwuid(uid);
        if pw.is_null() {
            return uid.to_string();
        }
        let name = CStr::from_ptr((*pw).pw_name).to_string_lossy().into_owned();
        if name.is_empty() {
            uid.to_string()
        } else {
            name
        }
    }
}

fn state_char(status: u32) -> char {
    match status {
        1 => 'I',
        2 => 'R',
        3 => 'S',
        4 => 'T',
        5 => 'Z',
        _ => '?',
    }
}

#[cfg(test)]
mod tests {
    use std::process::id;
    use std::thread;
    use std::time::{Duration, Instant};

    use super::*;

    #[test]
    fn cpu_time_deltas() {
        let pid = id() as i32;
        let spin = || {
            let start = Instant::now();
            let mut x = 0u64;
            while start.elapsed().as_millis() < 600 {
                x = x.wrapping_add(1);
            }
            std::hint::black_box(x);
        };

        spin();
        let mut sampler = ProcessSampler::new();
        let first = sampler
            .sample()
            .unwrap()
            .iter()
            .find(|p| p.pid == pid)
            .cloned();
        thread::sleep(Duration::from_millis(300));
        spin();
        let second = sampler
            .sample()
            .unwrap()
            .iter()
            .find(|p| p.pid == pid)
            .cloned();

        println!("first={first:?}");
        println!("second={second:?}");
        if let (Some(a), Some(b)) = (first, second) {
            let delta = b.cpu_time.saturating_sub(a.cpu_time);
            println!("delta cpu_time = {delta} ns, cpu% = {:.1}", b.cpu);
            assert!(b.cpu > 1.0, "expected measurable CPU usage, got {:.1}%", b.cpu);
        }
    }
}

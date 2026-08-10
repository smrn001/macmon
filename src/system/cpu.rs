use std::ptr;

#[allow(deprecated)]
use libc::{
    host_processor_info, mach_host_self, mach_msg_type_number_t, mach_task_self, natural_t,
    vm_deallocate, vm_size_t, CPU_STATE_IDLE, CPU_STATE_MAX, CPU_STATE_NICE, CPU_STATE_SYSTEM,
    CPU_STATE_USER, KERN_SUCCESS, PROCESSOR_CPU_LOAD_INFO,
};

use crate::models::cpu::{usage, CpuCore, CpuSample, CpuUsage};

pub struct CpuSampler {
    previous: Option<CpuSample>,
}

impl CpuSampler {
    pub fn new() -> Self {
        Self { previous: None }
    }

    pub fn sample(&mut self) -> Option<CpuUsage> {
        let current = read_sample()?;
        let result = self
            .previous
            .as_ref()
            .map(|previous| usage(previous, &current));
        self.previous = Some(current);
        result
    }
}

impl Default for CpuSampler {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(deprecated)]
fn read_sample() -> Option<CpuSample> {
    unsafe {
        let mut processor_count: natural_t = 0;
        let mut cpu_info: *mut i32 = ptr::null_mut();
        let mut out_count: mach_msg_type_number_t = 0;

        let host = mach_host_self();
        let result = host_processor_info(
            host,
            PROCESSOR_CPU_LOAD_INFO,
            &mut processor_count,
            &mut cpu_info,
            &mut out_count,
        );

        if result != KERN_SUCCESS || cpu_info.is_null() {
            return None;
        }

        let core_count = out_count as usize;
        let state_count = CPU_STATE_MAX as usize;
        let info = std::slice::from_raw_parts(cpu_info, core_count * state_count);

        let cores = info
            .chunks(state_count)
            .map(|chunk| CpuCore {
                user: chunk[CPU_STATE_USER as usize] as u64,
                system: chunk[CPU_STATE_SYSTEM as usize] as u64,
                idle: chunk[CPU_STATE_IDLE as usize] as u64,
                nice: chunk[CPU_STATE_NICE as usize] as u64,
            })
            .collect();

        let bytes = core_count * state_count * std::mem::size_of::<i32>();
        vm_deallocate(mach_task_self(), cpu_info as usize, bytes as vm_size_t);

        Some(CpuSample { cores })
    }
}

use std::mem::size_of;
use std::ptr;

#[allow(deprecated)]
use libc::{
    HOST_VM_INFO64, HOST_VM_INFO64_COUNT, KERN_SUCCESS, sysctlbyname, vm_statistics64, xsw_usage,
};

use crate::models::memory::MemoryInfo;

pub fn sample() -> Option<MemoryInfo> {
    let physical = physical_memory()?;
    let stats = vm_stats()?;
    let page_size = stats.0;
    let s = &stats.1;

    let swap = swap_usage().unwrap_or(unsafe { std::mem::zeroed::<xsw_usage>() });

    Some(MemoryInfo {
        physical,
        free: u64::from(s.free_count) * page_size,
        active: u64::from(s.active_count) * page_size,
        inactive: u64::from(s.inactive_count) * page_size,
        wired: u64::from(s.wire_count) * page_size,
        compressed: u64::from(s.compressor_page_count) * page_size,
        purgeable: u64::from(s.purgeable_count) * page_size,
        swap_total: swap.xsu_total,
        swap_used: swap.xsu_used,
    })
}

#[allow(deprecated)]
fn vm_stats() -> Option<(u64, vm_statistics64)> {
    unsafe {
        let mut stats = std::mem::zeroed::<vm_statistics64>();
        let mut count = HOST_VM_INFO64_COUNT as libc::mach_msg_type_number_t;
        let host = libc::mach_host_self();
        let result = libc::host_statistics64(
            host,
            HOST_VM_INFO64,
            &mut stats as *mut vm_statistics64 as *mut i32,
            &mut count,
        );
        if result != KERN_SUCCESS {
            return None;
        }
        let page_size = sysctl_u64(c"hw.pagesize")?;
        Some((page_size, stats))
    }
}

fn physical_memory() -> Option<u64> {
    sysctl_u64(c"hw.memsize")
}

fn swap_usage() -> Option<xsw_usage> {
    unsafe {
        let mut usage = std::mem::zeroed::<xsw_usage>();
        let mut len = size_of::<xsw_usage>();
        let result = sysctlbyname(
            c"vm.swapusage".as_ptr(),
            &mut usage as *mut xsw_usage as *mut libc::c_void,
            &mut len,
            ptr::null_mut(),
            0,
        );
        if result != 0 {
            return None;
        }
        Some(usage)
    }
}

fn sysctl_u64(name: &std::ffi::CStr) -> Option<u64> {
    unsafe {
        let mut value: u64 = 0;
        let mut len = size_of::<u64>();
        let result = sysctlbyname(
            name.as_ptr(),
            &mut value as *mut u64 as *mut libc::c_void,
            &mut len,
            ptr::null_mut(),
            0,
        );
        if result != 0 {
            return None;
        }
        Some(value)
    }
}

use std::ffi::c_char;
use std::mem::zeroed;
use std::ptr::null;
use std::time::Instant;

use crate::models::disk::DiskInfo;

pub struct DiskSampler {
    previous: Option<(u64, u64, Instant)>,
}

impl DiskSampler {
    pub fn new() -> Self {
        Self { previous: None }
    }

    pub fn sample(&mut self) -> Option<DiskInfo> {
        let (capacity, used, available) = storage_usage()?;
        let now = Instant::now();

        // First sample has no baseline for rates; report zero rather than
        // a spike from process start.
        let io = io_totals();
        let (read_rate, write_rate) = match (self.previous, io) {
            (Some((prev_read, prev_write, prev_at)), Some((read, write))) => {
                let elapsed = now.duration_since(prev_at).as_secs_f64();
                if elapsed > 0.0 {
                    (
                        read.saturating_sub(prev_read) as f64 / elapsed,
                        write.saturating_sub(prev_write) as f64 / elapsed,
                    )
                } else {
                    (0.0, 0.0)
                }
            }
            _ => (0.0, 0.0),
        };

        self.previous = io.map(|(read, write)| (read, write, now));

        Some(DiskInfo {
            capacity,
            used,
            available,
            read_rate,
            write_rate,
        })
    }
}

impl Default for DiskSampler {
    fn default() -> Self {
        Self::new()
    }
}

fn storage_usage() -> Option<(u64, u64, u64)> {
    unsafe {
        let mut fs: libc::statfs = zeroed();
        if libc::statfs(c"/".as_ptr(), &mut fs) != 0 {
            return None;
        }
        let block = fs.f_bsize as u64;
        Some((
            fs.f_blocks * block,
            (fs.f_blocks - fs.f_bfree) * block,
            fs.f_bavail * block,
        ))
    }
}

// --- IOKit disk statistics (same source as `iostat`) ---

type CFDictionaryRef = *const std::ffi::c_void;
type CFMutableDictionaryRef = *mut std::ffi::c_void;
type CFStringRef = *const std::ffi::c_void;
type CFNumberRef = *const std::ffi::c_void;
type IoObject = u32;

const KERN_SUCCESS: i32 = 0;
const K_CF_NUMBER_SINT64: isize = 4;
const K_CF_STRING_UTF8: u32 = 0x0800_0100;

#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
    fn IOServiceMatching(name: *const c_char) -> CFMutableDictionaryRef;
    fn IOServiceGetMatchingServices(
        main_port: u32,
        matching: CFMutableDictionaryRef,
        existing: *mut IoObject,
    ) -> i32;
    fn IOIteratorNext(iterator: IoObject) -> IoObject;
    fn IOObjectRelease(object: IoObject) -> i32;
    fn IORegistryEntryCreateCFProperties(
        entry: IoObject,
        properties: *mut CFMutableDictionaryRef,
        allocator: *const std::ffi::c_void,
        options: u32,
    ) -> i32;
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFStringCreateWithCString(
        alloc: *const std::ffi::c_void,
        c_str: *const c_char,
        encoding: u32,
    ) -> CFStringRef;
    fn CFDictionaryGetValue(dictionary: CFDictionaryRef, key: *const std::ffi::c_void)
        -> *const std::ffi::c_void;
    fn CFNumberGetValue(
        number: CFNumberRef,
        the_type: isize,
        value_ptr: *mut std::ffi::c_void,
    ) -> u8;
    fn CFRelease(cf: *const std::ffi::c_void);
}

/// Sums read/write byte counters across all block storage drivers.
fn io_totals() -> Option<(u64, u64)> {
    unsafe {
        let matching = IOServiceMatching(b"IOBlockStorageDriver\0".as_ptr() as *const c_char);
        if matching.is_null() {
            return None;
        }
        let mut iterator: IoObject = 0;
        if IOServiceGetMatchingServices(0, matching, &mut iterator) != KERN_SUCCESS {
            CFRelease(matching);
            return None;
        }

        let mut read = 0u64;
        let mut write = 0u64;
        loop {
            let entry = IOIteratorNext(iterator);
            if entry == 0 {
                break;
            }
            let mut props: CFMutableDictionaryRef = std::ptr::null_mut();
            if IORegistryEntryCreateCFProperties(entry, &mut props, null(), 0) == KERN_SUCCESS
                && !props.is_null()
            {
                if let Some(stats) = dict(props, "Statistics") {
                    read += number(stats, "Bytes (Read)");
                    write += number(stats, "Bytes (Write)");
                }
                CFRelease(props);
            }
            IOObjectRelease(entry);
        }
        IOObjectRelease(iterator);

        if read == 0 && write == 0 {
            return None;
        }
        Some((read, write))
    }
}

unsafe fn dict(dictionary: CFDictionaryRef, key: &str) -> Option<CFDictionaryRef> {
    unsafe {
        let key = cf_string(key)?;
        let value = CFDictionaryGetValue(dictionary, key as *const _);
        CFRelease(key);
        (!value.is_null()).then_some(value)
    }
}

unsafe fn number(dictionary: CFDictionaryRef, key: &str) -> u64 {
    unsafe {
        let Some(key) = cf_string(key) else {
            return 0;
        };
        let value = CFDictionaryGetValue(dictionary, key as *const _);
        CFRelease(key);
        if value.is_null() {
            return 0;
        }
        let mut out: u64 = 0;
        CFNumberGetValue(
            value as CFNumberRef,
            K_CF_NUMBER_SINT64,
            &mut out as *mut u64 as *mut _,
        );
        out
    }
}

unsafe fn cf_string(value: &str) -> Option<CFStringRef> {
    unsafe {
        let Ok(cstr) = std::ffi::CString::new(value) else {
            return None;
        };
        let s = CFStringCreateWithCString(null(), cstr.as_ptr(), K_CF_STRING_UTF8);
        (!s.is_null()).then_some(s)
    }
}

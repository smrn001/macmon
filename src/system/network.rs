use std::time::Instant;
use std::ptr;

use libc::{freeifaddrs, getifaddrs, if_data, ifaddrs, AF_LINK, IFF_LOOPBACK};

use crate::models::network::NetworkUsage;

pub struct NetworkSampler {
    previous: Option<(u64, u64, Instant)>,
}

impl NetworkSampler {
    pub fn new() -> Self {
        Self { previous: None }
    }

    pub fn sample(&mut self) -> Option<NetworkUsage> {
        let (rx, tx) = totals()?;
        let now = Instant::now();

        let usage = match self.previous {
            None => NetworkUsage {
                download_rate: 0.0,
                upload_rate: 0.0,
                total_downloaded: rx,
                total_uploaded: tx,
            },
            Some((prev_rx, prev_tx, prev_at)) => {
                let elapsed = now.duration_since(prev_at).as_secs_f64();
                let (drx, dtx) = (
                    rx.saturating_sub(prev_rx),
                    tx.saturating_sub(prev_tx),
                );
                let rate = |delta: u64| {
                    if elapsed > 0.0 {
                        delta as f64 / elapsed
                    } else {
                        0.0
                    }
                };
                NetworkUsage {
                    download_rate: rate(drx),
                    upload_rate: rate(dtx),
                    total_downloaded: rx,
                    total_uploaded: tx,
                }
            }
        };

        self.previous = Some((rx, tx, now));
        Some(usage)
    }
}

impl Default for NetworkSampler {
    fn default() -> Self {
        Self::new()
    }
}

fn totals() -> Option<(u64, u64)> {
    unsafe {
        let mut list: *mut ifaddrs = ptr::null_mut();
        if getifaddrs(&mut list) != 0 {
            return None;
        }

        let mut rx = 0u64;
        let mut tx = 0u64;
        let mut current = list;
        while !current.is_null() {
            let ifa = &*current;
            let is_link = !ifa.ifa_addr.is_null()
                && (*ifa.ifa_addr).sa_family == AF_LINK as u8;
            let is_loopback = ifa.ifa_flags & IFF_LOOPBACK as u32 != 0;
            if is_link && !is_loopback && !ifa.ifa_data.is_null() {
                let data = &*(ifa.ifa_data as *const if_data);
                rx += u64::from(data.ifi_ibytes);
                tx += u64::from(data.ifi_obytes);
            }
            current = ifa.ifa_next;
        }

        freeifaddrs(list);
        Some((rx, tx))
    }
}

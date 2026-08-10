pub struct MemoryInfo {
    pub physical: u64,
    pub free: u64,
    pub active: u64,
    pub inactive: u64,
    pub wired: u64,
    pub compressed: u64,
    pub purgeable: u64,
    pub swap_total: u64,
    pub swap_used: u64,
}

impl MemoryInfo {
    pub fn used(&self) -> u64 {
        self.active + self.wired + self.compressed
    }

    pub fn cached(&self) -> u64 {
        self.inactive + self.purgeable
    }

    pub fn used_ratio(&self) -> f64 {
        if self.physical == 0 {
            0.0
        } else {
            self.used() as f64 / self.physical as f64
        }
    }
}

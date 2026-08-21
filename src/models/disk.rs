pub struct DiskInfo {
    pub capacity: u64,
    pub used: u64,
    pub available: u64,
    pub read_rate: f64,
    pub write_rate: f64,
}

impl DiskInfo {
    pub fn used_ratio(&self) -> f64 {
        if self.capacity == 0 {
            0.0
        } else {
            self.used as f64 / self.capacity as f64
        }
    }
}

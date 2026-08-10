pub struct CpuCore {
    pub user: u64,
    pub system: u64,
    pub idle: u64,
    pub nice: u64,
}

pub struct CpuSample {
    pub cores: Vec<CpuCore>,
}

impl CpuSample {
    pub fn user_ticks(&self) -> u64 {
        self.cores.iter().map(|c| c.user + c.nice).sum()
    }

    pub fn system_ticks(&self) -> u64 {
        self.cores.iter().map(|c| c.system).sum()
    }

    pub fn idle_ticks(&self) -> u64 {
        self.cores.iter().map(|c| c.idle).sum()
    }
}

pub struct CpuUsage {
    pub total: f64,
    pub user: f64,
    pub system: f64,
    pub idle: f64,
    pub cores: Vec<f64>,
}

pub fn usage(previous: &CpuSample, current: &CpuSample) -> CpuUsage {
    let cores = previous
        .cores
        .iter()
        .zip(current.cores.iter())
        .map(|(a, b)| {
            let user = b.user.saturating_sub(a.user);
            let system = b.system.saturating_sub(a.system);
            let nice = b.nice.saturating_sub(a.nice);
            let idle = b.idle.saturating_sub(a.idle);
            let total = user + system + nice + idle;
            if total == 0 {
                0.0
            } else {
                ((user + system + nice) as f64 / total as f64) * 100.0
            }
        })
        .collect();

    let user = current.user_ticks().saturating_sub(previous.user_ticks());
    let system = current
        .system_ticks()
        .saturating_sub(previous.system_ticks());
    let idle = current.idle_ticks().saturating_sub(previous.idle_ticks());
    let total = user + system + idle;
    let (total_pct, user_pct, system_pct, idle_pct) = if total == 0 {
        (0.0, 0.0, 0.0, 0.0)
    } else {
        let t = total as f64;
        (
            ((user + system) as f64 / t) * 100.0,
            (user as f64 / t) * 100.0,
            (system as f64 / t) * 100.0,
            (idle as f64 / t) * 100.0,
        )
    };

    CpuUsage {
        total: total_pct,
        user: user_pct,
        system: system_pct,
        idle: idle_pct,
        cores,
    }
}

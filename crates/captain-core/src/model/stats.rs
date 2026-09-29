/// One resource sample for a container. The engine sends one about every second.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct StatsSample {
    /// CPU use as a percent of one core, so four busy cores read 400.
    pub cpu_percent: f64,
    pub memory_bytes: u64,
    /// Total bytes received on all networks since the container started.
    pub rx_bytes: u64,
    /// Total bytes sent on all networks since the container started.
    pub tx_bytes: u64,
}

/// Computes CPU percent the way `docker stats` does, from two readings of the
/// container's and the host's CPU counters.
pub fn cpu_percent(
    total: u64,
    previous_total: u64,
    system: u64,
    previous_system: u64,
    online_cpus: u32,
) -> f64 {
    // The first sample of a stream has no previous reading.
    if previous_system == 0 {
        return 0.0;
    }
    let cpu_delta = total.saturating_sub(previous_total) as f64;
    let system_delta = system.saturating_sub(previous_system) as f64;
    if cpu_delta <= 0.0 || system_delta <= 0.0 {
        return 0.0;
    }
    cpu_delta / system_delta * f64::from(online_cpus.max(1)) * 100.0
}

#[cfg(test)]
mod tests;

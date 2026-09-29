use std::collections::HashMap;

use bollard::models::{ContainerCpuStats, ContainerStatsResponse};
use captain_core::model::{StatsSample, cpu_percent};

pub fn stats(response: ContainerStatsResponse) -> StatsSample {
    let cpu = cpu(response.cpu_stats.as_ref(), response.precpu_stats.as_ref());
    let memory_bytes = response
        .memory_stats
        .map(|memory| {
            let stats = memory.stats.unwrap_or_default();
            memory_used(memory.usage.unwrap_or_default(), &stats)
        })
        .unwrap_or_default();
    let (rx_bytes, tx_bytes) =
        response
            .networks
            .unwrap_or_default()
            .values()
            .fold((0, 0), |(rx, tx), net| {
                (
                    rx + net.rx_bytes.unwrap_or_default(),
                    tx + net.tx_bytes.unwrap_or_default(),
                )
            });

    StatsSample {
        cpu_percent: cpu,
        memory_bytes,
        rx_bytes,
        tx_bytes,
    }
}

/// Memory use without the reclaimable page cache, as `docker stats` shows it:
/// `total_inactive_file` on cgroup v1, `inactive_file` on v2, and the raw usage when
/// the value is not below it. See calculateMemUsageUnixNoCache in
/// <https://github.com/docker/cli/blob/master/cli/command/container/stats_helpers.go>.
fn memory_used(usage: u64, stats: &HashMap<String, u64>) -> u64 {
    let cache = match stats.get("total_inactive_file") {
        Some(&v1) if v1 < usage => Some(v1),
        _ => stats.get("inactive_file").copied().filter(|&v2| v2 < usage),
    };
    usage - cache.unwrap_or_default()
}

fn cpu(current: Option<&ContainerCpuStats>, previous: Option<&ContainerCpuStats>) -> f64 {
    let total = |s: Option<&ContainerCpuStats>| {
        s.and_then(|s| s.cpu_usage.as_ref())
            .and_then(|u| u.total_usage)
            .unwrap_or_default()
    };
    let system =
        |s: Option<&ContainerCpuStats>| s.and_then(|s| s.system_cpu_usage).unwrap_or_default();
    let cpus = current
        .and_then(|s| {
            s.online_cpus.or_else(|| {
                s.cpu_usage
                    .as_ref()
                    .and_then(|u| u.percpu_usage.as_ref())
                    .map(|p| p.len() as u32)
            })
        })
        .unwrap_or(1);
    cpu_percent(
        total(current),
        total(previous),
        system(current),
        system(previous),
        cpus,
    )
}

#[cfg(test)]
mod tests;

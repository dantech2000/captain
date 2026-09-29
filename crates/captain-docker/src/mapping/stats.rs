use bollard::models::{ContainerCpuStats, ContainerStatsResponse};
use captain_core::model::{StatsSample, cpu_percent};

pub fn stats(response: ContainerStatsResponse) -> StatsSample {
    let cpu = cpu(response.cpu_stats.as_ref(), response.precpu_stats.as_ref());
    let memory_bytes = response
        .memory_stats
        .map(|memory| {
            let usage = memory.usage.unwrap_or_default();
            // cgroup v2 reports reclaimable page cache as inactive_file, v1 as cache.
            // `docker stats` subtracts it, so Captain does too.
            let cache = memory
                .stats
                .and_then(|s| s.get("inactive_file").or_else(|| s.get("cache")).copied())
                .unwrap_or_default();
            usage.saturating_sub(cache)
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

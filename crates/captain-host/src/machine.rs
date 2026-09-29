//! The CPUs and memory of this computer, for the default engine resources.

use captain_core::{GIB, HostResources};

/// Used when the memory size cannot be read.
const FALLBACK_MEMORY: u64 = 8 * GIB;

/// The default resources for Captain Engine on this computer. See ADR 0008.
pub fn recommended_resources() -> HostResources {
    HostResources::recommended(host_cpus(), host_memory())
}

/// The number of CPUs this computer has.
pub fn host_cpus() -> u32 {
    std::thread::available_parallelism().map_or(4, |n| u32::try_from(n.get()).unwrap_or(u32::MAX))
}

/// The memory of this computer in bytes.
pub fn host_memory() -> u64 {
    read_memory().unwrap_or(FALLBACK_MEMORY)
}

#[cfg(target_os = "macos")]
fn read_memory() -> Option<u64> {
    let output = std::process::Command::new("/usr/sbin/sysctl")
        .args(["-n", "hw.memsize"])
        .output()
        .ok()?;
    parse_sysctl(&String::from_utf8_lossy(&output.stdout))
}

#[cfg(target_os = "linux")]
fn read_memory() -> Option<u64> {
    parse_meminfo(&std::fs::read_to_string("/proc/meminfo").ok()?)
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn read_memory() -> Option<u64> {
    None
}

/// Reads `sysctl -n hw.memsize`: a byte count.
pub fn parse_sysctl(output: &str) -> Option<u64> {
    output.trim().parse().ok().filter(|&bytes| bytes > 0)
}

/// Reads the `MemTotal:` line of `/proc/meminfo`, which is in KiB.
pub fn parse_meminfo(meminfo: &str) -> Option<u64> {
    let line = meminfo.lines().find(|l| l.starts_with("MemTotal:"))?;
    let kib: u64 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(kib * 1024).filter(|&bytes| bytes > 0)
}

#[cfg(test)]
mod tests;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::format::bytes_label;

/// One gibibyte in bytes.
pub const GIB: u64 = 1024 * 1024 * 1024;

const MIN_CPUS: u32 = 2;
const MAX_DEFAULT_CPUS: u32 = 8;
const MIN_MEMORY: u64 = 4 * GIB;
const MAX_DEFAULT_MEMORY: u64 = 16 * GIB;
const DEFAULT_DISK: u64 = 64 * GIB;
/// Docker needs room for a few images; smaller disks fill up at once.
const MIN_DISK: u64 = 16 * GIB;
const MAX_DISK: u64 = 1024 * GIB;
const MIN_USER_MEMORY: u64 = 2 * GIB;

/// The CPUs, memory, and disk that the engine's machine gets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct HostResources {
    /// The number of CPUs, from 1 to the number of CPUs of this computer.
    #[schemars(range(min = 1))]
    pub cpus: u32,
    /// Memory in bytes, at least 2 GiB (2147483648).
    #[schemars(range(min = 2147483648u64))]
    pub memory_bytes: u64,
    /// The disk size in bytes, from 16 GiB to 1 TiB. The disk is sparse, so it takes
    /// only the space in use, and it cannot shrink.
    #[schemars(range(min = 17179869184u64, max = 1099511627776u64))]
    pub disk_bytes: u64,
}

impl HostResources {
    /// The defaults for a computer with `host_cpus` CPUs and `host_memory` bytes of
    /// memory: half the CPUs (2 to 8), 4 GiB or a quarter of the memory if that is
    /// larger (at most 16 GiB), and a 64 GiB disk. See ADR 0008.
    pub fn recommended(host_cpus: u32, host_memory: u64) -> Self {
        Self {
            cpus: (host_cpus / 2).clamp(MIN_CPUS, MAX_DEFAULT_CPUS),
            memory_bytes: (host_memory / 4).clamp(MIN_MEMORY, MAX_DEFAULT_MEMORY),
            disk_bytes: DEFAULT_DISK,
        }
    }

    /// Adds `delta` CPUs, keeping 1 to `host_cpus`.
    pub fn step_cpus(self, delta: i32, host_cpus: u32) -> Self {
        let cpus = self.cpus.saturating_add_signed(delta);
        Self {
            cpus: cpus.clamp(1, host_cpus.max(1)),
            ..self
        }
    }

    /// Adds `delta` GiB of memory, keeping 2 GiB to three quarters of `host_memory`.
    pub fn step_memory(self, delta: i64, host_memory: u64) -> Self {
        let max = (host_memory / 4 * 3).max(MIN_USER_MEMORY);
        Self {
            memory_bytes: step(self.memory_bytes, delta).clamp(MIN_USER_MEMORY, max),
            ..self
        }
    }

    /// Adds `delta` GiB of disk, keeping 16 GiB to 1 TiB, and never below `current`,
    /// the size of the machine's disk now, because a disk cannot shrink.
    pub fn step_disk(self, delta: i64, current: Option<u64>) -> Self {
        let floor = current.map_or(MIN_DISK, |bytes| step(bytes, 0).clamp(MIN_DISK, MAX_DISK));
        Self {
            disk_bytes: step(self.disk_bytes, delta).clamp(floor, MAX_DISK),
            ..self
        }
    }

    /// What a restart would change in a machine that runs with `running`: its
    /// resources and these, each as "4 CPUs and 6.0 GB memory", or `None` when they
    /// match. Memory compares in MiB and the disk only when it grows, as Lima applies
    /// them.
    pub fn restart_change(&self, running: &HostResources) -> Option<(String, String)> {
        const MIB: u64 = 1024 * 1024;
        let mut before = Vec::new();
        let mut after = Vec::new();
        if self.cpus != running.cpus {
            before.push(format!("{} CPUs", running.cpus));
            after.push(format!("{} CPUs", self.cpus));
        }
        if self.memory_bytes / MIB != running.memory_bytes / MIB {
            before.push(format!("{} memory", bytes_label(running.memory_bytes)));
            after.push(format!("{} memory", bytes_label(self.memory_bytes)));
        }
        if self.disk_bytes / GIB > running.disk_bytes / GIB {
            before.push(format!("a {} disk", bytes_label(running.disk_bytes)));
            after.push(format!("a {} disk", bytes_label(self.disk_bytes)));
        }
        (!after.is_empty()).then(|| (and_list(&before), and_list(&after)))
    }

    /// "4 CPUs · 8 GB memory · 64 GB disk".
    pub fn summary(&self) -> String {
        format!(
            "{} CPUs · {} memory · {} disk",
            self.cpus,
            bytes_label(self.memory_bytes),
            bytes_label(self.disk_bytes)
        )
    }

    /// Memory in whole GiB, rounded down, at least 1.
    pub fn memory_gib(&self) -> u64 {
        (self.memory_bytes / GIB).max(1)
    }

    /// Disk in whole GiB, rounded down, at least 1.
    pub fn disk_gib(&self) -> u64 {
        (self.disk_bytes / GIB).max(1)
    }
}

/// "a", "a and b", or "a, b, and c".
fn and_list(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [one] => one.clone(),
        [first, second] => format!("{first} and {second}"),
        [rest @ .., last] => format!("{}, and {last}", rest.join(", ")),
    }
}

/// `bytes` plus `delta` whole GiB, rounded to a whole GiB.
fn step(bytes: u64, delta: i64) -> u64 {
    let gib = (bytes / GIB).saturating_add_signed(delta);
    gib.saturating_mul(GIB)
}

#[cfg(test)]
mod tests;

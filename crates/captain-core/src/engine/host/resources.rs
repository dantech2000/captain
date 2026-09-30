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

    /// Adds `delta` GiB of disk, keeping 16 GiB to 1 TiB.
    pub fn step_disk(self, delta: i64) -> Self {
        Self {
            disk_bytes: step(self.disk_bytes, delta).clamp(MIN_DISK, MAX_DISK),
            ..self
        }
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

/// `bytes` plus `delta` whole GiB, rounded to a whole GiB.
fn step(bytes: u64, delta: i64) -> u64 {
    let gib = (bytes / GIB).saturating_add_signed(delta);
    gib.saturating_mul(GIB)
}

#[cfg(test)]
mod tests;

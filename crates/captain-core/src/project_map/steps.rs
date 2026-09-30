const MIB: u64 = 1024 * 1024;

/// The memory limits the editor steps through: 64 MB to 16 GB, in doublings.
pub const MEMORY_STEPS: [u64; 9] = [
    64 * MIB,
    128 * MIB,
    256 * MIB,
    512 * MIB,
    1024 * MIB,
    2048 * MIB,
    4096 * MIB,
    8192 * MIB,
    16384 * MIB,
];

/// The next memory step above (`up`) or below `current`. A limit between steps
/// moves to the nearest step in that direction; the ends stay where they are.
pub fn memory_step(current: u64, up: bool) -> u64 {
    let next = if up {
        MEMORY_STEPS.iter().find(|step| **step > current)
    } else {
        MEMORY_STEPS.iter().rev().find(|step| **step < current)
    };
    next.copied().unwrap_or(current)
}

/// The limit to stage after an out-of-memory kill: twice the old one, at least 512 MB.
pub fn raised_memory(limit: u64) -> u64 {
    limit.saturating_mul(2).max(512 * MIB)
}

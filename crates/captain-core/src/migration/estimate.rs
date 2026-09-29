/// The copy speed the estimate assumes: 80 MB/s, typical for a copy between two
/// VMs on one Mac. Image export and volume tar streams both run near it.
pub const DEFAULT_THROUGHPUT: u64 = 80 * 1024 * 1024;

/// A rough size and time for a plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Estimate {
    pub bytes: u64,
    pub seconds: u64,
}

impl Estimate {
    /// The time to copy `bytes` at `throughput` bytes a second, rounded up. A zero
    /// throughput counts as one byte a second.
    pub fn new(bytes: u64, throughput: u64) -> Self {
        Self {
            bytes,
            seconds: bytes.div_ceil(throughput.max(1)),
        }
    }
}

/// A coarse duration, for example `under a minute`, `about 4 minutes`, or
/// `about 1 hour 20 minutes`.
pub fn duration_label(seconds: u64) -> String {
    let minutes = seconds.div_ceil(60);
    match minutes {
        0 | 1 if seconds < 60 => "under a minute".into(),
        m if m < 60 => format!("about {m} minute{}", plural(m)),
        m => {
            let (hours, rest) = (m / 60, m % 60);
            let hours_label = format!("about {hours} hour{}", plural(hours));
            if rest == 0 {
                hours_label
            } else {
                format!("{hours_label} {rest} minute{}", plural(rest))
            }
        }
    }
}

fn plural(n: u64) -> &'static str {
    if n == 1 { "" } else { "s" }
}

#[cfg(test)]
mod tests;

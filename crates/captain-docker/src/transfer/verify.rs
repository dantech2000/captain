//! The check after a volume copy: both sides must hold the same number of entries
//! and the same number of file bytes.

/// Prints the entry count of `/v`, then the total size of its files in bytes.
pub const MEASURE_SCRIPT: &str = "cd /v && find . | wc -l && find . -type f -exec stat -c %s {} + | awk '{s+=$1} END {printf \"%.0f\\n\", s}'";

/// What [`MEASURE_SCRIPT`] reports about one volume.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Measure {
    /// Files, folders, and links, including the root.
    pub entries: u64,
    /// The apparent size of the regular files.
    pub bytes: u64,
}

impl Measure {
    /// Reads the two numbers that [`MEASURE_SCRIPT`] prints.
    pub fn parse(output: &str) -> Option<Self> {
        let mut numbers = output
            .split_whitespace()
            .map(|word| word.parse::<f64>().ok().map(|n| n as u64));
        let entries = numbers.next()??;
        let bytes = numbers.next()??;
        Some(Self { entries, bytes })
    }

    /// Fails with a message when `copy` differs from `self`.
    pub fn check(&self, copy: &Measure) -> Result<(), String> {
        if self == copy {
            return Ok(());
        }
        Err(format!(
            "The copy does not match: the source has {} entries and {} bytes, the copy has {} entries and {} bytes.",
            self.entries, self.bytes, copy.entries, copy.bytes
        ))
    }
}

#[cfg(test)]
mod tests;

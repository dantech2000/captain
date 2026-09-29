use crate::format::bytes_label;

/// What a volume prune removed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VolumePrune {
    /// The names of the removed volumes.
    pub removed: Vec<String>,
    /// Disk space freed, in bytes.
    pub reclaimed_bytes: u64,
}

impl VolumePrune {
    /// For example `Removed 3 volumes · reclaimed 120 MB`, or `No unused volumes to remove`.
    pub fn summary(&self) -> String {
        let count = match self.removed.len() {
            0 => return "No unused volumes to remove".into(),
            1 => "Removed 1 volume".to_string(),
            n => format!("Removed {n} volumes"),
        };
        if self.reclaimed_bytes == 0 {
            count
        } else {
            format!("{count} · reclaimed {}", bytes_label(self.reclaimed_bytes))
        }
    }
}

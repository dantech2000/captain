/// The input of the free-space check: what the plan copies and what the target has.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiskCheck {
    /// Bytes the selected items copy.
    pub bytes: u64,
    /// Free bytes on the target engine's disk. `None` if Captain could not read it.
    pub free: Option<u64>,
}

/// The result of the free-space check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiskVerdict {
    /// Captain could not read the free space.
    Unknown,
    /// At least twice the copy size is free.
    Enough,
    /// The copy fits, but not twice. An image load briefly needs about twice its size.
    Tight,
    /// The copy does not fit.
    NotEnough,
}

impl DiskCheck {
    /// A copy briefly needs about twice its size (ADR 0009), so less than that is
    /// tight, and less than the size itself is not enough.
    pub fn verdict(&self) -> DiskVerdict {
        match self.free {
            None => DiskVerdict::Unknown,
            Some(free) if free >= self.bytes.saturating_mul(2) => DiskVerdict::Enough,
            Some(free) if free >= self.bytes => DiskVerdict::Tight,
            Some(_) => DiskVerdict::NotEnough,
        }
    }
}

#[cfg(test)]
mod tests;

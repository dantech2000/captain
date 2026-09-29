use crate::GIB;
use crate::format::bytes_label;

/// The free space a snapshot step needs for itself.
pub const STEP_SPACE: u64 = 2 * GIB;

/// Fails when `free` bytes are not enough. A clone needs only [`STEP_SPACE`]; a full
/// copy also needs the `allocated` bytes it copies. An unknown `free` passes.
pub fn check_space(free: Option<u64>, allocated: u64, clone: bool) -> Result<(), String> {
    let Some(free) = free else {
        return Ok(());
    };
    let needed = STEP_SPACE + if clone { 0 } else { allocated };
    if free < needed {
        return Err(format!(
            "The snapshot needs {} free, but only {} is free.",
            bytes_label(needed),
            bytes_label(free)
        ));
    }
    Ok(())
}

/// A warning when less space is free than the engine's disk uses. A clone costs
/// almost nothing at first, but each block the engine changes afterwards takes new
/// space, up to the whole disk. `None` when there is enough space, or when a value is
/// unknown.
pub fn space_warning(free: Option<u64>, engine_disk: Option<u64>) -> Option<String> {
    let (free, used) = (free?, engine_disk?);
    (free < used).then(|| {
        format!(
            "Only {} is free, and the engine's disk uses {}. The snapshot grows as the \
             engine changes files, so the disk may fill up.",
            bytes_label(free),
            bytes_label(used)
        )
    })
}

#[cfg(test)]
mod tests;

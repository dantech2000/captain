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

#[cfg(test)]
mod tests;

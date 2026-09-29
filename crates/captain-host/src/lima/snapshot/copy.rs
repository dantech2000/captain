//! File copies for snapshots: an APFS clone on macOS, a plain copy elsewhere.

use std::io;
use std::path::Path;

/// Copies `from` to `to`. On macOS `/bin/cp -c` clones it with `clonefile(2)`, and
/// falls back to a full copy when the volume cannot clone. The path is spelled out
/// because a GNU `cp` on `PATH` reads `-c` differently.
#[cfg(target_os = "macos")]
pub fn copy_file(from: &Path, to: &Path) -> io::Result<()> {
    let output = std::process::Command::new("/bin/cp")
        .arg("-c")
        .arg(from)
        .arg(to)
        .output()?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(io::Error::other(stderr.trim().to_string()))
}

#[cfg(not(target_os = "macos"))]
pub fn copy_file(from: &Path, to: &Path) -> io::Result<()> {
    std::fs::copy(from, to).map(drop)
}

/// The bytes `path` uses on disk. A sparse disk uses less than its length.
#[cfg(unix)]
pub fn allocated(path: &Path) -> u64 {
    use std::os::unix::fs::MetadataExt;
    std::fs::metadata(path).map_or(0, |meta| meta.blocks() * 512)
}

#[cfg(not(unix))]
pub fn allocated(path: &Path) -> u64 {
    std::fs::metadata(path).map_or(0, |meta| meta.len())
}

/// True if `a` and `b` are on the same volume, so a clone can work.
#[cfg(unix)]
pub fn same_volume(a: &Path, b: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    match (std::fs::metadata(a), std::fs::metadata(b)) {
        (Ok(a), Ok(b)) => a.dev() == b.dev(),
        _ => false,
    }
}

#[cfg(not(unix))]
pub fn same_volume(_: &Path, _: &Path) -> bool {
    false
}

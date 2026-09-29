//! Finds `limactl`. An app started from the Finder gets a short `PATH`, so the
//! Homebrew folders come after it.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

const BINARY: &str = "limactl";

/// Install folders to try after `PATH`, in order.
const SYSTEM_DIRS: &[&str] = &["/opt/homebrew/bin", "/usr/local/bin"];

/// The copy inside `Captain.app` for an executable at `exe`
/// (`Captain.app/Contents/MacOS/captain`): `Contents/Resources/lima/bin/limactl`.
pub fn bundled(exe: &Path) -> Option<PathBuf> {
    let contents = exe.parent()?.parent()?;
    Some(contents.join("Resources/lima/bin").join(BINARY))
}

/// The first `limactl`: the bundled copy, then `path` (a `PATH`-style list), then
/// the Homebrew folders. `exists` checks whether a file exists.
pub fn locate_limactl(
    exe: Option<&Path>,
    path: Option<&OsStr>,
    exists: impl Fn(&Path) -> bool,
) -> Option<PathBuf> {
    let bundled = exe.and_then(bundled);
    let from_path = path
        .into_iter()
        .flat_map(std::env::split_paths)
        .map(|dir| dir.join(BINARY));
    let system = SYSTEM_DIRS.iter().map(|dir| Path::new(dir).join(BINARY));
    bundled
        .into_iter()
        .chain(from_path)
        .chain(system)
        .find(|binary| exists(binary))
}

#[cfg(test)]
mod tests;

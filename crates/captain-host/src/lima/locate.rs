//! Finds `limactl`: the copy in `Captain.app`, then `PATH`, then the Homebrew
//! folders, because an app started from the Finder gets a short `PATH`.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use captain_core::tools::{Bundle, locate_tool};

/// Install folders to try after `PATH`, in order.
const SYSTEM_DIRS: &[&str] = &["/opt/homebrew/bin", "/usr/local/bin"];

/// The first `limactl` for Captain running from `exe`. `exists` checks whether a
/// file exists.
pub fn locate_limactl(
    exe: Option<&Path>,
    path: Option<&OsStr>,
    exists: impl Fn(&Path) -> bool,
) -> Option<PathBuf> {
    let bundled = exe
        .and_then(Bundle::from_exe)
        .map(|bundle| bundle.limactl());
    let system = SYSTEM_DIRS.iter().map(PathBuf::from);
    locate_tool("limactl", bundled, path, system, exists)
}

/// The running executable with links resolved, so a `captain` linked onto `PATH`
/// still finds the tools in its app bundle.
pub fn current_exe() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    Some(exe.canonicalize().unwrap_or(exe))
}

#[cfg(test)]
mod tests;

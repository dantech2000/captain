//! Finds the `docker` binary. An app started from the Finder or a desktop launcher
//! gets a short `PATH`, so known install folders come after `PATH`.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

#[cfg(windows)]
const BINARY: &str = "docker.exe";
#[cfg(not(windows))]
const BINARY: &str = "docker";

/// Install folders to try after `PATH`, in order.
#[cfg(windows)]
const SYSTEM_DIRS: &[&str] = &[r"C:\Program Files\Docker\Docker\resources\bin"];
#[cfg(not(windows))]
const SYSTEM_DIRS: &[&str] = &[
    "/usr/local/bin",
    "/opt/homebrew/bin",
    "/usr/bin",
    "/Applications/Docker.app/Contents/Resources/bin",
];

/// Install folders relative to the home directory, after the system ones.
const HOME_DIRS: &[&str] = &[".docker/bin", ".orbstack/bin", ".rd/bin"];

/// The first `docker` binary in `path` (a `PATH`-style list), then in the known
/// install folders. `exists` checks whether a file exists.
pub fn locate_docker(
    path: Option<&OsStr>,
    home: Option<&Path>,
    exists: impl Fn(&Path) -> bool,
) -> Option<PathBuf> {
    let from_path = path.into_iter().flat_map(std::env::split_paths);
    let system = SYSTEM_DIRS.iter().map(PathBuf::from);
    let in_home = home
        .into_iter()
        .flat_map(|home| HOME_DIRS.iter().map(move |dir| home.join(dir)));
    from_path
        .chain(system)
        .chain(in_home)
        .map(|dir| dir.join(BINARY))
        .find(|binary| exists(binary))
}

#[cfg(test)]
mod tests;

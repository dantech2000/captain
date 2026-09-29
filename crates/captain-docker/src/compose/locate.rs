//! Finds the `docker` binary and its helpers: the copy in `Captain.app`, then
//! `PATH`, then known install folders, because an app started from the Finder or a
//! desktop launcher gets a short `PATH`.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use captain_core::tools::{Bundle, locate_tool};

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

/// The first `docker` binary: the one in `bundle`, then `path` (a `PATH`-style
/// list), then the known install folders. `exists` checks whether a file exists.
pub fn locate_docker(
    bundle: Option<&Bundle>,
    path: Option<&OsStr>,
    home: Option<&Path>,
    exists: impl Fn(&Path) -> bool,
) -> Option<PathBuf> {
    let bundled = bundle.map(Bundle::docker);
    locate_tool(BINARY, bundled, path, install_dirs(home), exists)
}

/// The first binary `name` (with `.exe` on Windows) in `path`, then the folders
/// where `docker` gets installed, for example a credential helper. Captain.app does
/// not bundle helpers.
pub fn locate_helper(
    name: &str,
    path: Option<&OsStr>,
    home: Option<&Path>,
    exists: impl Fn(&Path) -> bool,
) -> Option<PathBuf> {
    let binary = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    };
    locate_tool(&binary, None, path, install_dirs(home), exists)
}

/// The known install folders: the system ones, then the ones in `home`.
fn install_dirs(home: Option<&Path>) -> impl Iterator<Item = PathBuf> {
    let system = SYSTEM_DIRS.iter().map(PathBuf::from);
    let in_home = home
        .map(Path::to_path_buf)
        .into_iter()
        .flat_map(|home| HOME_DIRS.iter().map(move |dir| home.join(dir)));
    system.chain(in_home)
}

#[cfg(test)]
mod tests;

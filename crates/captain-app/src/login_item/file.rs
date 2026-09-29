//! Writes and removes the login item file on macOS and Linux.

use std::path::{Path, PathBuf};
use std::{env, fs, io};

/// The running binary, which the login item starts.
pub fn program() -> io::Result<String> {
    Ok(env::current_exe()?.to_string_lossy().into_owned())
}

/// `dir/name` under `base`, or an error when the platform has no such folder.
pub fn in_dir(base: Option<PathBuf>, dir: &str, name: &str) -> io::Result<PathBuf> {
    base.map(|base| base.join(dir).join(name))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no home folder"))
}

pub fn write(path: &Path, text: &str) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(path, text)
}

/// Removes `path`. A file that is already gone is not an error.
pub fn remove(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        result => result,
    }
}

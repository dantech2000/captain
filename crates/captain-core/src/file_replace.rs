//! Replaces the text of a user's file safely: a new file next to it, synced to
//! disk, then renamed over it. A rename is atomic, so a crash or a full disk leaves
//! the old file or the new one, never half a file. Callers pass the file itself,
//! not a symlink to it (see [`crate::link_target`]).

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Who may read the new file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    /// The old file's permissions, or the usual ones for a new file.
    Keep,
    /// Only the owner (0600), for a file that can hold secrets.
    Private,
}

/// Replaces the file at `path` with `bytes`, and creates its folder if needed.
pub(crate) fn replace(path: &Path, bytes: &[u8], mode: Mode) -> io::Result<()> {
    let temp = temp_path(path);
    write_new(&temp, path, bytes, mode)?;
    commit(&temp, path)
}

/// Renames the new file `temp` over `path`. On an error, `temp` goes.
pub(crate) fn commit(temp: &Path, path: &Path) -> io::Result<()> {
    if let Err(error) = fs::rename(temp, path) {
        fs::remove_file(temp).ok();
        return Err(error);
    }
    // The rename is on disk once the folder is synced. Not every system can
    // sync a folder, and the file is whole either way.
    #[cfg(unix)]
    if let Some(dir) = path.parent().filter(|dir| !dir.as_os_str().is_empty()) {
        fs::File::open(dir).and_then(|dir| dir.sync_all()).ok();
    }
    Ok(())
}

/// Writes and syncs `bytes` into the new file `temp`, with the permissions that
/// `mode` gives for `path`, and creates the folder if needed. On an error, `temp`
/// goes.
pub(crate) fn write_new(temp: &Path, path: &Path, bytes: &[u8], mode: Mode) -> io::Result<()> {
    if let Some(dir) = temp.parent().filter(|dir| !dir.as_os_str().is_empty()) {
        fs::create_dir_all(dir)?;
    }
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    if mode == Mode::Private {
        std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    }
    let mut file = options.open(temp)?;
    let written = file.write_all(bytes).and_then(|()| {
        if mode == Mode::Keep
            && let Ok(meta) = fs::metadata(path)
        {
            file.set_permissions(meta.permissions())?;
        }
        file.sync_all()
    });
    if written.is_err() {
        fs::remove_file(temp).ok();
    }
    written
}

/// `.zshrc` becomes `..zshrc.<pid>-<n>.tmp` in the same folder, so the rename
/// stays on one file system and two writers never share a temporary file.
pub(crate) fn temp_path(path: &Path) -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let name = path
        .file_name()
        .map_or_else(|| "file".into(), |name| name.to_string_lossy());
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    path.with_file_name(format!(".{name}.{}-{n}.tmp", std::process::id()))
}

/// Copies the file at `path` to `backup`, unless `backup` exists or `path` does
/// not. Captain never overwrites a backup, so it keeps the file from before
/// Captain's first change.
pub(crate) fn backup_once(path: &Path, backup: &Path) -> io::Result<()> {
    if backup.exists() || !path.exists() {
        return Ok(());
    }
    fs::copy(path, backup).map(drop)
}

/// `.zshrc` becomes `.zshrc.<suffix>` in the same folder.
pub(crate) fn sibling(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".{suffix}"));
    path.with_file_name(name)
}

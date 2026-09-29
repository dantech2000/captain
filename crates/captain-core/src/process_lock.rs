//! A lock file that one process holds while it acts, so the app and the `captain`
//! CLI do not act at the same time. The operating system drops the lock when the
//! process ends, also after a crash. See docs/features/0022-command-line.md.

use std::fs::{File, OpenOptions, TryLockError};
use std::io::{self, Read, Seek, Write};
use std::path::{Path, PathBuf};

/// The lock file next to `settings.json` that the app holds while it runs.
pub fn app_lock_path(settings: &Path) -> PathBuf {
    settings.with_file_name("app.lock")
}

/// An exclusive lock on a file. Dropping it releases the lock.
#[derive(Debug)]
pub struct ProcessLock {
    _file: File,
}

impl ProcessLock {
    /// Takes the lock at `path` and writes `note` into the file, for others to read.
    /// `Ok(None)` when another process holds it.
    pub fn try_acquire(path: &Path, note: &str) -> io::Result<Option<Self>> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;
        match file.try_lock() {
            Ok(()) => {}
            Err(TryLockError::WouldBlock) => return Ok(None),
            Err(TryLockError::Error(error)) => return Err(error),
        }
        file.set_len(0)?;
        file.rewind()?;
        file.write_all(note.as_bytes())?;
        Ok(Some(Self { _file: file }))
    }

    /// The note of the process that holds the lock at `path`, or `None` when no
    /// process holds it. The note is empty where the file cannot be read while
    /// locked (Windows).
    pub fn holder(path: &Path) -> Option<String> {
        let mut file = File::open(path).ok()?;
        match file.try_lock_shared() {
            Ok(()) => None,
            Err(TryLockError::WouldBlock) => {
                let mut note = String::new();
                file.read_to_string(&mut note).ok();
                Some(note)
            }
            Err(TryLockError::Error(_)) => None,
        }
    }
}

#[cfg(test)]
mod tests;

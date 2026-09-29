//! The file side of an update: moves the installed files to a backup folder and the
//! new ones in, and can put everything back. See docs/features/0025-extensions.md.

use std::path::{Path, PathBuf};

use captain_core::EngineError;
use captain_core::extension::MANIFEST_FILE;

use super::files::io_error;

/// What an update replaces in the extension's folder. Everything else stays.
const REPLACED: [&str; 4] = ["ui", "bin", "compose", MANIFEST_FILE];

/// One update of an extension's folder. It records each move, so
/// [`Swap::roll_back`] undoes exactly what happened.
pub struct Swap {
    live: PathBuf,
    staged: PathBuf,
    backup: PathBuf,
    moved_out: Vec<&'static str>,
    /// Every old part is in the backup, so the parts in the folder are new.
    cleared: bool,
}

impl Swap {
    pub fn new(live: PathBuf, staged: PathBuf, backup: PathBuf) -> Self {
        Self {
            live,
            staged,
            backup,
            moved_out: Vec::new(),
            cleared: false,
        }
    }

    /// Moves the installed parts to the backup and the staged parts in, then writes
    /// `manifest` as the new `extension.json`.
    pub fn replace(&mut self, manifest: &str) -> Result<(), EngineError> {
        std::fs::create_dir_all(&self.backup).map_err(io_error)?;
        for part in REPLACED {
            let from = self.live.join(part);
            if from.exists() {
                std::fs::rename(&from, self.backup.join(part)).map_err(io_error)?;
                self.moved_out.push(part);
            }
        }
        self.cleared = true;
        for part in REPLACED {
            let from = self.staged.join(part);
            if from.exists() {
                std::fs::rename(&from, self.live.join(part)).map_err(io_error)?;
            }
        }
        std::fs::write(self.live.join(MANIFEST_FILE), manifest).map_err(io_error)
    }

    /// Removes the new parts, also those the new backend made, and moves the backup
    /// back. The backup stays when a step fails, so no file is lost. The old
    /// `extension.json` replaces the new one in one rename, so the folder always has
    /// one and the extension stays listed, with Remove.
    pub fn roll_back(&self) -> Result<(), EngineError> {
        for part in REPLACED {
            let live = self.live.join(part);
            let replaces = part == MANIFEST_FILE && self.moved_out.contains(&part);
            if self.cleared && !replaces {
                remove(&live)?;
            }
            if self.moved_out.contains(&part) {
                std::fs::rename(self.backup.join(part), &live).map_err(io_error)?;
            }
        }
        std::fs::remove_dir_all(&self.backup).ok();
        Ok(())
    }

    /// Deletes the backup, once the new version runs.
    pub fn commit(self) {
        std::fs::remove_dir_all(&self.backup).ok();
    }
}

fn remove(path: &Path) -> Result<(), EngineError> {
    let removed = if path.is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    };
    match removed {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(io_error(error)),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests;

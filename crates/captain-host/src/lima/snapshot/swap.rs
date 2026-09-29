//! The restore swap: stage every file first, then rename each one into place.
//! A rename is atomic, so a crash leaves each file old or new. The old files wait in
//! a backup folder until every rename worked, and go back if one fails.

use std::io;
use std::path::{Path, PathBuf};

const STAGING: &str = ".restore-staging";
const BACKUP: &str = ".restore-backup";

/// One file to restore: the copy in the snapshot and its place in the instance.
#[derive(Debug, Clone)]
pub struct Replace {
    pub from: PathBuf,
    pub live: PathBuf,
}

/// A file that the swap moved, to undo it.
struct Moved {
    live: PathBuf,
    backup: Option<PathBuf>,
}

/// Copies each `from` into a staging folder in `work` with `copy`, then renames it
/// over its `live` file. `work` must be on the same volume as every `live` file.
pub fn swap(
    files: &[Replace],
    work: &Path,
    copy: &dyn Fn(&Path, &Path) -> io::Result<()>,
) -> Result<(), String> {
    let staging = work.join(STAGING);
    let backup = work.join(BACKUP);
    for dir in [&staging, &backup] {
        std::fs::remove_dir_all(dir).ok();
        std::fs::create_dir_all(dir).map_err(|error| describe(dir, error))?;
    }
    let result = stage(files, &staging, copy).and_then(|staged| place(&staged, &backup));
    std::fs::remove_dir_all(&staging).ok();
    std::fs::remove_dir_all(&backup).ok();
    result
}

/// Copies each file into `staging`, named by its position.
fn stage(
    files: &[Replace],
    staging: &Path,
    copy: &dyn Fn(&Path, &Path) -> io::Result<()>,
) -> Result<Vec<(PathBuf, PathBuf)>, String> {
    files
        .iter()
        .enumerate()
        .map(|(ix, file)| {
            let staged = staging.join(ix.to_string());
            copy(&file.from, &staged).map_err(|error| describe(&file.from, error))?;
            Ok((staged, file.live.clone()))
        })
        .collect()
}

/// Renames each staged file over its live file, keeping the old one in `backup`.
fn place(staged: &[(PathBuf, PathBuf)], backup: &Path) -> Result<(), String> {
    let mut moved = Vec::new();
    for (ix, (from, live)) in staged.iter().enumerate() {
        if let Err(error) = place_one(from, live, &backup.join(ix.to_string()), &mut moved) {
            roll_back(moved);
            return Err(describe(live, error));
        }
    }
    Ok(())
}

fn place_one(from: &Path, live: &Path, backup: &Path, moved: &mut Vec<Moved>) -> io::Result<()> {
    if let Some(dir) = live.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let had_file = std::fs::symlink_metadata(live).is_ok();
    if had_file {
        std::fs::rename(live, backup)?;
    }
    let entry = Moved {
        live: live.to_path_buf(),
        backup: had_file.then(|| backup.to_path_buf()),
    };
    if let Err(error) = std::fs::rename(from, live) {
        roll_back(vec![entry]);
        return Err(error);
    }
    moved.push(entry);
    Ok(())
}

/// Puts the old files back, newest move first.
fn roll_back(moved: Vec<Moved>) {
    for entry in moved.into_iter().rev() {
        match entry.backup {
            Some(backup) => {
                if std::fs::symlink_metadata(&entry.live).is_ok_and(|meta| !meta.is_dir()) {
                    std::fs::remove_file(&entry.live).ok();
                }
                if let Err(error) = std::fs::rename(&backup, &entry.live) {
                    tracing::error!(%error, file = %entry.live.display(), "cannot put back");
                }
            }
            None => {
                std::fs::remove_file(&entry.live).ok();
            }
        }
    }
}

fn describe(path: &Path, error: io::Error) -> String {
    format!("Cannot restore {}: {error}", path.display())
}

#[cfg(test)]
mod tests;

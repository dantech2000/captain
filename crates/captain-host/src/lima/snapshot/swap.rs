//! The restore swap: stage every file first, then rename each one into place.
//! A rename is atomic, so a crash leaves each file old or new. The old files wait in
//! a backup folder until every rename worked, and go back if one fails. A backup
//! folder can hold the only copy of an old file, so the swap deletes it only after
//! the swap finished or every old file went back. An earlier swap's folder stops
//! the next swap and every engine start.

use std::io;
use std::path::{Path, PathBuf};

const STAGING: &str = ".restore-staging";
const BACKUP: &str = ".restore-backup";
/// In the backup folder: one `<backup name>\t<live path>` line per file.
const JOURNAL: &str = "journal.txt";

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

/// Fails while an earlier swap's backup folder is in `work`. Its live files can be
/// missing or mixed, and the folder can hold the only copy of an old file, so no
/// start and no other restore may run until the user puts the files back.
pub fn check_finished(work: &Path) -> Result<(), String> {
    let backup = work.join(BACKUP);
    if !backup.exists() {
        return Ok(());
    }
    Err(format!(
        "An earlier snapshot restore did not finish, so Captain Engine may be missing \
         files. The old files are in {}; {JOURNAL} there lists where each one belongs. \
         Move each file back, or move that folder away and restore a snapshot again.",
        backup.display()
    ))
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
    check_finished(work)?;
    std::fs::remove_dir_all(&staging).ok();
    for dir in [&staging, &backup] {
        std::fs::create_dir_all(dir).map_err(|error| describe(dir, error))?;
    }
    let result = stage(files, &staging, copy).and_then(|staged| place(&staged, &backup));
    std::fs::remove_dir_all(&staging).ok();
    if result.is_ok() || is_empty(&backup) {
        std::fs::remove_dir_all(&backup).ok();
    }
    result.map_err(|error| match backup.exists() {
        true => format!(
            "{error} Some old files could not go back. They are in {}; {JOURNAL} there \
             lists where each one belongs.",
            backup.display()
        ),
        false => error,
    })
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
/// The journal goes once every old file is back, so an empty `backup` is safe to
/// delete.
fn place(staged: &[(PathBuf, PathBuf)], backup: &Path) -> Result<(), String> {
    let journal: String = staged
        .iter()
        .enumerate()
        .map(|(ix, (_, live))| format!("{ix}\t{}\n", live.display()))
        .collect();
    let journal_file = backup.join(JOURNAL);
    if let Err(error) = std::fs::write(&journal_file, journal) {
        std::fs::remove_file(&journal_file).ok();
        return Err(describe(&journal_file, error));
    }
    let mut moved = Vec::new();
    for (ix, (from, live)) in staged.iter().enumerate() {
        if let Err(error) = place_one(from, live, &backup.join(ix.to_string()), &mut moved) {
            if roll_back(moved) {
                std::fs::remove_file(&journal_file).ok();
            }
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
    let renamed = std::fs::rename(from, live);
    moved.push(Moved {
        live: live.to_path_buf(),
        backup: had_file.then(|| backup.to_path_buf()),
    });
    renamed
}

/// Puts the old files back, newest move first. Returns false if one could not go
/// back and stays in the backup folder.
fn roll_back(moved: Vec<Moved>) -> bool {
    let mut restored = true;
    for entry in moved.into_iter().rev() {
        match entry.backup {
            Some(backup) => {
                if std::fs::symlink_metadata(&entry.live).is_ok_and(|meta| !meta.is_dir()) {
                    std::fs::remove_file(&entry.live).ok();
                }
                if let Err(error) = std::fs::rename(&backup, &entry.live) {
                    tracing::error!(%error, file = %entry.live.display(), "cannot put back");
                    restored = false;
                }
            }
            None => {
                std::fs::remove_file(&entry.live).ok();
            }
        }
    }
    restored
}

fn is_empty(dir: &Path) -> bool {
    std::fs::read_dir(dir).is_ok_and(|mut entries| entries.next().is_none())
}

fn describe(path: &Path, error: io::Error) -> String {
    format!("Cannot restore {}: {error}", path.display())
}

#[cfg(test)]
mod tests;

//! The snapshots folder: one folder per snapshot, named by a UUID, with
//! `metadata.json` and, once complete, `complete.txt`.

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::io;
use std::path::{Path, PathBuf};

use captain_core::snapshot::{
    COMPLETE_FILE, METADATA_FILE, Snapshot, SnapshotMetadata, newest_first,
};

/// The complete snapshots in `root`, newest first. Folders without `complete.txt`
/// or with unreadable metadata are skipped.
pub fn read_all(root: &Path) -> Vec<Snapshot> {
    let mut snapshots: Vec<Snapshot> = folders(root)
        .filter(|dir| dir.join(COMPLETE_FILE).is_file())
        .filter_map(|dir| {
            let json = std::fs::read_to_string(dir.join(METADATA_FILE)).ok()?;
            let metadata = SnapshotMetadata::from_json(&json).ok()?;
            let id = dir.file_name()?.to_string_lossy().into_owned();
            Some(Snapshot { id, metadata })
        })
        .collect();
    newest_first(&mut snapshots);
    snapshots
}

/// Deletes the folders that a failed or interrupted create left behind. Call it
/// only while holding the engine lock, so no create is running.
pub fn remove_incomplete(root: &Path) {
    for dir in folders(root).filter(|dir| !dir.join(COMPLETE_FILE).exists()) {
        tracing::info!(dir = %dir.display(), "deleting an incomplete snapshot");
        std::fs::remove_dir_all(&dir).ok();
    }
}

/// Writes `metadata.json`, then `complete.txt`.
pub fn finish(dir: &Path, metadata: &SnapshotMetadata) -> io::Result<()> {
    std::fs::write(dir.join(METADATA_FILE), metadata.to_json())?;
    std::fs::write(dir.join(COMPLETE_FILE), "")
}

/// A random version 4 UUID, from the standard library's random hash keys.
pub fn new_id() -> String {
    let random = || RandomState::new().build_hasher().finish();
    let bytes = ((random() as u128) << 64 | random() as u128).to_be_bytes();
    let mut bytes = bytes;
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    format!(
        "{}-{}-{}-{}-{}",
        &hex[..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..]
    )
}

fn folders(root: &Path) -> impl Iterator<Item = PathBuf> {
    std::fs::read_dir(root)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .filter(|entry| !entry.file_name().to_string_lossy().starts_with('.'))
        .map(|entry| entry.path())
}

#[cfg(test)]
mod tests;

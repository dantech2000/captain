//! Folder listings and file reads, from the listing script's output or from the
//! tar archives the engine's `/archive` endpoint returns.

use std::io::{self, Read};

use bollard::models::ContainerTopResponse;
use captain_core::model::{FileEntry, FilePreview, ProcessTable};
use tar::{Archive, EntryType};

/// The line between the `stat` lines and the names.
const STAT_SEPARATOR: &str = "---";

/// Entries from the listing script: `stat -c "%f %s %Y"` lines, the separator,
/// then one record per name in the same order: `1` if it opens as a folder (else
/// `0`), the name, and a NUL. `None` when the counts differ: the folder changed
/// between the two steps.
pub fn stat_listing(output: &str) -> Option<Vec<FileEntry>> {
    let (stats, names) = output
        .split_once(&format!("\n{STAT_SEPARATOR}\n"))
        .or_else(|| {
            output
                .strip_prefix(&format!("{STAT_SEPARATOR}\n"))
                .map(|rest| ("", rest))
        })?;
    let stats: Vec<&str> = stats.lines().collect();
    let names: Vec<&str> = names.split_terminator('\0').collect();
    if stats.len() != names.len() {
        return None;
    }
    stats
        .into_iter()
        .zip(names)
        .map(|(line, record)| {
            let mut parts = line.splitn(3, ' ');
            let mode = u32::from_str_radix(parts.next()?, 16).ok()?;
            let size = parts.next()?.parse().ok()?;
            let modified = parts.next()?.parse().ok()?;
            let (opens, name) = record.split_at_checked(1)?;
            let mut entry = FileEntry::new(name, mode, size, modified);
            entry.opens |= opens == "1";
            Some(entry)
        })
        .collect()
}

/// The direct children of the folder a tar holds. The first entry is the folder
/// itself, as `etc/` or `/`; its children are one level below it.
pub fn tar_listing(tar: &[u8]) -> io::Result<Vec<FileEntry>> {
    let mut archive = Archive::new(tar);
    let mut root_depth = None;
    let mut entries = Vec::new();
    for entry in archive.entries()? {
        let entry = entry?;
        let path = entry.path()?.to_string_lossy().into_owned();
        let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
        let depth = *root_depth.get_or_insert(parts.len());
        if parts.len() != depth + 1 {
            continue;
        }
        let header = entry.header();
        let mode = (header.mode()? & 0o7777) | type_bits(header.entry_type());
        let name = parts[depth].to_string();
        entries.push(FileEntry::new(
            name,
            mode,
            header.size()?,
            header.mtime()? as i64,
        ));
    }
    Ok(entries)
}

/// The start of the one file a tar holds: at most `limit` bytes, and its size.
/// The tar may end early, when the caller stopped reading the stream.
pub fn tar_preview(tar: &[u8], limit: u64) -> io::Result<FilePreview> {
    let mut archive = Archive::new(tar);
    let entry = archive
        .entries()?
        .next()
        .ok_or_else(|| io::Error::other("the archive is empty"))??;
    if entry.header().entry_type() != EntryType::Regular {
        return Err(io::Error::other("only a regular file has a preview"));
    }
    let size = entry.header().size()?;
    let mut bytes = Vec::new();
    entry.take(limit).read_to_end(&mut bytes)?;
    Ok(FilePreview { bytes, size })
}

pub fn processes(response: ContainerTopResponse) -> ProcessTable {
    ProcessTable {
        titles: response.titles.unwrap_or_default(),
        rows: response.processes.unwrap_or_default(),
    }
}

fn type_bits(kind: EntryType) -> u32 {
    match kind {
        EntryType::Directory => 0o040000,
        EntryType::Symlink => 0o120000,
        EntryType::Char => 0o020000,
        EntryType::Block => 0o060000,
        EntryType::Fifo => 0o010000,
        _ => 0o100000,
    }
}

#[cfg(test)]
mod tests;

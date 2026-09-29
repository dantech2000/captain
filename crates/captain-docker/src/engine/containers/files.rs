//! The engine side of the Files tab: list a folder, preview a file, and save a file
//! or folder to the host. See docs/features/0017-files-and-processes.md.

mod archive;
mod capture;

use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

use bollard::Docker;
use captain_core::EngineError;
use captain_core::model::{FileEntry, FilePreview, save_name};
use tar::{Archive, EntryType};

use self::archive::{Collected, io_error};
use crate::mapping;

/// Lists the folder `$1` with `stat`, which prints the same fields in GNU coreutils
/// and BusyBox. The second run follows links, to find the ones that point to
/// folders. Exit code 3 means the folder cannot be opened; 127 means no `stat`.
const LIST_SCRIPT: &str = r#"command -v stat >/dev/null || exit 127
cd -- "$1" || exit 3
set -- .[!.]* ..?* *
stat -c "%f %s %Y %n" -- "$@" 2>/dev/null
echo ---
stat -L -c "%f %n" -- "$@" 2>/dev/null
exit 0"#;

/// The most of a folder's tar the archive fallback reads.
const ARCHIVE_CAP: usize = 32 * 1024 * 1024;
/// Room for the tar headers in front of a previewed file.
const HEADER_ROOM: usize = 64 * 1024;

pub async fn list(docker: &Docker, id: &str, path: &str) -> Result<Vec<FileEntry>, EngineError> {
    let cmd = ["sh", "-c", LIST_SCRIPT, "sh", path]
        .map(String::from)
        .to_vec();
    match capture::capture(docker, id, cmd).await {
        Ok(out) if out.code == Some(0) => Ok(mapping::stat_listing(&out.stdout)),
        Ok(out) if out.code == Some(3) => Err(EngineError::Api(format!(
            "Could not open {path}. It does not exist, or it is not a folder."
        ))),
        Ok(out) if out.missing_command() => list_from_archive(docker, id, path).await,
        Ok(out) => Err(EngineError::Api(format!(
            "Could not list {path}: {}",
            out.stderr.trim()
        ))),
        Err(EngineError::Api(message)) if message.contains("executable file not found") => {
            list_from_archive(docker, id, path).await
        }
        Err(error) => Err(error),
    }
}

/// Lists a folder from its tar, for images with no shell. The tar holds the whole
/// folder, so this gives up past [`ARCHIVE_CAP`].
async fn list_from_archive(
    docker: &Docker,
    id: &str,
    path: &str,
) -> Result<Vec<FileEntry>, EngineError> {
    match archive::collect(docker, id, path, ARCHIVE_CAP).await? {
        Collected::All(tar) => mapping::tar_listing(&tar).map_err(io_error),
        Collected::Capped(_) => Err(EngineError::Api(
            "This folder is too large to list, and the container has no shell.".into(),
        )),
    }
}

pub async fn read(
    docker: &Docker,
    id: &str,
    path: &str,
    limit: u64,
) -> Result<FilePreview, EngineError> {
    let cap = usize::try_from(limit)
        .unwrap_or(usize::MAX)
        .saturating_add(HEADER_ROOM);
    let (Collected::All(tar) | Collected::Capped(tar)) =
        archive::collect(docker, id, path, cap).await?;
    mapping::tar_preview(&tar, limit).map_err(io_error)
}

/// Streams the tar to a hidden file in `dir`, then keeps a folder as `name.tar` or
/// unpacks a single file.
pub async fn save(
    docker: &Docker,
    id: &str,
    path: &str,
    dir: PathBuf,
) -> Result<PathBuf, EngineError> {
    let name = path
        .rsplit('/')
        .find(|part| !part.is_empty())
        .unwrap_or("root")
        .to_string();
    let part = dir.join(format!(".{name}.captain-download"));
    if let Err(error) = archive::write_to(docker, id, path, &part).await {
        fs::remove_file(&part).ok();
        return Err(error);
    }
    let result = tokio::task::spawn_blocking(move || {
        let saved = unpack(&part, &dir, &name);
        fs::remove_file(&part).ok();
        saved
    })
    .await
    .map_err(|error| EngineError::Api(error.to_string()))?;
    result.map_err(io_error)
}

fn unpack(tar: &Path, dir: &Path, name: &str) -> io::Result<PathBuf> {
    let free = |name: &str| dir.join(save_name(name, |n| dir.join(n).exists()));
    let kind = {
        let mut archive = Archive::new(File::open(tar)?);
        let mut entry = archive
            .entries()?
            .next()
            .ok_or_else(|| io::Error::other("the archive is empty"))??;
        let kind = entry.header().entry_type();
        if kind == EntryType::Regular {
            let target = free(name);
            io::copy(&mut entry, &mut File::create(&target)?)?;
            return Ok(target);
        }
        kind
    };
    if kind != EntryType::Directory {
        return Err(io::Error::other(
            "Captain can save only files and folders. Open the link's target instead.",
        ));
    }
    // The archive is closed, so Windows allows the rename.
    let target = free(&format!("{name}.tar"));
    fs::rename(tar, &target)?;
    Ok(target)
}

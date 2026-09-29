//! The engine side of the Files tab: list a folder, preview a file, and save a file
//! or folder to the host. See docs/features/0017-files-and-processes.md.

mod archive;
mod capture;

use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use bollard::Docker;
use captain_core::EngineError;
use captain_core::model::{FileEntry, FilePreview, host_file_name, save_name};
use tar::{Archive, EntryType};

use self::archive::{Collected, io_error};
use crate::mapping;

/// Lists the folder `$1`. `stat` prints the same fields in GNU coreutils and
/// BusyBox, but ends each name with a newline, which a name may hold. So `stat`
/// prints no names; the shell's `printf` prints them after it, each ending in a NUL
/// and led by `1` when it opens as a folder, following links. A pattern that
/// matches nothing stays as itself and does not exist, so both skip it. Exit code 3
/// means the folder cannot be opened; 127 means no `stat`.
const LIST_SCRIPT: &str = r#"command -v stat >/dev/null || exit 127
cd -- "$1" || exit 3
set -- .[!.]* ..?* *
stat -c "%f %s %Y" -- "$@" 2>/dev/null
echo ---
for f; do
  [ -e "$f" ] || [ -L "$f" ] || continue
  if [ -d "$f" ]; then d=1; else d=0; fi
  printf '%s%s\0' "$d" "$f"
done
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
        Ok(out) if out.code == Some(0) => mapping::stat_listing(&out.stdout).ok_or_else(|| {
            EngineError::Api(format!(
                "{path} changed while Captain listed it. Try again."
            ))
        }),
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

/// Streams the tar to a new hidden file in `dir`, then keeps a folder as `name.tar`
/// or unpacks a single file. The name comes from the container, so it passes
/// through [`host_file_name`] first.
pub async fn save(
    docker: &Docker,
    id: &str,
    path: &str,
    dir: PathBuf,
) -> Result<PathBuf, EngineError> {
    let name = host_file_name(
        path.rsplit('/')
            .find(|part| !part.is_empty())
            .unwrap_or("root"),
    );
    let part = dir.join(temp_name());
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

/// A hidden name that no other download uses: the process ID, the time, and a count.
fn temp_name() -> String {
    static COUNT: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let count = COUNT.fetch_add(1, Ordering::Relaxed);
    format!(".captain-download-{}-{nanos}-{count}", std::process::id())
}

fn unpack(tar: &Path, dir: &Path, name: &str) -> io::Result<PathBuf> {
    let kind = {
        let mut archive = Archive::new(File::open(tar)?);
        let mut entry = archive
            .entries()?
            .next()
            .ok_or_else(|| io::Error::other("the archive is empty"))??;
        let kind = entry.header().entry_type();
        if kind == EntryType::Regular {
            let (target, mut file) = reserve(dir, name)?;
            io::copy(&mut entry, &mut file)?;
            return Ok(target);
        }
        kind
    };
    if kind != EntryType::Directory {
        return Err(io::Error::other(
            "Captain can save only files and folders. Open the link's target instead.",
        ));
    }
    // The rename replaces only the empty file that `reserve` made. The archive and
    // that file are closed, so Windows allows it.
    let (target, file) = reserve(dir, &format!("{name}.tar"))?;
    drop(file);
    fs::rename(tar, &target).inspect_err(|_| {
        fs::remove_file(&target).ok();
    })?;
    Ok(target)
}

/// Creates a new, empty file with the first free name like `name` in `dir`. It
/// replaces nothing: not another save that picked the same name meanwhile, and not
/// a link that points nowhere.
fn reserve(dir: &Path, name: &str) -> io::Result<(PathBuf, File)> {
    let taken = |n: &str| dir.join(n).symlink_metadata().is_ok();
    for _ in 0..100 {
        let target = dir.join(save_name(name, taken));
        match File::create_new(&target) {
            Ok(file) => return Ok((target, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::other(format!("no free name for {name}")))
}

#[cfg(all(test, unix))]
mod tests;

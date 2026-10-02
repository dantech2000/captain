use std::fs;
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};

use crate::file_replace::temp_path;

/// One file of a new project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewFile {
    /// The path inside the project folder, with `/` between folders.
    pub path: String,
    pub text: String,
    /// Only the owner may read it (mode 0600 on Unix), for `.env` with passwords.
    pub private: bool,
}

impl NewFile {
    pub fn new(path: &str, text: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            text: text.into(),
            private: false,
        }
    }

    pub fn private(path: &str, text: impl Into<String>) -> Self {
        Self {
            private: true,
            ..Self::new(path, text)
        }
    }
}

/// Why a new project could not be written.
#[derive(Debug, thiserror::Error)]
pub enum NewProjectError {
    /// Captain never writes into a folder that has files, or through a link.
    #[error("{} already exists and is not empty. Pick another name.", .0.display())]
    NotEmpty(PathBuf),
    #[error("cannot write {}: {source}", path.display())]
    Io { path: PathBuf, source: io::Error },
}

/// True if `dir` is missing or an empty folder, so a new project may go there.
/// A link or a file at `dir` is not free. An error means Captain cannot tell,
/// for example when a folder above `dir` is a file.
pub fn folder_is_free(dir: &Path) -> io::Result<bool> {
    match fs::symlink_metadata(dir) {
        Ok(meta) if !meta.is_dir() => Ok(false),
        Ok(_) => Ok(fs::read_dir(dir)?.next().is_none()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(true),
        Err(error) => Err(error),
    }
}

/// Creates `dir` with `files`. The files go into a new hidden folder next to
/// `dir`, each created with `create_new`, and the folder is then renamed to
/// `dir`. The rename fails when `dir` has files, so nothing is overwritten, and
/// a failure leaves no half-made project.
pub fn write_project(dir: &Path, files: &[NewFile]) -> Result<(), NewProjectError> {
    let io_error = |path: &Path| {
        let path = path.to_path_buf();
        move |source| NewProjectError::Io { path, source }
    };
    if !folder_is_free(dir).map_err(io_error(dir))? {
        return Err(NewProjectError::NotEmpty(dir.to_path_buf()));
    }
    let parent = dir.parent().filter(|p| !p.as_os_str().is_empty());
    if let Some(parent) = parent {
        fs::create_dir_all(parent).map_err(io_error(parent))?;
    }
    let temp = temp_path(dir);
    fs::create_dir(&temp).map_err(io_error(&temp))?;
    let result = files
        .iter()
        .try_for_each(|file| write_new_file(&temp, file))
        .and_then(|()| move_into_place(&temp, dir));
    if result.is_err() {
        fs::remove_dir_all(&temp).ok();
    }
    #[cfg(unix)]
    if let Some(parent) = parent {
        fs::File::open(parent).and_then(|p| p.sync_all()).ok();
    }
    result
}

/// Writes `file` under the new folder `root`. Its folders must be new or real
/// folders, never links, and the file itself must be new.
fn write_new_file(root: &Path, file: &NewFile) -> Result<(), NewProjectError> {
    let relative = Path::new(&file.path);
    let path = root.join(relative);
    let io_error = |source| NewProjectError::Io {
        path: path.clone(),
        source,
    };
    if !relative
        .components()
        .all(|part| matches!(part, Component::Normal(_)))
    {
        let error = io::Error::new(io::ErrorKind::InvalidInput, "not a path inside the project");
        return Err(io_error(error));
    }
    let mut folder = root.to_path_buf();
    for part in relative.parent().into_iter().flat_map(Path::components) {
        folder.push(part);
        match fs::create_dir(&folder) {
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                if !fs::symlink_metadata(&folder).map_err(io_error)?.is_dir() {
                    return Err(NewProjectError::NotEmpty(folder));
                }
            }
            other => other.map_err(io_error)?,
        }
    }
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    if file.private {
        std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    }
    let mut out = options.open(&path).map_err(io_error)?;
    out.write_all(file.text.as_bytes())
        .and_then(|()| out.sync_all())
        .map_err(io_error)
}

/// Renames the folder `temp` to `dir`. On Unix the rename replaces an empty
/// folder and fails on one with files. Windows never replaces a folder, so an
/// empty `dir` is removed first; `remove_dir` fails if files appeared.
fn move_into_place(temp: &Path, dir: &Path) -> Result<(), NewProjectError> {
    let first = fs::rename(temp, dir);
    let Err(error) = first else {
        return Ok(());
    };
    if folder_is_free(dir).unwrap_or(false)
        && fs::symlink_metadata(dir).is_ok_and(|meta| meta.is_dir())
        && fs::remove_dir(dir).is_ok()
        && fs::rename(temp, dir).is_ok()
    {
        return Ok(());
    }
    match fs::symlink_metadata(dir) {
        Ok(_) => Err(NewProjectError::NotEmpty(dir.to_path_buf())),
        Err(_) => Err(NewProjectError::Io {
            path: dir.to_path_buf(),
            source: error,
        }),
    }
}

#[cfg(test)]
mod tests;

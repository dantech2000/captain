use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::file_replace::{Mode, replace};

/// One file of a new project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewFile {
    /// The path inside the project folder, with `/` between folders.
    pub path: String,
    pub text: String,
    /// Only the owner may read it (mode 0600), for `.env` with passwords.
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
    /// Captain never writes into a folder that has files.
    #[error("{} already exists and is not empty. Pick another name.", .0.display())]
    NotEmpty(PathBuf),
    #[error("cannot write {}: {source}", path.display())]
    Io { path: PathBuf, source: io::Error },
}

/// True if `dir` is missing or empty, so a new project may go there.
pub fn folder_is_free(dir: &Path) -> bool {
    match fs::read_dir(dir) {
        Ok(mut entries) => entries.next().is_none(),
        Err(error) => error.kind() == io::ErrorKind::NotFound,
    }
}

/// Creates `dir` with `files`. A folder that has files is refused, so nothing is
/// overwritten. Each file is written through a temporary file and a rename.
pub fn write_project(dir: &Path, files: &[NewFile]) -> Result<(), NewProjectError> {
    if !folder_is_free(dir) {
        return Err(NewProjectError::NotEmpty(dir.to_path_buf()));
    }
    for file in files {
        let path = dir.join(&file.path);
        let mode = if file.private {
            Mode::Private
        } else {
            Mode::Keep
        };
        replace(&path, file.text.as_bytes(), mode)
            .map_err(|source| NewProjectError::Io { path, source })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;

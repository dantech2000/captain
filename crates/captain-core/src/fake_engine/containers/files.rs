//! The fake engine's container files: folders by path and file contents by path.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::EngineError;
use crate::model::{FileEntry, FilePreview, save_name};

#[derive(Debug, Clone, Default)]
pub struct FakeFiles {
    /// The entries of each folder, by absolute path.
    pub folders: BTreeMap<String, Vec<FileEntry>>,
    /// The contents of each file, by absolute path.
    pub contents: BTreeMap<String, Vec<u8>>,
}

impl FakeFiles {
    pub(super) fn list(&self, path: &str) -> Result<Vec<FileEntry>, EngineError> {
        self.folders.get(path).cloned().ok_or_else(|| missing(path))
    }

    pub(super) fn read(&self, path: &str, limit: u64) -> Result<FilePreview, EngineError> {
        let bytes = self.contents.get(path).ok_or_else(|| missing(path))?;
        let end = bytes
            .len()
            .min(usize::try_from(limit).unwrap_or(usize::MAX));
        Ok(FilePreview {
            bytes: bytes[..end].to_vec(),
            size: bytes.len() as u64,
        })
    }

    /// Writes the file's contents into `dir` under a free name.
    pub(super) fn save(&self, path: &str, dir: &Path) -> Result<PathBuf, EngineError> {
        let bytes = self.contents.get(path).ok_or_else(|| missing(path))?;
        let name = path.rsplit('/').next().unwrap_or(path);
        let target = dir.join(save_name(name, |name| dir.join(name).exists()));
        std::fs::write(&target, bytes).map_err(|error| EngineError::Api(error.to_string()))?;
        Ok(target)
    }
}

fn missing(path: &str) -> EngineError {
    EngineError::Api(format!("Could not find the file {path} in container"))
}

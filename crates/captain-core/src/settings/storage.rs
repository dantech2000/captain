use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use super::Settings;

/// Why the settings file could not be read or written.
#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("cannot access {path}: {source}")]
    Io { path: PathBuf, source: io::Error },
    #[error("{path} is not valid JSON: {source}")]
    Parse {
        path: PathBuf,
        source: serde_json::Error,
    },
}

impl Settings {
    /// Reads the settings at `path`. A missing file gives the defaults.
    pub fn load(path: &Path) -> Result<Self, SettingsError> {
        let json = match fs::read_to_string(path) {
            Ok(json) => json,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(source) => return Err(io_error(path, source)),
        };
        Self::from_json(&json).map_err(|source| SettingsError::Parse {
            path: path.to_path_buf(),
            source,
        })
    }

    /// Writes the settings to `path`, creating its directory if needed. It writes a
    /// temporary file next to `path` and renames it, so a crash never leaves half a file.
    pub fn save(&self, path: &Path) -> Result<(), SettingsError> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|source| io_error(dir, source))?;
        }
        let temp = temp_path(path);
        let written =
            write_synced(&temp, self.to_json().as_bytes()).and_then(|()| fs::rename(&temp, path));
        if let Err(source) = written {
            let _ = fs::remove_file(&temp);
            return Err(io_error(path, source));
        }
        Ok(())
    }
}

fn write_synced(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = fs::File::create(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

/// `settings.json` becomes `.settings.json.tmp` in the same directory, so the rename
/// stays on one file system.
fn temp_path(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .map_or_else(|| "settings".into(), |name| name.to_string_lossy());
    path.with_file_name(format!(".{name}.tmp"))
}

fn io_error(path: &Path, source: io::Error) -> SettingsError {
    SettingsError::Io {
        path: path.to_path_buf(),
        source,
    }
}

#[cfg(test)]
mod tests;

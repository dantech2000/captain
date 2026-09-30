use std::fmt;
use std::io;
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::file_replace::{Mode, replace};
use crate::link_target::link_target;

/// The SHA-256 of a file's bytes. Two versions differ when the text on disk
/// changed, whatever the file's times say.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextVersion([u8; 32]);

impl TextVersion {
    pub fn of(bytes: &[u8]) -> Self {
        Self(Sha256::digest(bytes).into())
    }
}

/// A file's text and the version it was read at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedText {
    pub text: String,
    pub version: TextVersion,
}

/// Why [`save_text`] did not write.
#[derive(Debug)]
pub enum SaveError {
    /// The file on disk is not the version the editor loaded. Captain never
    /// overwrites such a change without asking.
    Changed,
    Io(io::Error),
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Changed => f.write_str("the file changed on disk since Captain read it"),
            Self::Io(error) => write!(f, "{error}"),
        }
    }
}

/// Reads the text of the file at `path`, through a symlink. A file that is not
/// UTF-8 fails with `InvalidData`.
pub fn read_text(path: &Path) -> io::Result<LoadedText> {
    let bytes = std::fs::read(path)?;
    let version = TextVersion::of(&bytes);
    let text = String::from_utf8(bytes)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "the file is not UTF-8 text"))?;
    Ok(LoadedText { text, version })
}

/// The version of the file at `path` now, or `None` when it cannot be read, for
/// example after it was deleted.
pub fn disk_version(path: &Path) -> Option<TextVersion> {
    std::fs::read(path)
        .ok()
        .map(|bytes| TextVersion::of(&bytes))
}

/// Writes `text` over the file at `path` if the file is still at `loaded`, and
/// returns the new version. A symlink keeps pointing at its target, which gets
/// the text; the file keeps its permissions. See [`crate::file_replace`].
pub fn save_text(path: &Path, text: &str, loaded: &TextVersion) -> Result<TextVersion, SaveError> {
    let target = link_target(path);
    if disk_version(&target).as_ref() != Some(loaded) {
        return Err(SaveError::Changed);
    }
    replace(&target, text.as_bytes(), Mode::Keep).map_err(SaveError::Io)?;
    Ok(TextVersion::of(text.as_bytes()))
}

#[cfg(all(test, unix))]
mod tests;

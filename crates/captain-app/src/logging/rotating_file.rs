//! A log file with a size limit. Captain needs no crate for it: `tracing-appender`
//! rotates only by time.

use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;

/// A file grows to this size before it rotates. With one old file kept, the logs
/// use at most twice this.
const MAX_BYTES: u64 = 10 * 1024 * 1024;

/// Appends to `captain.log`. When a write would pass the limit, the file becomes
/// `captain.log.1`, replacing the older one, and a new file starts.
pub struct RotatingFile {
    path: PathBuf,
    file: File,
    written: u64,
    max: u64,
}

impl RotatingFile {
    pub fn open(path: PathBuf) -> io::Result<Self> {
        Self::with_limit(path, MAX_BYTES)
    }

    pub fn with_limit(path: PathBuf, max: u64) -> io::Result<Self> {
        let file = append(&path)?;
        let written = file.metadata()?.len();
        Ok(Self {
            path,
            file,
            written,
            max,
        })
    }

    /// `captain.log.1` next to the file.
    pub fn old_path(&self) -> PathBuf {
        let mut name = self.path.clone().into_os_string();
        name.push(".1");
        PathBuf::from(name)
    }

    fn rotate(&mut self) -> io::Result<()> {
        self.file.flush()?;
        std::fs::rename(&self.path, self.old_path())?;
        self.file = append(&self.path)?;
        self.written = 0;
        Ok(())
    }
}

fn append(path: &PathBuf) -> io::Result<File> {
    OpenOptions::new().create(true).append(true).open(path)
}

impl Write for RotatingFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.written > 0 && self.written + buf.len() as u64 > self.max {
            self.rotate()?;
        }
        let written = self.file.write(buf)?;
        self.written += written as u64;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

#[cfg(test)]
mod tests;

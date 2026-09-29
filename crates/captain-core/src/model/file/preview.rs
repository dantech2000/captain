/// The most bytes of a file the preview reads.
pub const PREVIEW_LIMIT: u64 = 256 * 1024;

/// The start of a file, for the read-only preview.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FilePreview {
    /// At most the limit the caller asked for.
    pub bytes: Vec<u8>,
    /// The full size of the file.
    pub size: u64,
}

impl FilePreview {
    /// True if the file is longer than `bytes`.
    pub fn is_truncated(&self) -> bool {
        (self.bytes.len() as u64) < self.size
    }

    /// The text, or `None` for a binary file: one with a NUL byte, or that is not
    /// UTF-8. A character cut at the end of a truncated preview does not count.
    pub fn text(&self) -> Option<&str> {
        if self.bytes.contains(&0) {
            return None;
        }
        match std::str::from_utf8(&self.bytes) {
            Ok(text) => Some(text),
            Err(error) if self.is_truncated() && error.error_len().is_none() => {
                std::str::from_utf8(&self.bytes[..error.valid_up_to()]).ok()
            }
            Err(_) => None,
        }
    }
}

#[cfg(test)]
mod tests;

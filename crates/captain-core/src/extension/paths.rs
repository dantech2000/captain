//! Where Captain keeps installed extensions, and how the `captain-ext://` scheme maps
//! a URL to a UI file.

use std::path::{Path, PathBuf};

/// The file in each extension folder that records the installed extension.
pub const MANIFEST_FILE: &str = "extension.json";

/// `~/.captain/extensions`, with one folder per extension ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionPaths {
    root: PathBuf,
}

impl ExtensionPaths {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// `<home>/.captain/extensions`.
    pub fn in_home(home: &Path) -> Self {
        Self::new(home.join(".captain").join("extensions"))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn dir(&self, id: &str) -> PathBuf {
        self.root.join(id)
    }

    /// The UI files, copied from the image's `ui.dashboard-tab.root`.
    pub fn ui_dir(&self, id: &str) -> PathBuf {
        self.dir(id).join("ui")
    }

    /// The host binaries for this platform.
    pub fn bin_dir(&self, id: &str) -> PathBuf {
        self.dir(id).join("bin")
    }

    /// The Compose file Captain runs for the backend, and the files next to it.
    pub fn compose_dir(&self, id: &str) -> PathBuf {
        self.dir(id).join("compose")
    }

    pub fn manifest(&self, id: &str) -> PathBuf {
        self.dir(id).join(MANIFEST_FILE)
    }

    /// The old files of an update that has not finished: `.backup/<id>`.
    pub fn backup_dir(&self, id: &str) -> PathBuf {
        self.root.join(".backup").join(id)
    }

    /// Where an update copies the new files first: `.update/<id>`.
    pub fn staging(&self) -> Self {
        Self::new(self.root.join(".update"))
    }
}

/// The file in `ui_dir` for the URL path `path`, for example `/assets/app.js`.
/// Percent escapes are decoded. `None` for a path that tries to leave `ui_dir`.
/// `/` means `index.html`.
pub fn ui_file(ui_dir: &Path, path: &str) -> Option<PathBuf> {
    let decoded = percent_decode(path)?;
    let mut file = ui_dir.to_path_buf();
    let mut parts = 0;
    for part in decoded
        .split('/')
        .filter(|part| !part.is_empty() && *part != ".")
    {
        if part == ".." || part.contains('\\') || part.contains(':') {
            return None;
        }
        file.push(part);
        parts += 1;
    }
    if parts == 0 {
        file.push("index.html");
    }
    Some(file)
}

/// `%XX` escapes decoded as UTF-8. `None` for a broken escape.
fn percent_decode(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = std::str::from_utf8(bytes.get(i + 1..i + 3)?).ok()?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

/// The `Content-Type` for a UI file, from its extension.
pub fn mime_type(file: &Path) -> &'static str {
    let extension = file
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    match extension.as_deref() {
        Some("html" | "htm") => "text/html; charset=utf-8",
        Some("js" | "mjs") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json" | "map") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("wasm") => "application/wasm",
        Some("txt") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests;

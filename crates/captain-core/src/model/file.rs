//! Files inside a container: folder entries, paths, previews, and saved names.

mod host_name;
mod path;
mod preview;
mod save_name;

pub use host_name::host_file_name;
pub use path::{Crumb, breadcrumbs, join_path, parent_path};
pub use preview::{FilePreview, PREVIEW_LIMIT};
pub use save_name::save_name;

/// The type bits of a Unix mode.
const TYPE_MASK: u32 = 0o170000;
const FOLDER: u32 = 0o040000;
const REGULAR: u32 = 0o100000;
const LINK: u32 = 0o120000;

/// What an entry in a folder is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    Folder,
    File,
    Link,
    /// A device, pipe, or socket.
    Other,
}

/// One entry of a folder in a container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    pub name: String,
    pub kind: FileKind,
    /// True for a folder, and for a link that points to a folder.
    pub opens: bool,
    pub size: u64,
    /// The Unix mode: type and permission bits, as `stat` reports them.
    pub mode: u32,
    /// The modified time, in seconds since the Unix epoch.
    pub modified: i64,
}

impl FileEntry {
    /// An entry whose kind comes from the type bits of `mode`.
    pub fn new(name: impl Into<String>, mode: u32, size: u64, modified: i64) -> Self {
        let kind = match mode & TYPE_MASK {
            FOLDER => FileKind::Folder,
            REGULAR => FileKind::File,
            LINK => FileKind::Link,
            _ => FileKind::Other,
        };
        Self {
            name: name.into(),
            kind,
            opens: kind == FileKind::Folder,
            size,
            mode,
            modified,
        }
    }

    /// The mode as `ls -l` prints it, for example `drwxr-xr-x`.
    pub fn mode_label(&self) -> String {
        let kind = match self.mode & TYPE_MASK {
            FOLDER => 'd',
            LINK => 'l',
            0o020000 => 'c',
            0o060000 => 'b',
            0o010000 => 'p',
            0o140000 => 's',
            _ => '-',
        };
        let mut label = String::with_capacity(10);
        label.push(kind);
        // Owner, group, other; each with its special bit: setuid, setgid, sticky.
        for (shift, special, mark) in [(6, 0o4000, 's'), (3, 0o2000, 's'), (0, 0o1000, 't')] {
            let bits = (self.mode >> shift) & 0o7;
            label.push(if bits & 4 != 0 { 'r' } else { '-' });
            label.push(if bits & 2 != 0 { 'w' } else { '-' });
            label.push(match (bits & 1 != 0, self.mode & special != 0) {
                (true, true) => mark,
                (false, true) => mark.to_ascii_uppercase(),
                (true, false) => 'x',
                (false, false) => '-',
            });
        }
        label
    }
}

/// Sorts folders (and links to folders) first, then by name.
pub fn sort_entries(entries: &mut [FileEntry]) {
    entries.sort_by(|a, b| b.opens.cmp(&a.opens).then_with(|| a.name.cmp(&b.name)));
}

#[cfg(test)]
mod tests;

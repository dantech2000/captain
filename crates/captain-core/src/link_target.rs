//! [`link_target`]: the file a write should replace. A write that renames a new
//! file over a symlink replaces the link, and the user's dotfiles (chezmoi, for
//! example) often link `~/.kube/config`, so Captain writes to the target.

use std::path::{Path, PathBuf};

/// How many links in a row Captain follows, like the kernel's limit.
const MAX_LINKS: usize = 40;

/// `path` with its symlinks followed, also to a target that does not exist yet.
/// A path that is not a link comes back as it is.
pub fn link_target(path: &Path) -> PathBuf {
    let mut current = path.to_path_buf();
    for _ in 0..MAX_LINKS {
        let Ok(next) = std::fs::read_link(&current) else {
            return current;
        };
        current = match current.parent() {
            Some(dir) if next.is_relative() => dir.join(next),
            _ => next,
        };
    }
    current
}

#[cfg(all(test, unix))]
mod tests;

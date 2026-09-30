//! The links in `~/.captain/bin` and `~/.captain/cli-plugins` into `Captain.app`.
//! The folders stay put, so moving the app only needs new links.

use std::io;
use std::path::{Path, PathBuf};

use super::ToolPaths;
use crate::tools::Bundle;

/// The names Captain links into `~/.captain/bin`, and removes on uninstall.
const BIN_NAMES: [&str; 4] = [
    "docker",
    "docker-compose",
    "docker-credential-osxkeychain",
    "captain",
];

/// The names Captain links into `~/.captain/cli-plugins`.
const PLUGIN_NAMES: [&str; 2] = ["docker-compose", "docker-buildx"];

/// One link Captain keeps: `path` points at `target` in the bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolLink {
    pub path: PathBuf,
    pub target: PathBuf,
}

/// What is at a link's path now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkState {
    /// A symlink to the target, which exists.
    Linked,
    /// Nothing is there.
    Missing,
    /// A symlink to somewhere else, or to a target that is gone. The old target.
    Stale(PathBuf),
    /// A file or folder that is not a symlink. Captain never replaces it.
    NotALink,
    /// The bundle has no such tool, for example a build without the tools.
    NoTarget,
}

impl LinkState {
    /// True if Captain creates or replaces the link.
    pub fn needs_link(&self) -> bool {
        matches!(self, LinkState::Missing | LinkState::Stale(_))
    }
}

/// A link and what Captain found or did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkReport {
    pub link: ToolLink,
    /// The state before a [`relink`].
    pub state: LinkState,
    pub error: Option<String>,
}

/// The links for `bundle`. The credential helper is only for macOS.
pub fn tool_links(bundle: &Bundle, paths: &ToolPaths) -> Vec<ToolLink> {
    let plugins = bundle.cli_plugins();
    let mut links = vec![
        (paths.bin.join("docker"), bundle.docker()),
        (
            paths.bin.join("docker-compose"),
            plugins.join("docker-compose"),
        ),
    ];
    if cfg!(target_os = "macos") {
        links.push((
            paths.bin.join("docker-credential-osxkeychain"),
            bundle.credential_helper(),
        ));
    }
    links.push((paths.bin.join("captain"), bundle.captain_cli()));
    links.extend(PLUGIN_NAMES.map(|name| (paths.plugins.join(name), plugins.join(name))));
    links
        .into_iter()
        .map(|(path, target)| ToolLink { path, target })
        .collect()
}

/// What is at `link.path` now. It reads the link, so it is cheap.
pub fn link_state(link: &ToolLink) -> LinkState {
    let Ok(meta) = std::fs::symlink_metadata(&link.path) else {
        return match link.target.exists() {
            true => LinkState::Missing,
            false => LinkState::NoTarget,
        };
    };
    if !meta.file_type().is_symlink() {
        return LinkState::NotALink;
    }
    let old = std::fs::read_link(&link.path).unwrap_or_default();
    match (old == link.target, link.target.exists()) {
        (_, false) => LinkState::NoTarget,
        (true, true) => LinkState::Linked,
        (false, true) => LinkState::Stale(old),
    }
}

/// Creates each missing link and replaces each stale one. Other paths stay as they are.
pub fn relink(links: &[ToolLink]) -> Vec<LinkReport> {
    links
        .iter()
        .map(|link| {
            let state = link_state(link);
            let error = state
                .needs_link()
                .then(|| make_link(link, &state).err())
                .flatten()
                .map(|error| format!("Cannot link {}: {error}", link.path.display()));
            LinkReport {
                link: link.clone(),
                state,
                error,
            }
        })
        .collect()
}

fn make_link(link: &ToolLink, state: &LinkState) -> io::Result<()> {
    if let Some(dir) = link.path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    if matches!(state, LinkState::Stale(_)) {
        std::fs::remove_file(&link.path)?;
    }
    symlink(&link.target, &link.path)
}

/// Removes Captain's links from `~/.captain/bin` and `~/.captain/cli-plugins`. A
/// path that is not a symlink stays. Returns the errors.
pub fn remove_links(paths: &ToolPaths) -> Vec<String> {
    let bin = BIN_NAMES.map(|name| paths.bin.join(name));
    let plugins = PLUGIN_NAMES.map(|name| paths.plugins.join(name));
    bin.iter()
        .chain(&plugins)
        .filter(|path| is_symlink(path))
        .filter_map(|path| {
            std::fs::remove_file(path)
                .err()
                .map(|error| format!("Cannot remove {}: {error}", path.display()))
        })
        .collect()
}

fn is_symlink(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok_and(|meta| meta.file_type().is_symlink())
}

#[cfg(unix)]
fn symlink(target: &Path, path: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(target, path)
}

#[cfg(not(unix))]
fn symlink(_: &Path, _: &Path) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        super::UNSUPPORTED,
    ))
}

#[cfg(all(test, unix))]
mod tests;

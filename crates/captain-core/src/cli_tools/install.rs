//! Install, uninstall, and status: the links, the plugin folder, and the shell
//! files together.

use std::path::Path;

use super::chezmoi::chezmoi_manages;
use super::links::{LinkReport, link_state, relink, remove_links, tool_links};
use super::plugin_config::{add_plugin_dir, has_plugin_dir, remove_plugin_dir};
use super::rc_files::{
    RcAccess, RcFile, RcState, add_block, rc_access, rc_files, rc_state, remove_block,
};
use super::{PathMode, ToolPaths};
use crate::tools::Bundle;

/// A shell file, what it has, and whether Captain may write it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RcStatus {
    pub file: RcFile,
    pub state: RcState,
    pub access: RcAccess,
}

impl RcStatus {
    /// True if the user must add the line: Manual mode, or a file Captain skips.
    pub fn needs_user(&self, mode: PathMode) -> bool {
        self.state == RcState::Missing
            && (mode == PathMode::Manual || self.access != RcAccess::Writable)
    }
}

/// Everything the card and `captain tools status` show, except the shell lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolsStatus {
    /// Empty without a bundle.
    pub links: Vec<LinkReport>,
    /// True if the docker `config.json` lists `~/.captain/cli-plugins`.
    pub plugins: bool,
    pub rc: Vec<RcStatus>,
}

/// Reads the state without changing anything. It may ask chezmoi about each file.
pub fn status(bundle: Option<&Bundle>, paths: &ToolPaths, shell: Option<&Path>) -> ToolsStatus {
    let links = bundle
        .map(|bundle| tool_links(bundle, paths))
        .unwrap_or_default()
        .into_iter()
        .map(|link| LinkReport {
            state: link_state(&link),
            link,
            error: None,
        })
        .collect();
    let rc = rc_files(&paths.home, shell)
        .into_iter()
        .map(|file| RcStatus {
            state: rc_state(&file.path),
            access: rc_access(&file.path, |path| chezmoi_manages(&paths.home, path)),
            file,
        })
        .collect();
    ToolsStatus {
        links,
        plugins: has_plugin_dir(&paths.docker_config, &paths.plugins),
        rc,
    }
}

/// Makes the links, adds the plugin folder, and, in Automatic, adds the block to
/// each shell file Captain may write (Manual removes the blocks). Each step runs
/// even when one before it fails. Returns the errors.
pub fn install(
    bundle: &Bundle,
    paths: &ToolPaths,
    shell: Option<&Path>,
    mode: PathMode,
) -> Vec<String> {
    let mut errors: Vec<String> = relink(&tool_links(bundle, paths))
        .into_iter()
        .filter_map(|report| report.error)
        .collect();
    errors.extend(add_plugin_dir(&paths.docker_config, &paths.plugins).err());
    for file in rc_files(&paths.home, shell) {
        let result = match (mode, rc_state(&file.path)) {
            (PathMode::Manual, RcState::Added) => remove_block(&file).map(|_| ()),
            (PathMode::Automatic, RcState::Missing) => {
                match rc_access(&file.path, |path| chezmoi_manages(&paths.home, path)) {
                    RcAccess::Writable => add_block(&file),
                    RcAccess::Skip(_) => Ok(()),
                }
            }
            _ => Ok(()),
        };
        errors.extend(result.err());
    }
    errors
}

/// Removes the links, the plugin folder, and the blocks. Returns the errors.
pub fn uninstall(paths: &ToolPaths, shell: Option<&Path>) -> Vec<String> {
    let mut errors = remove_links(paths);
    errors.extend(remove_plugin_dir(&paths.docker_config, &paths.plugins).err());
    for file in rc_files(&paths.home, shell) {
        errors.extend(remove_block(&file).err());
    }
    errors
}

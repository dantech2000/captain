//! Asks chezmoi whether it manages a file, so Captain does not write a file that
//! the next `chezmoi apply` overwrites.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use super::command::output_within;
use crate::tools::locate_tool;

const TIMEOUT: Duration = Duration::from_secs(5);

/// Where chezmoi gets installed, after `PATH`. An app started from the Finder has
/// a short `PATH`.
fn install_dirs(home: &Path) -> Vec<PathBuf> {
    let mut dirs = vec![
        PathBuf::from("/opt/homebrew/bin"),
        PathBuf::from("/usr/local/bin"),
        home.join(".nix-profile/bin"),
        PathBuf::from("/run/current-system/sw/bin"),
        home.join(".local/bin"),
        home.join("bin"),
    ];
    if let Some(user) = std::env::var_os("USER") {
        dirs.push(Path::new("/etc/profiles/per-user").join(user).join("bin"));
    }
    dirs
}

/// True if `chezmoi source-path <file>` succeeds. False when chezmoi is not
/// installed.
pub fn chezmoi_manages(home: &Path, file: &Path) -> bool {
    let path = std::env::var_os("PATH");
    let binary = format!("chezmoi{}", std::env::consts::EXE_SUFFIX);
    let Some(chezmoi) = locate_tool(&binary, None, path.as_deref(), install_dirs(home), |p| {
        p.is_file()
    }) else {
        return false;
    };
    let mut command = Command::new(chezmoi);
    command.arg("source-path").arg(file);
    output_within(command, TIMEOUT).is_ok_and(|output| output.status.success())
}

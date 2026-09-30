//! Opens `settings.json` in the user's text editor. See ADR 0013.

use std::path::Path;
use std::process::Command;

use captain_core::process_lock::{ProcessLock, settings_lock_path};
use captain_core::settings::Settings;
use gpui_kit::*;

use super::store::settings_file_path;

/// Opens the settings file in the default text editor: `open -t` on macOS,
/// `xdg-open` on Linux, and `start` on Windows. A missing file is created first,
/// with its `$schema` line, so the editor has something to open.
pub fn open_settings_file(cx: &mut App) {
    let Some(path) = settings_file_path(cx) else {
        return;
    };
    cx.background_executor()
        .spawn(async move {
            if let Err(error) = create_if_missing(&path).and_then(|()| open(&path)) {
                tracing::warn!(%error, "cannot open {}", path.display());
            }
        })
        .detach();
}

fn create_if_missing(path: &Path) -> std::io::Result<()> {
    if path.exists() {
        return Ok(());
    }
    let _lock = ProcessLock::acquire(&settings_lock_path(path))?;
    let defaults = Settings::default();
    Settings::save_change(path, &defaults, &defaults).map_err(std::io::Error::other)
}

/// Starts the editor and waits for the launcher, which returns at once.
fn open(path: &Path) -> std::io::Result<()> {
    let status = editor_command(path).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "the launcher exited with {status}"
        )))
    }
}

#[cfg(target_os = "macos")]
fn editor_command(path: &Path) -> Command {
    let mut command = Command::new("open");
    command.arg("-t").arg(path);
    command
}

#[cfg(target_os = "windows")]
fn editor_command(path: &Path) -> Command {
    let mut command = Command::new("cmd");
    command.args(["/c", "start", ""]).arg(path);
    command
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn editor_command(path: &Path) -> Command {
    let mut command = Command::new("xdg-open");
    command.arg(path);
    command
}

//! Keeps the command-line tools in place at each start: the links into this
//! `Captain.app`, the plugin folder, and the PATH blocks. See feature 0035.

use captain_core::cli_tools::{self, CliToolsSettings, ToolPaths};

/// Runs on a background thread, because it may ask chezmoi about the shell files.
/// A `cargo run` build has no bundle, so it does nothing.
pub fn refresh(settings: &CliToolsSettings) {
    if !cli_tools::SUPPORTED || !settings.enabled {
        return;
    }
    let (Some(bundle), Some(paths)) = (cli_tools::running_bundle(), ToolPaths::user()) else {
        return;
    };
    let mode = settings.path;
    std::thread::spawn(move || {
        let shell = cli_tools::login_shell();
        for error in cli_tools::install(&bundle, &paths, shell.as_deref(), mode) {
            tracing::warn!(%error, "cannot set up the command-line tools");
        }
    });
}

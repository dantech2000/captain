//! Which clients are on this computer and whether each one lists Captain's server.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use super::client::{AgentClient, ConfigFormat};
use super::config_edit::{codex_has_server, has_server};
use super::paths::ClientPaths;
use crate::cli_tools::command::output_within;
use crate::cli_tools::parse_command_v;

/// A login shell reads all its files, which can take a while.
const TIMEOUT: Duration = Duration::from_secs(10);

/// One client as Captain finds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientState {
    pub client: AgentClient,
    /// The client's command is on the user's PATH.
    pub has_command: bool,
    /// The file that lists its servers.
    pub config: Option<PathBuf>,
    /// Whether that file lists Captain's server, or why Captain cannot read it.
    pub connected: Result<bool, String>,
}

/// The clients found under `paths`: those whose command is in `commands` (from
/// [`find_commands`]) or whose settings folder exists.
pub fn detect(paths: &ClientPaths, commands: &[(String, Option<PathBuf>)]) -> Vec<ClientState> {
    AgentClient::ALL
        .into_iter()
        .filter_map(|client| {
            let has_command = client.command().is_some_and(|name| {
                commands
                    .iter()
                    .any(|(found, path)| found == name && path.is_some())
            });
            let config = paths.config_file(client);
            let folder = config.as_deref().and_then(|file| match client {
                // The home folder always exists; the file itself says more.
                AgentClient::ClaudeCode => Some(file),
                _ => file.parent(),
            });
            let found = has_command || folder.is_some_and(Path::exists);
            found.then(|| ClientState {
                client,
                has_command,
                connected: config
                    .as_deref()
                    .map_or(Ok(false), |file| lists_captain(client, file, paths)),
                config,
            })
        })
        .collect()
}

/// True if `file` lists Captain's server. A missing file lists nothing.
fn lists_captain(client: AgentClient, file: &Path, paths: &ClientPaths) -> Result<bool, String> {
    let text = match std::fs::read_to_string(file) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(format!("cannot read {}: {error}", paths.tilde(file))),
    };
    match client.format() {
        ConfigFormat::CodexToml => Ok(codex_has_server(&text)),
        ConfigFormat::Json(_) if text.trim().is_empty() => Ok(false),
        ConfigFormat::Json(key) => has_server(&text, key)
            .map_err(|why| format!("cannot read {}: {why}", paths.tilde(file))),
    }
}

/// Where the user's login shell finds each client's command, as a new terminal
/// would: `<shell> -lic 'command -v …'`.
pub fn find_commands(shell: &Path, home: &Path) -> Vec<(String, Option<PathBuf>)> {
    let names: Vec<&str> = AgentClient::ALL
        .into_iter()
        .filter_map(AgentClient::command)
        .collect();
    let mut command = Command::new(shell);
    command
        .arg("-lic")
        .arg(format!("command -v {}", names.join(" ")))
        .current_dir(home);
    let text = output_within(command, TIMEOUT)
        .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
        .unwrap_or_default();
    parse_command_v(&text, &names)
}

#[cfg(test)]
mod tests;

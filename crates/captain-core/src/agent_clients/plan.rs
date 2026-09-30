//! What Connect and Remove do for each client: its own installer command where it
//! has one, a link it confirms (Cursor), or an in-place edit of its file. The sheet
//! shows the step before it runs.

use std::path::{Path, PathBuf};

use base64::Engine as _;
use serde_json::{Value, json};

use super::client::{AgentClient, ConfigFormat, SERVER_NAME};
use super::config_edit::{with_server, without_server};
use super::paths::ClientPaths;
use crate::cli_tools::chezmoi_manages;

/// One step that connects or removes a client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientStep {
    /// Run the client's own command. The first item is the command's name.
    Run(Vec<String>),
    /// Open a link that the client confirms.
    Open(String),
    /// Replace the text of the client's file.
    Edit {
        path: PathBuf,
        before: String,
        after: String,
    },
}

/// The server entry for `client`: `captain mcp` with an absolute path.
pub fn server_entry(client: AgentClient, captain: &Path) -> Value {
    let command = captain.display().to_string();
    match client {
        AgentClient::VsCode => json!({ "type": "stdio", "command": command, "args": ["mcp"] }),
        _ => json!({ "command": command, "args": ["mcp"] }),
    }
}

/// The JSON for a client Captain does not know, for Copy config.
pub fn copy_config(captain: &Path) -> String {
    let config =
        json!({ "mcpServers": { SERVER_NAME: server_entry(AgentClient::Cursor, captain) } });
    serde_json::to_string_pretty(&config).unwrap_or_default()
}

/// How to connect `client` to `captain` (an absolute path to the `captain`
/// command). `has_command` says whether the client's command is on the user's
/// PATH.
pub fn connect_step(
    client: AgentClient,
    paths: &ClientPaths,
    captain: &Path,
    has_command: bool,
) -> Result<ClientStep, String> {
    let path = captain.display().to_string();
    let argv = |args: &[&str]| args.iter().map(ToString::to_string).collect();
    match client {
        AgentClient::ClaudeCode | AgentClient::Codex | AgentClient::GeminiCli if !has_command => {
            Err(missing(client))
        }
        AgentClient::ClaudeCode => Ok(ClientStep::Run(argv(&[
            "claude",
            "mcp",
            "add",
            "--scope",
            "user",
            "--transport",
            "stdio",
            SERVER_NAME,
            "--",
            &path,
            "mcp",
        ]))),
        AgentClient::Codex => Ok(ClientStep::Run(argv(&[
            "codex",
            "mcp",
            "add",
            SERVER_NAME,
            "--",
            &path,
            "mcp",
        ]))),
        AgentClient::GeminiCli => Ok(ClientStep::Run(argv(&[
            "gemini",
            "mcp",
            "add",
            "--scope",
            "user",
            SERVER_NAME,
            &path,
            "mcp",
        ]))),
        AgentClient::VsCode if has_command => {
            let mut entry = server_entry(client, captain);
            entry["name"] = SERVER_NAME.into();
            Ok(ClientStep::Run(argv(&[
                "code",
                "--add-mcp",
                &entry.to_string(),
            ])))
        }
        AgentClient::Cursor => {
            let config = server_entry(client, captain).to_string();
            let encoded = base64::engine::general_purpose::STANDARD.encode(config);
            Ok(ClientStep::Open(format!(
                "cursor://anysphere.cursor-deeplink/mcp/install?name={SERVER_NAME}&config={}",
                encoded
                    .replace('+', "%2B")
                    .replace('/', "%2F")
                    .replace('=', "%3D")
            )))
        }
        _ => edit(client, paths, |text, key| {
            with_server(text, key, &server_entry(client, captain))
        }),
    }
}

/// How to remove Captain's server from `client`.
pub fn remove_step(
    client: AgentClient,
    paths: &ClientPaths,
    has_command: bool,
) -> Result<ClientStep, String> {
    let argv = |args: &[&str]| args.iter().map(ToString::to_string).collect();
    match client {
        AgentClient::ClaudeCode | AgentClient::Codex | AgentClient::GeminiCli if !has_command => {
            Err(missing(client))
        }
        AgentClient::ClaudeCode => Ok(ClientStep::Run(argv(&[
            "claude",
            "mcp",
            "remove",
            SERVER_NAME,
            "--scope",
            "user",
        ]))),
        AgentClient::Codex => Ok(ClientStep::Run(argv(&[
            "codex",
            "mcp",
            "remove",
            SERVER_NAME,
        ]))),
        AgentClient::GeminiCli => Ok(ClientStep::Run(argv(&[
            "gemini",
            "mcp",
            "remove",
            "--scope",
            "user",
            SERVER_NAME,
        ]))),
        _ => edit(client, paths, without_server),
    }
}

/// An edit of `client`'s JSON file with `change`.
fn edit(
    client: AgentClient,
    paths: &ClientPaths,
    change: impl Fn(&str, &str) -> Result<String, String>,
) -> Result<ClientStep, String> {
    let (ConfigFormat::Json(key), Some(path)) = (client.format(), paths.config_file(client)) else {
        return Err(format!("{} does not run on this system.", client.name()));
    };
    if chezmoi_manages(&paths.home, &path) {
        return Err(format!(
            "chezmoi manages {}. Add Captain there with Copy config.",
            paths.tilde(&path)
        ));
    }
    let path = crate::link_target::link_target(&path);
    let before = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => {
            return Err(format!(
                "Captain cannot read {}: {error}",
                paths.tilde(&path)
            ));
        }
    };
    let after = change(&before, key)
        .map_err(|why| format!("Captain cannot change {}: {why}", paths.tilde(&path)))?;
    Ok(ClientStep::Edit {
        path,
        before,
        after,
    })
}

fn missing(client: AgentClient) -> String {
    format!(
        "Captain cannot find the {} command in your terminal's PATH. Install {}, or use \
         Copy config.",
        client.command().unwrap_or_default(),
        client.name()
    )
}

#[cfg(test)]
mod tests;

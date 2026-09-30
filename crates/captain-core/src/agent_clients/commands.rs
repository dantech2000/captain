//! Finds and runs the clients' commands as the user's terminal would. On macOS and
//! Linux, Captain asks the user's login shell. Windows has no such shell, so
//! Captain searches `PATH` with each extension in `PATHEXT`, as `cmd.exe` does.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::client::AgentClient;
use super::detect::find_commands;
use super::run::login_shell_command;
use crate::cli_tools::login_shell;

/// `PATHEXT` when it is not set, as on a new Windows install.
const DEFAULT_PATHEXT: &str = ".COM;.EXE;.BAT;.CMD";

/// Where the user's terminal finds each client's command.
pub fn user_commands(home: &Path) -> Vec<(String, Option<PathBuf>)> {
    if cfg!(windows) {
        let names: Vec<&str> = AgentClient::ALL
            .into_iter()
            .filter_map(AgentClient::command)
            .collect();
        return search_path(&names, std::env::var_os("PATH").as_deref(), &windows_exts());
    }
    login_shell()
        .map(|shell| find_commands(&shell, home))
        .unwrap_or_default()
}

/// A command that runs `argv` as the user's terminal would: through the login
/// shell, or on Windows the file that `PATH` finds for `argv[0]`.
pub fn client_command(argv: &[String]) -> Command {
    let shell = if cfg!(windows) { None } else { login_shell() };
    if let Some(shell) = shell {
        return login_shell_command(&shell, argv);
    }
    let program = if cfg!(windows) {
        search_path(
            &[&argv[0]],
            std::env::var_os("PATH").as_deref(),
            &windows_exts(),
        )
        .pop()
        .and_then(|(_, found)| found)
        .unwrap_or_else(|| PathBuf::from(&argv[0]))
    } else {
        PathBuf::from(&argv[0])
    };
    let mut command = Command::new(program);
    command.args(&argv[1..]);
    command
}

/// The first file for each of `names` in the folders of `path` (a `PATH`-style
/// list). With `exts`, a name matches only with one of them added, such as
/// `claude.cmd`; without, it matches as it is.
pub fn search_path(
    names: &[&str],
    path: Option<&OsStr>,
    exts: &[String],
) -> Vec<(String, Option<PathBuf>)> {
    let dirs: Vec<PathBuf> = path
        .map(std::env::split_paths)
        .into_iter()
        .flatten()
        .collect();
    names
        .iter()
        .map(|name| {
            let files: Vec<String> = if exts.is_empty() {
                vec![name.to_string()]
            } else {
                exts.iter().map(|ext| format!("{name}{ext}")).collect()
            };
            let found = dirs
                .iter()
                .flat_map(|dir| files.iter().map(move |file| dir.join(file)))
                .find(|candidate| candidate.is_file());
            (name.to_string(), found)
        })
        .collect()
}

/// The extensions in `PATHEXT`, or the default ones.
fn windows_exts() -> Vec<String> {
    let pathext = std::env::var_os("PATHEXT").filter(|value| !value.is_empty());
    let text = pathext.map_or_else(
        || DEFAULT_PATHEXT.to_string(),
        |value| value.to_string_lossy().into_owned(),
    );
    text.split(';')
        .filter(|ext| !ext.is_empty())
        .map(String::from)
        .collect()
}

#[cfg(test)]
mod tests;

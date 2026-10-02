//! What a new tab runs: the user's shell in a folder, with `DOCKER_HOST` set to the
//! engine Captain is connected to.

use std::path::{Path, PathBuf};

use captain_core::docker_host::cli_host;
use captain_core::host_shell::{local_env, shell_program};
use captain_core::store::GroupKey;
use captain_terminal::ShellCommand;
use gpui_kit::*;

use crate::engine_host::{captain_endpoint, captain_socket};
use crate::project::ProjectView;
use crate::shell::engine_name;
use crate::terminal::LocalSource;
use crate::workspace::{Connection, Page, Workspace};

/// A shell in `dir` whose docker CLI uses the engine this window shows.
pub fn local_source(dir: &Path, workspace: &Workspace, cx: &App) -> LocalSource {
    let endpoint = match workspace.connection() {
        Connection::Connected(info) => Some(info.endpoint.clone()),
        _ => captain_endpoint(cx),
    };
    let env = local_env(endpoint.as_deref());
    let shell = shell_program();
    let command = ShellCommand {
        program: shell.program,
        args: shell.args,
        cwd: dir.to_path_buf(),
        env: env.set,
        env_remove: env.remove,
    };
    LocalSource::new(command, banner(endpoint.as_deref(), cx))
}

/// The dim first line of a tab: `docker → Captain Engine (unix:///…)`.
fn banner(endpoint: Option<&str>, cx: &App) -> String {
    let Some(endpoint) = endpoint else {
        return "docker → your docker context (Captain is not connected to an engine)".into();
    };
    let name = if captain_socket(cx).as_deref() == Some(endpoint) {
        "Captain Engine"
    } else {
        engine_name(endpoint)
    };
    format!("docker → {name} ({})", cli_host(endpoint))
}

/// Where the + button opens a tab: the folder of the project the Project page
/// shows, else the home folder.
pub fn default_dir(workspace: &Workspace, project: &Entity<ProjectView>, cx: &App) -> PathBuf {
    let on_project = workspace.page() == Page::Project
        && matches!(workspace.focus(), Some(GroupKey::Project(_)));
    on_project
        .then(|| project.read(cx).folder())
        .flatten()
        .or_else(std::env::home_dir)
        .unwrap_or_else(|| PathBuf::from("/"))
}

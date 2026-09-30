//! Builds the `docker compose` arguments for a project action. Pure, so tests need
//! no CLI.

use std::path::{Path, PathBuf};

use captain_core::model::{ComposeProject, ProjectAction};

use crate::Endpoint;

/// The arguments after `docker`, and the folder to run them in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposeCommand {
    pub args: Vec<String>,
    pub dir: PathBuf,
}

/// A `docker compose` subcommand: its name for error messages, its arguments, and
/// whether it needs the Compose files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subcommand {
    pub label: String,
    pub args: Vec<String>,
    pub needs_files: bool,
}

impl From<ProjectAction> for Subcommand {
    fn from(action: ProjectAction) -> Self {
        Self {
            label: action.label().into(),
            args: action.args().iter().map(|arg| arg.to_string()).collect(),
            needs_files: action.needs_files(),
        }
    }
}

/// Builds `compose --ansi never -p <name> -f <file>... <action>` for `project`.
///
/// The command runs in the project's working directory, or in `fallback_dir` when
/// that folder is missing on this computer. `up` and `pull` need every Compose file.
/// `down`, `stop`, and `restart` use the files when all of them exist, and only the
/// project name otherwise. `exists` checks whether a path exists.
pub fn compose_command(
    project: &ComposeProject,
    action: ProjectAction,
    fallback_dir: &Path,
    exists: impl Fn(&Path) -> bool,
) -> Result<ComposeCommand, String> {
    compose_subcommand(project, action.into(), fallback_dir, exists)
}

/// Like [`compose_command`], for any subcommand, for example `exec` or `config`.
pub fn compose_subcommand(
    project: &ComposeProject,
    subcommand: Subcommand,
    fallback_dir: &Path,
    exists: impl Fn(&Path) -> bool,
) -> Result<ComposeCommand, String> {
    let working_dir = project.working_dir.as_deref().map(Path::new);
    let files: Vec<PathBuf> = project
        .config_files
        .iter()
        .map(|file| match working_dir {
            Some(dir) => dir.join(file),
            None => PathBuf::from(file),
        })
        .collect();
    let missing: Vec<String> = files
        .iter()
        .filter(|file| !exists(file))
        .map(|file| file.display().to_string())
        .collect();

    let use_files = !files.is_empty() && missing.is_empty();
    if subcommand.needs_files && !use_files {
        return Err(if files.is_empty() {
            format!(
                "Captain does not know the Compose files of {}. {} needs them.",
                project.name, subcommand.label
            )
        } else {
            format!(
                "Captain cannot find {} on this computer. {} needs the Compose files of {}.",
                missing.join(", "),
                subcommand.label,
                project.name
            )
        });
    }

    let mut args: Vec<String> = ["compose", "--ansi", "never", "-p", &project.name]
        .map(String::from)
        .into();
    if use_files {
        for file in &files {
            args.push("-f".into());
            args.push(file.display().to_string());
        }
    }
    args.extend(subcommand.args);

    let dir = match working_dir {
        Some(dir) if exists(dir) => dir.to_path_buf(),
        _ => fallback_dir.to_path_buf(),
    };
    Ok(ComposeCommand { args, dir })
}

/// The `DOCKER_HOST` value that points the CLI at `endpoint`. The CLI knows `tcp://`
/// but not `http://`.
pub fn docker_host(endpoint: &Endpoint) -> String {
    let host = endpoint.to_string();
    match host.strip_prefix("http://") {
        Some(address) => format!("tcp://{address}"),
        None => host,
    }
}

#[cfg(test)]
mod tests;

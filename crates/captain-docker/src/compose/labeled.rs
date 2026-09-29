//! `docker compose up` that puts extra labels on every container it creates. The
//! labels go in an override file on stdin (`-f -`), so Captain writes nothing into
//! the project folder. Compose records that file as `-` in its config files label.
//! Captain also sets [`ComposeLabels::CAPTAIN_OVERRIDE_LABEL`], and drops that `-`
//! only when it finds this label. See docs/adr/0009-migration.md.

use std::collections::HashMap;
use std::path::Path;
use std::process::{Command, Output};
use std::time::Duration;

use captain_core::model::{ComposeLabels, ComposeProject, ProjectAction};
use captain_core::{EngineError, EngineFuture};
use futures::FutureExt;
use futures::channel::oneshot;

use super::cli::ComposeCli;
use super::command::compose_command;
use super::output::error_message;
use crate::child::input_output_guarded;

/// How long `docker compose config --services` may take.
const LIST_TIMEOUT: Duration = Duration::from_secs(30);
/// How long `docker compose up -d` may take. The images are in the engine already.
const UP_TIMEOUT: Duration = Duration::from_secs(600);

impl ComposeCli {
    /// `up -d` for `services`, or for every service without a profile when
    /// `services` is empty, with `labels` on each container it creates. Dropping
    /// the future kills the running command, and `up` does not start after a drop
    /// during the service listing.
    pub fn up_labeled(
        &self,
        project: &ComposeProject,
        services: &[String],
        labels: HashMap<String, String>,
    ) -> EngineFuture<()> {
        let (list, up) = match self.labeled_commands(project, services) {
            Ok(commands) => commands,
            Err(message) => return futures::future::ready(Err(EngineError::Api(message))).boxed(),
        };
        async move {
            let names = checked(input_output_guarded(list, None, Some(LIST_TIMEOUT)).await)?;
            let names: Vec<&str> = names.lines().map(str::trim).collect();
            let input = label_override(&names, &labels).into_bytes();
            checked(input_output_guarded(up, Some(input), Some(UP_TIMEOUT)).await).map(drop)
        }
        .boxed()
    }

    /// Like [`Self::up_labeled`], but runs to the end on its own thread even when
    /// the future drops. A switch-over uses it after it stopped the source, so the
    /// item does not end up stopped in both engines.
    pub fn up_labeled_to_end(
        &self,
        project: &ComposeProject,
        services: &[String],
        labels: HashMap<String, String>,
    ) -> EngineFuture<()> {
        let up = self.up_labeled(project, services, labels);
        let (tx, rx) = oneshot::channel();
        std::thread::spawn(move || tx.send(futures::executor::block_on(up)).ok());
        rx.map(|result| {
            result.unwrap_or_else(|_| Err(EngineError::Api("the compose command stopped".into())))
        })
        .boxed()
    }

    /// The service listing and the `up` command. Both run with
    /// `--project-directory` set to the project's folder, as the original `up` did,
    /// so relative paths resolve the same and the copy gets the same folder label.
    fn labeled_commands(
        &self,
        project: &ComposeProject,
        services: &[String],
    ) -> Result<(Command, Command), String> {
        let up = ProjectAction::Up;
        let mut built = compose_command(project, up, &std::env::temp_dir(), Path::exists)?;
        if let Some(dir) = &project.working_dir {
            built.args = with_project_directory(&built.args, dir);
        }
        let list = self.command(&list_args(&built.args, up), Some(&built.dir));
        let mut args = stdin_file_args(&built.args, up);
        args.extend(services.iter().cloned());
        Ok((list, self.command(&args, Some(&built.dir))))
    }
}

/// `args` with `--project-directory dir` after `compose`.
fn with_project_directory(args: &[String], dir: &str) -> Vec<String> {
    let mut with_dir = args.to_vec();
    with_dir.splice(1..1, ["--project-directory".to_string(), dir.to_string()]);
    with_dir
}

/// The stdout of a command that succeeded, or its error output.
fn checked(output: std::io::Result<Output>) -> Result<String, EngineError> {
    let output = output.map_err(|err| EngineError::Api(format!("docker compose failed: {err}")))?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let message = error_message(&stderr)
        .unwrap_or_else(|| format!("docker compose exited with {}", output.status));
    Err(EngineError::Api(message))
}

/// `args` for `action` with the action replaced by a listing of every service,
/// including the ones behind a profile.
fn list_args(args: &[String], action: ProjectAction) -> Vec<String> {
    let mut list = args[..args.len() - action.args().len()].to_vec();
    list.extend(["--profile", "*", "config", "--services"].map(String::from));
    list
}

/// `args` for `action` with `-f -` after the project's own files.
fn stdin_file_args(args: &[String], action: ProjectAction) -> Vec<String> {
    let mut with_stdin = args.to_vec();
    let at = args.len() - action.args().len();
    with_stdin.splice(at..at, ["-f".to_string(), "-".to_string()]);
    with_stdin
}

/// A Compose file, in JSON, that adds `labels` and Captain's override label to each
/// of `services`.
fn label_override(services: &[&str], labels: &HashMap<String, String>) -> String {
    let mut labels = labels.clone();
    labels.insert(ComposeLabels::CAPTAIN_OVERRIDE_LABEL.into(), "true".into());
    let services: serde_json::Map<String, serde_json::Value> = services
        .iter()
        .filter(|name| !name.is_empty())
        .map(|name| (name.to_string(), serde_json::json!({ "labels": labels })))
        .collect();
    serde_json::json!({ "services": services }).to_string()
}

#[cfg(test)]
mod tests;

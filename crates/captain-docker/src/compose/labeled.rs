//! `docker compose up` that puts extra labels on every container it creates. The
//! labels go in an override file on stdin (`-f -`), so Captain writes nothing into
//! the project folder. Compose records that file as `-` in its config files label,
//! and Captain drops it when it reads the label. See docs/adr/0009-migration.md.

use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use captain_core::model::{ComposeProject, ProjectAction};
use captain_core::{EngineError, EngineFuture};
use futures::FutureExt;
use futures::channel::oneshot;

use super::cli::ComposeCli;
use super::command::compose_command;
use super::output::error_message;
use crate::process::{input_output_within, output_within};

/// How long `docker compose config --services` may take.
const LIST_TIMEOUT: Duration = Duration::from_secs(30);
/// How long `docker compose up -d` may take. The images are in the engine already.
const UP_TIMEOUT: Duration = Duration::from_secs(600);

impl ComposeCli {
    /// `up -d` for `services`, or for every service without a profile when
    /// `services` is empty, with `labels` on each container it creates.
    pub fn up_labeled(
        &self,
        project: &ComposeProject,
        services: &[String],
        labels: HashMap<String, String>,
    ) -> EngineFuture<()> {
        let up = ProjectAction::Up;
        let built = match compose_command(project, up, &std::env::temp_dir(), Path::exists) {
            Ok(built) => built,
            Err(message) => return futures::future::ready(Err(EngineError::Api(message))).boxed(),
        };
        let list = self.command(&list_args(&built.args, up), Some(&built.dir));
        let mut args = stdin_file_args(&built.args, up);
        args.extend(services.iter().cloned());
        let up = self.command(&args, Some(&built.dir));
        let (tx, rx) = oneshot::channel();
        std::thread::spawn(move || {
            let result = output_within(list, LIST_TIMEOUT)
                .map_err(EngineError::Api)
                .and_then(checked)
                .and_then(|names| {
                    let names: Vec<&str> = names.lines().map(str::trim).collect();
                    let input = label_override(&names, &labels).into_bytes();
                    input_output_within(up, Some(input), UP_TIMEOUT).map_err(EngineError::Api)
                })
                .and_then(checked)
                .map(drop);
            tx.send(result).ok()
        });
        rx.map(|result| {
            result.unwrap_or_else(|_| Err(EngineError::Api("the compose command stopped".into())))
        })
        .boxed()
    }
}

/// The stdout of a command that succeeded, or its error output.
fn checked(output: std::process::Output) -> Result<String, EngineError> {
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

/// A Compose file, in JSON, that adds `labels` to each of `services`.
fn label_override(services: &[&str], labels: &HashMap<String, String>) -> String {
    let services: serde_json::Map<String, serde_json::Value> = services
        .iter()
        .filter(|name| !name.is_empty())
        .map(|name| (name.to_string(), serde_json::json!({ "labels": labels })))
        .collect();
    serde_json::json!({ "services": services }).to_string()
}

#[cfg(test)]
mod tests;

//! Checks a folder the user opens: `docker compose config` on its Compose files,
//! for the project name. See docs/features/0040-new-projects.md.

use std::path::{Path, PathBuf};
use std::time::Duration;

use captain_core::known_projects::project_name_from_config;
use captain_core::{EngineError, EngineFuture};
use futures::FutureExt;

use super::cli::ComposeCli;
use super::output::error_message;
use crate::child::input_output_guarded;

/// How long the check may take.
const CHECK_TIMEOUT: Duration = Duration::from_secs(30);

impl ComposeCli {
    pub(super) fn read_project_name(&self, dir: &Path, files: &[PathBuf]) -> EngineFuture<String> {
        let command = self.command(&config_args(dir, files), Some(dir));
        input_output_guarded(command, None, Some(CHECK_TIMEOUT))
            .map(|output| {
                let output = output.map_err(|error| {
                    EngineError::Api(format!("cannot run docker compose: {error}"))
                })?;
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(EngineError::Api(error_message(&stderr).unwrap_or_else(
                        || format!("docker compose exited with {}", output.status),
                    )));
                }
                project_name_from_config(&String::from_utf8_lossy(&output.stdout))
                    .map_err(EngineError::Api)
            })
            .boxed()
    }
}

/// `compose --ansi never --project-directory <dir> -f <file>... config --format json`.
fn config_args(dir: &Path, files: &[PathBuf]) -> Vec<String> {
    let mut args: Vec<String> = ["compose", "--ansi", "never", "--project-directory"]
        .map(String::from)
        .into();
    args.push(dir.display().to_string());
    for file in files {
        args.push("-f".into());
        args.push(file.display().to_string());
    }
    args.extend(["config", "--format", "json"].map(String::from));
    args
}

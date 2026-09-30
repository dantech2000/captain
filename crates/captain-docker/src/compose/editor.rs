//! The project editor's commands: checks of unsaved text on stdin, the Dockerfile
//! list, the `up --dry-run` preview, and `up` itself. See
//! docs/features/0039-compose-and-dockerfile-editor.md.

use std::path::{Path, PathBuf};
use std::time::Duration;

use captain_core::model::ComposeProject;
use captain_core::project_files::{
    EditableFile, LineProblem, UpPreview, build_check_problems, compose_problems, dockerfiles,
    parse_dry_run,
};
use captain_core::{EngineError, EngineFuture};
use futures::FutureExt;

use super::cli::ComposeCli;
use super::command::{Subcommand, compose_subcommand};
use super::output::error_message;
use crate::child::{input_output_guarded, output_guarded};

/// How long a check of unsaved text may take.
const CHECK_TIMEOUT: Duration = Duration::from_secs(30);

/// How long the dry run may take. It can hang forever when a service waits for
/// another to complete (Compose issue #14269).
const PREVIEW_TIMEOUT: Duration = Duration::from_secs(60);

impl ComposeCli {
    pub(super) fn list_dockerfiles(
        &self,
        project: &ComposeProject,
    ) -> EngineFuture<Vec<EditableFile>> {
        let Some(dir) = project.working_dir.clone().map(PathBuf::from) else {
            return ready(Ok(Vec::new()));
        };
        let config = Subcommand {
            label: "The file list".into(),
            args: ["config", "--format", "json"].map(String::from).into(),
            needs_files: true,
        };
        let command = match self.project_command(project, config, &[]) {
            Ok(command) => command,
            Err(message) => return ready(Err(EngineError::Api(message))),
        };
        output_guarded(command)
            .map(move |output| {
                let output = succeeded(output, "docker compose")?;
                dockerfiles(&String::from_utf8_lossy(&output.stdout), &dir)
                    .map_err(EngineError::Api)
            })
            .boxed()
    }

    pub(super) fn check_text(
        &self,
        project: &ComposeProject,
        file: &Path,
        text: String,
    ) -> EngineFuture<Vec<LineProblem>> {
        let Some(dir) = project.working_dir.clone().map(PathBuf::from) else {
            return ready(Ok(Vec::new()));
        };
        let args = check_args(project, &dir, file);
        let command = self.command(&args, Some(&dir));
        input_output_guarded(
            command,
            Some(text.clone().into_bytes()),
            Some(CHECK_TIMEOUT),
        )
        .map(move |output| {
            let output = output.map_err(|err| cannot_run("docker compose", &err))?;
            Ok(compose_problems(
                &String::from_utf8_lossy(&output.stderr),
                &text,
            ))
        })
        .boxed()
    }

    pub(super) fn check_build(
        &self,
        context: &Path,
        text: String,
    ) -> EngineFuture<Vec<LineProblem>> {
        let args: Vec<String> = ["build", "--call", "check,format=json", "-q", "-f", "-"]
            .map(String::from)
            .into_iter()
            .chain([context.display().to_string()])
            .collect();
        let command = self.command(&args, Some(context));
        input_output_guarded(command, Some(text.into_bytes()), Some(CHECK_TIMEOUT))
            .map(|output| {
                let output = output.map_err(|err| cannot_run("docker build", &err))?;
                let stdout = String::from_utf8_lossy(&output.stdout);
                build_check_problems(&stdout).map_err(|why| {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    EngineError::Api(error_message(&stderr).unwrap_or(why))
                })
            })
            .boxed()
    }

    pub(super) fn dry_run(&self, project: &ComposeProject) -> EngineFuture<UpPreview> {
        let up = Subcommand {
            label: "The preview".into(),
            args: ["up", "-d", "--dry-run", "--remove-orphans"]
                .map(String::from)
                .into(),
            needs_files: true,
        };
        let mut command = match compose_subcommand(project, up, &std::env::temp_dir(), Path::exists)
        {
            Ok(args) => args,
            Err(message) => return ready(Err(EngineError::Api(message))),
        };
        // `--progress` belongs to `docker compose`, before the subcommand.
        command
            .args
            .splice(1..1, ["--progress".into(), "json".into()]);
        let command = self.command(&command.args, Some(&command.dir));
        let name = project.name.clone();
        let known: Vec<(String, String)> = project
            .services
            .iter()
            .flat_map(|service| {
                let service_name = service.name.clone();
                service.containers.iter().map(move |c| {
                    (
                        c.name.trim_start_matches('/').to_string(),
                        service_name.clone(),
                    )
                })
            })
            .collect();
        input_output_guarded(command, None, Some(PREVIEW_TIMEOUT))
            .map(move |output| {
                let output = output.map_err(|err| match err.kind() {
                    std::io::ErrorKind::TimedOut => EngineError::Api(
                        "The preview did not finish in 60 seconds. A dry run can hang when a \
                         service waits for another to complete."
                            .into(),
                    ),
                    _ => cannot_run("docker compose", &err),
                })?;
                let stderr = String::from_utf8_lossy(&output.stderr);
                if !output.status.success() {
                    return Err(EngineError::Api(error_message(&stderr).unwrap_or_else(
                        || format!("docker compose exited with {}", output.status),
                    )));
                }
                Ok(parse_dry_run(&stderr, &name, &known))
            })
            .boxed()
    }

    pub(super) fn up_now(
        &self,
        project: &ComposeProject,
        build: &[String],
    ) -> EngineFuture<String> {
        let mut args: Vec<String> = ["up", "-d"].map(String::from).into();
        if build.is_empty() {
            args.push("--remove-orphans".into());
        } else {
            args.push("--build".into());
            args.extend(build.iter().cloned());
        }
        let up = Subcommand {
            label: "Up".into(),
            args,
            needs_files: true,
        };
        let command = match self.project_command(project, up, &[]) {
            Ok(command) => command,
            Err(message) => return ready(Err(EngineError::Api(message))),
        };
        output_guarded(command)
            .map(|output| {
                let output = succeeded(output, "docker compose")?;
                let mut text = String::from_utf8_lossy(&output.stderr).into_owned();
                text.push_str(&String::from_utf8_lossy(&output.stdout));
                Ok(text)
            })
            .boxed()
    }
}

/// `compose --ansi never -p <name> --project-directory <dir> -f ... config --quiet`,
/// with `-f -` in place of `edited`, so Compose reads the unsaved text from stdin
/// and resolves relative paths as for the real file.
pub fn check_args(project: &ComposeProject, dir: &Path, edited: &Path) -> Vec<String> {
    let mut args: Vec<String> = ["compose", "--ansi", "never", "-p", &project.name]
        .map(String::from)
        .into();
    args.push("--project-directory".into());
    args.push(dir.display().to_string());
    for file in &project.config_files {
        let path = dir.join(file);
        args.push("-f".into());
        args.push(if path == edited {
            "-".into()
        } else {
            path.display().to_string()
        });
    }
    args.extend(["config", "--quiet"].map(String::from));
    args
}

fn ready<T: Send + 'static>(result: Result<T, EngineError>) -> EngineFuture<T> {
    futures::future::ready(result).boxed()
}

fn cannot_run(program: &str, error: &std::io::Error) -> EngineError {
    EngineError::Api(format!("cannot run {program}: {error}"))
}

/// The output of a command that exited with success, or its error message.
fn succeeded(
    output: std::io::Result<std::process::Output>,
    program: &str,
) -> Result<std::process::Output, EngineError> {
    let output = output.map_err(|err| cannot_run(program, &err))?;
    if output.status.success() {
        return Ok(output);
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(EngineError::Api(error_message(&stderr).unwrap_or_else(
        || format!("{program} exited with {}", output.status),
    )))
}

#[cfg(test)]
mod tests;

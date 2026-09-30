//! Runs the `docker compose` CLI on a plain thread and hands the result back through
//! a runtime-neutral future. See docs/adr/0005-compose-via-cli.md.

use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::time::Duration;

use captain_core::model::{ComposeProject, ProjectAction, ProjectTask, ProjectTasks, TaskOutput};
use captain_core::project_files::{EditableFile, LineProblem, UpPreview};
use captain_core::{EngineError, EngineFuture, ProjectRunner};
use futures::FutureExt;

use super::command::{Subcommand, compose_subcommand, docker_host};
use super::docker_cli::DockerCli;
use super::output::{error_message, parse_version};
use crate::Endpoint;
use crate::child::output_guarded;

/// How long `docker compose version` may take before Captain gives up on the CLI.
const DETECT_TIMEOUT: Duration = Duration::from_secs(5);

/// The `docker compose` CLI, pointed at one engine endpoint.
#[derive(Debug, Clone)]
pub struct ComposeCli {
    docker: DockerCli,
    host: String,
    version: String,
}

impl ComposeCli {
    /// Finds the `docker` binary and asks it for the Compose version. Fails with a
    /// reason when there is no binary or no Compose plugin.
    ///
    /// This blocks for up to a few seconds. Call it from a background thread.
    pub fn detect(endpoint: &Endpoint) -> Result<Self, String> {
        let docker =
            DockerCli::find().ok_or_else(|| "the docker CLI is not installed".to_string())?;
        let mut cli = Self {
            docker,
            host: docker_host(endpoint),
            version: String::new(),
        };
        let output = cli.output_within(&["compose", "version", "--format", "json"])?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(error_message(&stderr)
                .unwrap_or_else(|| "the Compose plugin is not installed".into()));
        }
        cli.version = parse_version(&String::from_utf8_lossy(&output.stdout))
            .ok_or_else(|| "cannot read the Compose version".to_string())?;
        tracing::info!(version = %cli.version, docker = %cli.docker.binary.display(), "found docker compose");
        Ok(cli)
    }

    /// A `docker` command with the environment pointed at Captain's endpoint.
    pub(super) fn command(&self, args: &[String], dir: Option<&Path>) -> Command {
        let mut command = self.docker.command();
        command
            .args(args)
            .env("DOCKER_HOST", &self.host)
            // DOCKER_HOST and DOCKER_CONTEXT together make the CLI refuse to run.
            .env_remove("DOCKER_CONTEXT")
            .stdin(Stdio::null());
        if let Some(dir) = dir {
            command.current_dir(dir);
        }
        command
    }

    /// Runs `args` and waits at most [`DETECT_TIMEOUT`]. A command that hangs is
    /// killed.
    fn output_within(&self, args: &[&str]) -> Result<Output, String> {
        let args: Vec<String> = args.iter().map(ToString::to_string).collect();
        crate::process::output_within(self.command(&args, None), DETECT_TIMEOUT)
    }
}

impl ProjectRunner for ComposeCli {
    fn version(&self) -> &str {
        &self.version
    }

    fn run_project(&self, project: &ComposeProject, action: ProjectAction) -> EngineFuture<()> {
        self.run_with(project, action, &[])
    }

    fn tasks(&self, project: &ComposeProject) -> EngineFuture<ProjectTasks> {
        self.read_tasks(project)
    }

    fn run_task(&self, project: &ComposeProject, task: &ProjectTask) -> EngineFuture<TaskOutput> {
        self.exec_task(project, task)
    }

    fn dockerfiles(&self, project: &ComposeProject) -> EngineFuture<Vec<EditableFile>> {
        self.list_dockerfiles(project)
    }

    fn check_compose(
        &self,
        project: &ComposeProject,
        file: &Path,
        text: String,
    ) -> EngineFuture<Vec<LineProblem>> {
        self.check_text(project, file, text)
    }

    fn check_dockerfile(&self, context: &Path, text: String) -> EngineFuture<Vec<LineProblem>> {
        self.check_build(context, text)
    }

    fn preview_up(&self, project: &ComposeProject) -> EngineFuture<UpPreview> {
        self.dry_run(project)
    }

    fn apply_up(&self, project: &ComposeProject, build: &[String]) -> EngineFuture<String> {
        self.up_now(project, build)
    }
}

impl ComposeCli {
    /// Runs `subcommand` (a [`ProjectAction`] or any other) for `project`, with
    /// `services` after its arguments, for example `restart worker`.
    pub fn run_with(
        &self,
        project: &ComposeProject,
        subcommand: impl Into<Subcommand>,
        services: &[String],
    ) -> EngineFuture<()> {
        let command = match self.project_command(project, subcommand.into(), services) {
            Ok(command) => command,
            Err(message) => return futures::future::ready(Err(EngineError::Api(message))).boxed(),
        };
        // Dropping the future kills the command. Migration uses `up_labeled` instead.
        output_guarded(command).map(checked).boxed()
    }

    /// The command for `subcommand` on `project`, with `extra` arguments after it.
    pub(super) fn project_command(
        &self,
        project: &ComposeProject,
        subcommand: Subcommand,
        extra: &[String],
    ) -> Result<Command, String> {
        let mut built =
            compose_subcommand(project, subcommand, &std::env::temp_dir(), Path::exists)?;
        built.args.extend(extra.iter().cloned());
        Ok(self.command(&built.args, Some(&built.dir)))
    }
}

/// Fails with the command's error output.
fn checked(output: std::io::Result<Output>) -> Result<(), EngineError> {
    let output =
        output.map_err(|err| EngineError::Api(format!("cannot run docker compose: {err}")))?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let message = error_message(&stderr)
        .unwrap_or_else(|| format!("docker compose exited with {}", output.status));
    Err(EngineError::Api(message))
}

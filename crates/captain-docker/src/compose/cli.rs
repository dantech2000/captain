//! Runs the `docker compose` CLI on a plain thread and hands the result back through
//! a runtime-neutral future. See docs/adr/0005-compose-via-cli.md.

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::mpsc;
use std::time::Duration;

use captain_core::model::{ComposeProject, ProjectAction};
use captain_core::{EngineError, EngineFuture, ProjectRunner};
use futures::FutureExt;
use futures::channel::oneshot;

use super::command::{compose_command, docker_host};
use super::locate::locate_docker;
use super::output::{error_message, parse_version};
use crate::Endpoint;

/// How long `docker compose version` may take before Captain gives up on the CLI.
const DETECT_TIMEOUT: Duration = Duration::from_secs(5);

/// The `docker compose` CLI, pointed at one engine endpoint.
#[derive(Debug, Clone)]
pub struct ComposeCli {
    docker: PathBuf,
    host: String,
    version: String,
}

impl ComposeCli {
    /// Finds the `docker` binary and asks it for the Compose version. Fails with a
    /// reason when there is no binary or no Compose plugin.
    ///
    /// This blocks for up to a few seconds. Call it from a background thread.
    pub fn detect(endpoint: &Endpoint) -> Result<Self, String> {
        let path = std::env::var_os("PATH");
        let home = std::env::home_dir();
        let docker = locate_docker(path.as_deref(), home.as_deref(), Path::is_file)
            .ok_or_else(|| "the docker CLI is not installed".to_string())?;
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
        tracing::info!(version = %cli.version, docker = %cli.docker.display(), "found docker compose");
        Ok(cli)
    }

    /// A `docker` command with the environment pointed at Captain's endpoint.
    fn command(&self, args: &[String], dir: Option<&Path>) -> Command {
        let mut command = Command::new(&self.docker);
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

    /// Runs `args` and waits at most [`DETECT_TIMEOUT`]. A command that hangs keeps
    /// its thread, which is fine for a one-off check.
    fn output_within(&self, args: &[&str]) -> Result<Output, String> {
        let args: Vec<String> = args.iter().map(ToString::to_string).collect();
        let mut command = self.command(&args, None);
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || tx.send(command.output()).ok());
        match rx.recv_timeout(DETECT_TIMEOUT) {
            Ok(result) => result.map_err(|err| err.to_string()),
            Err(_) => Err("docker compose did not answer".into()),
        }
    }
}

impl ProjectRunner for ComposeCli {
    fn version(&self) -> &str {
        &self.version
    }

    fn run_project(&self, project: &ComposeProject, action: ProjectAction) -> EngineFuture<()> {
        let built = compose_command(project, action, &std::env::temp_dir(), Path::exists);
        let command = match built {
            Ok(built) => self.command(&built.args, Some(&built.dir)),
            Err(message) => return futures::future::ready(Err(EngineError::Api(message))).boxed(),
        };
        let (tx, rx) = oneshot::channel();
        std::thread::spawn(move || tx.send(run(command)).ok());
        rx.map(|result| {
            result.unwrap_or_else(|_| Err(EngineError::Api("the compose command stopped".into())))
        })
        .boxed()
    }
}

/// Runs `command` to the end. Fails with the command's error output.
fn run(mut command: Command) -> Result<(), EngineError> {
    let output = command
        .output()
        .map_err(|err| EngineError::Api(format!("cannot run docker compose: {err}")))?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let message = error_message(&stderr)
        .unwrap_or_else(|| format!("docker compose exited with {}", output.status));
    Err(EngineError::Api(message))
}

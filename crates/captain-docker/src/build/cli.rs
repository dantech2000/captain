//! Runs `docker buildx build` on a plain thread and streams its output.

use std::process::{Command, Stdio};
use std::time::Duration;

use captain_core::model::BuildSpec;
use captain_core::{EngineStream, ImageBuilder};

use super::args::{build_args, parse_buildx_version, with_absolute_context};
use super::lines::stream_lines;
use crate::compose::{DockerCli, docker_host, error_message};
use crate::{Endpoint, process};

/// How long `docker buildx version` may take before Captain gives up on buildx.
const DETECT_TIMEOUT: Duration = Duration::from_secs(5);

/// The `docker buildx` CLI, pointed at one engine endpoint.
#[derive(Debug, Clone)]
pub struct BuildCli {
    docker: DockerCli,
    host: String,
    version: String,
}

impl BuildCli {
    /// Finds the `docker` binary and asks it for the buildx version. Fails with a
    /// reason when there is no binary or no buildx plugin.
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
        let mut command = cli.command();
        command.args(["buildx", "version"]);
        let output = process::output_within(command, DETECT_TIMEOUT)?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(error_message(&stderr)
                .unwrap_or_else(|| "the buildx plugin is not installed".into()));
        }
        cli.version = parse_buildx_version(&String::from_utf8_lossy(&output.stdout))
            .ok_or_else(|| "cannot read the buildx version".to_string())?;
        tracing::info!(version = %cli.version, docker = %cli.docker.binary.display(), "found docker buildx");
        Ok(cli)
    }

    /// The buildx version, for example `v0.35.0`.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// A `docker` command with the environment pointed at Captain's endpoint.
    fn command(&self) -> Command {
        let mut command = self.docker.command();
        command
            .env("DOCKER_HOST", &self.host)
            // DOCKER_HOST and DOCKER_CONTEXT together make the CLI refuse to run.
            .env_remove("DOCKER_CONTEXT")
            .stdin(Stdio::null());
        command
    }
}

impl ImageBuilder for BuildCli {
    fn build(&self, spec: &BuildSpec) -> EngineStream<String> {
        let spec = with_absolute_context(spec);
        let mut command = self.command();
        command.args(build_args(&spec)).current_dir(&spec.context);
        stream_lines(command, "docker buildx")
    }
}

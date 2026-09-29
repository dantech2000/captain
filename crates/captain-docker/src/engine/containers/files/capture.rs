//! Runs a short command in a container without a TTY and collects its output.

use bollard::Docker;
use bollard::container::LogOutput;
use bollard::exec::StartExecResults;
use bollard::models::ExecConfig;
use captain_core::EngineError;
use futures::StreamExt;

use super::super::exec::exit_code;
use crate::mapping;

/// What a finished command printed, and its exit code if the engine reported one.
pub struct Captured {
    pub stdout: String,
    pub stderr: String,
    pub code: Option<i64>,
}

impl Captured {
    /// The shell's codes for "command not found" and "cannot run": the image has no
    /// shell, or no `stat`.
    pub fn missing_command(&self) -> bool {
        matches!(self.code, Some(126 | 127))
    }
}

/// Runs `cmd` in container `id` and waits for it to finish.
pub async fn capture(docker: &Docker, id: &str, cmd: Vec<String>) -> Result<Captured, EngineError> {
    let config = ExecConfig {
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        cmd: Some(cmd),
        ..ExecConfig::default()
    };
    let created = docker
        .create_exec(id, config)
        .await
        .map_err(mapping::engine_error)?;
    let started = docker
        .start_exec(&created.id, None)
        .await
        .map_err(mapping::engine_error)?;
    let mut captured = Captured {
        stdout: String::new(),
        stderr: String::new(),
        code: None,
    };
    if let StartExecResults::Attached { mut output, .. } = started {
        while let Some(frame) = output.next().await {
            match frame.map_err(mapping::engine_error)? {
                LogOutput::StdErr { message } => {
                    captured.stderr.push_str(&String::from_utf8_lossy(&message))
                }
                other => captured
                    .stdout
                    .push_str(&String::from_utf8_lossy(other.as_ref())),
            }
        }
    }
    captured.code = exit_code(docker, &created.id).await;
    Ok(captured)
}

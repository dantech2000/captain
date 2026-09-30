//! Tasks from `x-captain.tasks`: read with `docker compose config`, run with
//! `docker compose exec -T`. See docs/features/0030-project-window.md.

use captain_core::model::{ComposeProject, ProjectTask, ProjectTasks, TaskOutput};
use captain_core::{EngineError, EngineFuture};
use futures::FutureExt;

use super::cli::ComposeCli;
use super::command::Subcommand;
use super::output::error_message;
use crate::child::output_guarded;

impl ComposeCli {
    /// Reads the project's merged config as JSON. Top-level `x-` fields stay in it.
    pub(super) fn read_tasks(&self, project: &ComposeProject) -> EngineFuture<ProjectTasks> {
        let config = Subcommand {
            label: "Tasks".into(),
            args: ["config", "--format", "json"].map(String::from).into(),
            needs_files: true,
        };
        let command = match self.project_command(project, config, &[]) {
            Ok(command) => command,
            Err(message) => return futures::future::ready(Err(EngineError::Api(message))).boxed(),
        };
        output_guarded(command)
            .map(|output| {
                let output = output
                    .map_err(|err| EngineError::Api(format!("cannot run docker compose: {err}")))?;
                let stdout = String::from_utf8_lossy(&output.stdout);
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let message = error_message(&stderr)
                        .unwrap_or_else(|| format!("docker compose exited with {}", output.status));
                    return Err(EngineError::Api(message));
                }
                ProjectTasks::parse(&stdout).map_err(EngineError::Api)
            })
            .boxed()
    }

    /// Runs `task` with `exec -T`, so it needs no terminal. The exit code of the
    /// task's command is the result; a failed task is not an error.
    pub(super) fn exec_task(
        &self,
        project: &ComposeProject,
        task: &ProjectTask,
    ) -> EngineFuture<TaskOutput> {
        let exec = Subcommand {
            label: format!("The task {}", task.name),
            args: ["exec", "-T", &task.service].map(String::from).into(),
            needs_files: false,
        };
        let command = match self.project_command(project, exec, &task.command.exec_args()) {
            Ok(command) => command,
            Err(message) => return futures::future::ready(Err(EngineError::Api(message))).boxed(),
        };
        output_guarded(command)
            .map(|output| {
                let output = output
                    .map_err(|err| EngineError::Api(format!("cannot run the task: {err}")))?;
                let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
                text.push_str(&String::from_utf8_lossy(&output.stderr));
                Ok(TaskOutput {
                    exit_code: output.status.code().unwrap_or(-1),
                    output: text,
                })
            })
            .boxed()
    }
}

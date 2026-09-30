//! The answers of the action tools: `start`, `stop`, `restart`, `raise_memory`,
//! and `run_task`. A task's output is container output, so it is masked, capped,
//! and wrapped like logs.

use schemars::JsonSchema;
use serde::Serialize;

use super::log_query::{MAX_BYTES, MAX_LINES};
use super::mask::mask_secrets;
use super::untrusted::{clean, plain, wrap_untrusted};
use crate::format::bytes_label;
use crate::model::{ProjectTask, TaskOutput};
use crate::project_map::raised_memory;

/// What an action did.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct ActionReport {
    /// The tool that ran, such as `restart`.
    pub action: String,
    /// The container or project it ran on.
    pub target: String,
    /// What Captain did, and what to call next.
    pub message: String,
}

impl ActionReport {
    pub fn new(action: &str, target: &str, message: String) -> Self {
        Self {
            action: action.into(),
            target: target.into(),
            message,
        }
    }

    pub fn text(&self) -> String {
        plain(&self.message)
    }
}

/// The new memory limit for a container whose limit is `limit` bytes: twice the
/// old one, at least 512 MB, as the app raises it. A container without a limit can
/// use all of the engine's memory, so a limit would not help.
pub fn memory_raise(name: &str, limit: u64) -> Result<u64, String> {
    if limit == 0 {
        return Err(format!(
            "{name} has no memory limit: it can use all of the engine's memory, so \
             raising its limit does not help. Give the engine more memory in Captain instead."
        ));
    }
    Ok(raised_memory(limit))
}

/// The sentence after a raise, such as "Raised the memory limit of api from 256 MB
/// to 512 MB."
pub fn raised_sentence(name: &str, from: u64, to: u64) -> String {
    format!(
        "Raised the memory limit of {name} from {} to {}. The container keeps running; \
         a Compose file with its own limit sets it back at the next up.",
        bytes_label(from),
        bytes_label(to)
    )
}

/// The end of a task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct TaskReport {
    pub project: String,
    pub task: String,
    /// The service it ran in.
    pub service: String,
    /// The command, as the Compose file declares it. Secret-looking values are
    /// masked.
    pub command: String,
    /// 0 when the task succeeded.
    pub exit_code: i32,
    /// True when the caps left out the start of the output.
    pub truncated: bool,
    /// The last lines of its output, stdout then stderr, between untrusted-output
    /// delimiters. Secret-looking values are masked.
    pub output: String,
}

/// The report of `task` in `project`, keeping the end of its output within the log
/// caps. Escapes go before masking, so they cannot hide a secret-looking key.
pub fn task_report(project: &str, task: &ProjectTask, output: &TaskOutput) -> TaskReport {
    let lines: Vec<String> = output
        .output
        .lines()
        .map(|line| mask_secrets(&clean(line)))
        .collect();
    let mut kept: Vec<&str> = Vec::new();
    let mut bytes = 0;
    for line in lines.iter().rev().take(MAX_LINES) {
        bytes += line.len() + 1;
        if bytes > MAX_BYTES {
            break;
        }
        kept.push(line);
    }
    let truncated = kept.len() < lines.len();
    kept.reverse();
    let source = format!("task {} in {project}/{}", task.name, task.service);
    TaskReport {
        project: project.into(),
        task: task.name.clone(),
        service: task.service.clone(),
        command: mask_secrets(&clean(&task.command.display())),
        exit_code: output.exit_code,
        truncated,
        output: wrap_untrusted(&source, &kept.join("\n")),
    }
}

impl TaskReport {
    pub fn text(&self) -> String {
        let end = match self.exit_code {
            0 => "succeeded".to_string(),
            code => format!("failed with exit code {code}"),
        };
        let cut = if self.truncated {
            " Only the last lines are shown."
        } else {
            ""
        };
        let head = format!(
            "The task {} in {} ({}) {end}.{cut}",
            self.task, self.project, self.service
        );
        format!("{}\n{}", plain(&head), self.output)
    }
}

#[cfg(test)]
mod tests;

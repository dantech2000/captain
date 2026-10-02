//! Tasks from `x-captain.tasks` in a Compose file: named commands that run in a
//! service, like `migrate` in `api`. See docs/features/0030-project-window.md.

use serde_json::Value;

/// One task of a project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectTask {
    pub name: String,
    /// The service to run it in.
    pub service: String,
    pub command: TaskCommand,
}

/// A task's command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskCommand {
    /// A string, run with `sh -c`.
    Shell(String),
    /// A list, run as it is.
    Args(Vec<String>),
}

impl TaskCommand {
    /// The arguments after `docker compose exec -T SERVICE`.
    pub fn exec_args(&self) -> Vec<String> {
        match self {
            Self::Shell(line) => vec!["sh".into(), "-c".into(), line.clone()],
            Self::Args(args) => args.clone(),
        }
    }

    /// The command as one line, for help sentences.
    pub fn display(&self) -> String {
        match self {
            Self::Shell(line) => line.clone(),
            Self::Args(args) => args.join(" "),
        }
    }
}

/// The tasks of a project, and the entries Captain could not read.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProjectTasks {
    /// Sorted by name.
    pub tasks: Vec<ProjectTask>,
    /// One sentence per entry that has no service or no command.
    pub problems: Vec<String>,
}

/// The end of a task: its exit code and its output, stdout then stderr.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskOutput {
    pub exit_code: i32,
    pub output: String,
}

impl ProjectTasks {
    /// Reads `x-captain.tasks` from the output of `docker compose config --format
    /// json`. A file without the field has no tasks.
    pub fn parse(config: &str) -> Result<Self, String> {
        let config: Value = serde_json::from_str(config)
            .map_err(|error| format!("cannot read the Compose config: {error}"))?;
        let Some(entries) = config
            .pointer("/x-captain/tasks")
            .and_then(Value::as_object)
        else {
            return Ok(Self::default());
        };
        let mut tasks = Self::default();
        for (name, entry) in entries {
            match task(name, entry) {
                Ok(task) => tasks.tasks.push(task),
                Err(problem) => tasks.problems.push(problem),
            }
        }
        tasks.tasks.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(tasks)
    }
}

fn task(name: &str, entry: &Value) -> Result<ProjectTask, String> {
    let service = entry
        .get("service")
        .and_then(Value::as_str)
        .filter(|service| !service.is_empty())
        .ok_or_else(|| format!("The task {name} needs a service."))?;
    let command = match entry.get("command") {
        Some(Value::String(line)) if !line.trim().is_empty() => TaskCommand::Shell(unescape(line)),
        Some(Value::Array(items)) if !items.is_empty() => TaskCommand::Args(
            items
                .iter()
                .map(|item| item.as_str().map(unescape))
                .collect::<Option<Vec<_>>>()
                .ok_or_else(|| format!("The command of the task {name} must list strings."))?,
        ),
        _ => return Err(format!("The task {name} needs a command.")),
    };
    Ok(ProjectTask {
        name: name.to_string(),
        service: service.to_string(),
        command,
    })
}

/// `text` with Compose's `$$` turned back into `$`. `config` interpolates
/// `${NAME}` but keeps `$$`, which in an extension field is meant for the shell
/// in the container.
fn unescape(text: &str) -> String {
    text.replace("$$", "$")
}

#[cfg(test)]
mod tests;

//! The arguments of the tools. The doc comments are the schema descriptions that
//! agents read.

use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct ListContainersParams {
    /// Only the containers of this Compose project.
    #[serde(default)]
    pub project: Option<String>,
    /// Only name, state, health, and needs_attention: the shortest answer.
    #[serde(default)]
    pub status_only: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ContainerParams {
    /// The container: its name, its pod/container name, or an ID prefix.
    pub container: String,
}

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct LogsParams {
    /// One container: its name, its pod/container name, or an ID prefix.
    #[serde(default)]
    pub container: Option<String>,
    /// A whole Compose project, its containers' lines merged in time order.
    #[serde(default)]
    pub project: Option<String>,
    /// Past lines per container, before the filters. Default 100, or 1000 with
    /// errors_only or grep; at most 5000.
    #[serde(default)]
    pub tail: Option<usize>,
    /// Only lines from this time on: an age such as 10m, 2h, or 1d, a Unix time, or
    /// an RFC 3339 time. Pass newest_time from an earlier answer to read on.
    #[serde(default)]
    pub since: Option<String>,
    /// Only lines that look like errors (error, fatal, panic, exception).
    #[serde(default)]
    pub errors_only: bool,
    /// Only lines that contain this text, ignoring case.
    #[serde(default)]
    pub grep: Option<String>,
}

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct WaitParams {
    /// One container: its name, its pod/container name, or an ID prefix.
    #[serde(default)]
    pub container: Option<String>,
    /// Every container of a Compose project.
    #[serde(default)]
    pub project: Option<String>,
    /// How long to wait. Default 60, at most 600.
    #[serde(default)]
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct ActionParams {
    /// One container: its name, its pod/container name, or an ID prefix.
    #[serde(default)]
    pub container: Option<String>,
    /// Every service of a Compose project.
    #[serde(default)]
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TaskParams {
    /// The Compose project whose file declares the task.
    pub project: String,
    /// The task's name in the project's x-captain.tasks.
    pub task: String,
}

/// A container or a whole project.
pub enum Target {
    Container(String),
    Project(String),
}

impl Target {
    /// Exactly one of `container` and `project`.
    pub fn of(container: Option<String>, project: Option<String>) -> Result<Self, String> {
        match (container, project) {
            (Some(container), None) => Ok(Self::Container(container)),
            (None, Some(project)) => Ok(Self::Project(project)),
            _ => Err("Give either container or project.".into()),
        }
    }
}

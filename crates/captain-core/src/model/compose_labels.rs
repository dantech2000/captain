/// The Compose labels of a container, besides the project name. Compose writes them
/// on every container it creates.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ComposeLabels {
    /// The service, from `com.docker.compose.service`.
    pub service: Option<String>,
    /// The folder `docker compose` ran in, from
    /// `com.docker.compose.project.working_dir`.
    pub working_dir: Option<String>,
    /// The Compose files, from the comma-separated
    /// `com.docker.compose.project.config_files`.
    pub config_files: Vec<String>,
}

impl ComposeLabels {
    /// Splits the `config_files` label value. Empty parts are dropped, and so is
    /// `-`, a file that Compose read from stdin (Captain's migration label).
    pub fn split_config_files(value: &str) -> Vec<String> {
        value
            .split(',')
            .map(str::trim)
            .filter(|file| !file.is_empty() && *file != "-")
            .map(ToString::to_string)
            .collect()
    }
}

#[cfg(test)]
mod tests;

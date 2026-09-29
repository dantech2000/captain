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
    /// Captain's label on a container that `docker compose up` created with
    /// Captain's labels-only override on stdin. It marks the last `-` in the config
    /// files label as that override, not as a file of the user.
    pub const CAPTAIN_OVERRIDE_LABEL: &str = "dev.captain.compose-labels-override";

    /// Splits the `config_files` label value and drops empty parts. When
    /// `captain_override` is true, it also drops the last `-`, Captain's override.
    /// Any other `-` is a file the user gave on stdin: it stays, so Captain knows it
    /// cannot replay the files.
    pub fn split_config_files(value: &str, captain_override: bool) -> Vec<String> {
        let mut files: Vec<String> = value
            .split(',')
            .map(str::trim)
            .filter(|file| !file.is_empty())
            .map(ToString::to_string)
            .collect();
        if captain_override && files.last().is_some_and(|file| file == "-") {
            files.pop();
        }
        files
    }
}

#[cfg(test)]
mod tests;

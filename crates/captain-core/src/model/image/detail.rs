use super::ExposedPort;
use crate::model::EnvVar;

/// The details the image inspector shows for one image, from `inspect`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImageDetail {
    /// The full ID, for example `sha256:4a3f…`.
    pub id: String,
    pub repo_tags: Vec<String>,
    /// Registry digests, for example `nginx@sha256:…`. Empty for a local build.
    pub repo_digests: Vec<String>,
    /// RFC 3339 creation time. Empty if the engine does not report one.
    pub created: String,
    pub architecture: String,
    /// The architecture variant, for example `v8`. Often empty.
    pub variant: String,
    pub os: String,
    /// The size on disk in bytes.
    pub size: u64,
    pub config: ImageConfig,
}

/// The run defaults an image sets: `ENTRYPOINT`, `CMD`, `ENV`, `EXPOSE`, and more.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImageConfig {
    pub entrypoint: Vec<String>,
    pub cmd: Vec<String>,
    pub env: Vec<EnvVar>,
    /// Sorted by port, then protocol.
    pub exposed_ports: Vec<ExposedPort>,
    /// Empty means `/`.
    pub working_dir: String,
    /// Empty means `root`.
    pub user: String,
    /// Sorted by key.
    pub labels: Vec<(String, String)>,
}

impl ImageDetail {
    /// The platform, for example `linux/arm64/v8` or `linux/amd64`.
    pub fn platform(&self) -> String {
        [&self.os, &self.architecture, &self.variant]
            .into_iter()
            .filter(|part| !part.is_empty())
            .cloned()
            .collect::<Vec<_>>()
            .join("/")
    }

    /// The creation date and time in UTC, for example `2024-05-01 12:30`. Returns
    /// the input unchanged if it is not RFC 3339, and `—` if it is empty.
    pub fn created_label(&self) -> String {
        let created = self.created.as_str();
        if created.is_empty() {
            return "—".into();
        }
        match (created.get(..10), created.get(11..16)) {
            (Some(date), Some(time)) if created.as_bytes().get(10) == Some(&b'T') => {
                format!("{date} {time} UTC")
            }
            _ => created.to_string(),
        }
    }
}

impl ImageConfig {
    /// The entrypoint and the command as one line, as `docker run` would start them.
    pub fn command_line(&self) -> String {
        self.entrypoint
            .iter()
            .chain(&self.cmd)
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests;

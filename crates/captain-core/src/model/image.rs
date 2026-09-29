//! Image models: the image list entry, pull progress, pull references, the inspect
//! details and history, what a new container needs, builds, and scan reports.

mod build_spec;
mod detail;
mod exposed_port;
mod layer;
mod pull_progress;
mod reference;
mod run_spec;
mod scan;
mod trivy;

pub use build_spec::BuildSpec;
pub use detail::{ImageConfig, ImageDetail};
pub use exposed_port::ExposedPort;
pub use layer::{ImageLayer, largest_layer_size};
pub use pull_progress::PullProgress;
pub use reference::ImageReference;
pub use run_spec::{PublishPort, RestartPolicy, RunSpec};
pub use scan::{ScanProgress, ScanReport, Severity, Vulnerability};
pub use trivy::{parse_trivy_report, trivy_status};

/// What Captain shows for a missing repository or tag.
const NONE_LABEL: &str = "<none>";

/// An image as Captain shows it in lists.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Image {
    /// The full ID, for example `sha256:4a3f…`.
    pub id: String,
    /// Tags such as `nginx:1.27`. Empty for an untagged image.
    pub repo_tags: Vec<String>,
    /// The size on disk in bytes.
    pub size: u64,
    /// Creation time as a Unix timestamp in seconds.
    pub created: i64,
    /// The number of containers, running or stopped, that use this image.
    pub containers: usize,
    /// True if the image has no tag. `docker image prune` removes these.
    pub dangling: bool,
}

impl Image {
    /// The first 12 hex characters of the ID, as the Docker CLI shows it.
    pub fn short_id(&self) -> &str {
        let hex = self.id.strip_prefix("sha256:").unwrap_or(&self.id);
        &hex[..hex.len().min(12)]
    }

    /// The first tag, for example `nginx:1.27`, or `<none>` for an untagged image.
    pub fn display_name(&self) -> String {
        match self.repo_tags.first() {
            Some(tag) => tag.clone(),
            None => NONE_LABEL.to_string(),
        }
    }

    /// The first tag split into repository and tag. An untagged image gives
    /// `("<none>", "<none>")`. A registry port (`localhost:5000/app`) is not a tag.
    pub fn repository_and_tag(&self) -> (&str, &str) {
        let Some(name) = self.repo_tags.first() else {
            return (NONE_LABEL, NONE_LABEL);
        };
        let slash = name.rfind('/').map_or(0, |i| i + 1);
        match name[slash..].rfind(':') {
            Some(colon) => (&name[..slash + colon], &name[slash + colon + 1..]),
            None => (name, ""),
        }
    }

    /// The number of tags after the first one.
    pub fn extra_tag_count(&self) -> usize {
        self.repo_tags.len().saturating_sub(1)
    }

    /// True if at least one container uses the image. The engine refuses to remove it.
    pub fn in_use(&self) -> bool {
        self.containers > 0
    }
}

#[cfg(test)]
mod tests;

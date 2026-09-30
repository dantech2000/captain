use super::{ComposeLabels, Health, PortMapping, kube_display_name};

/// A container as Captain shows it in lists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Container {
    pub id: String,
    /// The name without the leading `/`.
    pub name: String,
    pub image: String,
    pub state: ContainerState,
    /// The human status from the engine, for example "Up 3 minutes".
    pub status: String,
    pub ports: Vec<PortMapping>,
    /// Creation time as a Unix timestamp in seconds.
    pub created: i64,
    /// The Compose project, from the `com.docker.compose.project` label.
    pub compose_project: Option<String>,
    /// The other Compose labels: service, working directory, and config files.
    pub compose: ComposeLabels,
    /// `None` if the container has no health check.
    pub health: Option<Health>,
    /// The Kubernetes namespace of a pod container, from the
    /// `io.kubernetes.pod.namespace` label that cri-dockerd sets.
    pub kube_namespace: Option<String>,
}

impl Container {
    /// True for a container that Kubernetes runs for a pod.
    pub fn is_kubernetes(&self) -> bool {
        self.kube_namespace.is_some()
    }

    /// The name to show: `pod/container` for a Kubernetes pod container, else the
    /// name. See [`kube_display_name`].
    pub fn display_name(&self) -> String {
        self.is_kubernetes()
            .then(|| kube_display_name(&self.name))
            .flatten()
            .unwrap_or_else(|| self.name.clone())
    }

    /// The first 12 characters of the ID, as the Docker CLI shows it.
    pub fn short_id(&self) -> &str {
        &self.id[..self.id.len().min(12)]
    }

    /// The image split into name and tag: `nginx:1.27` gives `("nginx", "1.27")`.
    /// A registry port (`localhost:5000/app`) is not a tag. No tag gives an empty one.
    pub fn image_name_and_tag(&self) -> (&str, &str) {
        let slash = self.image.rfind('/').map_or(0, |i| i + 1);
        match self.image[slash..].rfind(':') {
            Some(colon) if !self.image.starts_with("sha256:") => {
                let colon = slash + colon;
                (&self.image[..colon], &self.image[colon + 1..])
            }
            _ => (&self.image, ""),
        }
    }

    /// A short uptime for the list, for example `3 hours`, `Paused`, or `Exited`.
    pub fn uptime_label(&self) -> String {
        match self.state {
            ContainerState::Running => {
                let status = self.status.split(" (").next().unwrap_or_default();
                status.strip_prefix("Up ").unwrap_or(status).to_string()
            }
            ContainerState::Paused => "Paused".into(),
            ContainerState::Restarting => "Restarting".into(),
            ContainerState::Created => "Created".into(),
            ContainerState::Dead => "Dead".into(),
            _ => "Exited".into(),
        }
    }

    /// Host ports that are published, without duplicates, in order.
    pub fn published_ports(&self) -> Vec<u16> {
        let mut ports: Vec<u16> = self.ports.iter().filter_map(|p| p.public_port).collect();
        ports.dedup();
        ports
    }

    /// The published ports as one comma-separated string.
    pub fn ports_label(&self) -> String {
        let mut labels: Vec<String> = self.ports.iter().map(ToString::to_string).collect();
        labels.dedup();
        labels.join(", ")
    }
}

/// The lifecycle state of a container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContainerState {
    Created,
    Running,
    Paused,
    Restarting,
    Removing,
    Exited,
    Dead,
    Unknown,
}

impl ContainerState {
    /// Parses the state string that the Docker API returns.
    pub fn parse(value: &str) -> Self {
        match value {
            "created" => Self::Created,
            "running" => Self::Running,
            "paused" => Self::Paused,
            "restarting" => Self::Restarting,
            "removing" => Self::Removing,
            "exited" => Self::Exited,
            "dead" => Self::Dead,
            _ => Self::Unknown,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Running => "running",
            Self::Paused => "paused",
            Self::Restarting => "restarting",
            Self::Removing => "removing",
            Self::Exited => "exited",
            Self::Dead => "dead",
            Self::Unknown => "unknown",
        }
    }

    /// True while the container has live processes.
    pub fn is_active(self) -> bool {
        matches!(self, Self::Running | Self::Paused | Self::Restarting)
    }
}

#[cfg(test)]
mod tests;

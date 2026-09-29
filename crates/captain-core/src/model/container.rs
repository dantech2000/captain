use super::PortMapping;

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
}

impl Container {
    /// The first 12 characters of the ID, as the Docker CLI shows it.
    pub fn short_id(&self) -> &str {
        &self.id[..self.id.len().min(12)]
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

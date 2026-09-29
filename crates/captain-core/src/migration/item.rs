use super::Step;

/// One thing the assistant can copy from the source engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationItem {
    /// A user-defined network. The built-in `bridge`, `host`, and `none` are left out.
    Network { name: String, driver: String },
    /// A named volume. `size` is `None` when the engine did not report it.
    Volume {
        name: String,
        size: Option<u64>,
        /// Running containers that mount the volume. Copying files that a running
        /// program writes (a database, for example) may not give a clean copy.
        used_by_running: Vec<String>,
    },
    /// An image with its tags. An untagged image has no tags and copies by ID.
    Image {
        id: String,
        tags: Vec<String>,
        size: u64,
        /// True if a container uses the image.
        in_use: bool,
    },
    /// A Compose project, recreated with `docker compose up -d` when its files exist.
    ComposeProject {
        name: String,
        working_dir: Option<String>,
        config_files: Vec<String>,
        /// True if the working folder and every Compose file exist on this computer.
        files_exist: bool,
        /// Names of the project's containers.
        containers: Vec<String>,
        /// Names of the containers whose own filesystem has changes.
        changed: Vec<String>,
        /// Services with a running container. A switch-over starts only these, so
        /// services that were off (profiles, one-off tools) stay off.
        running_services: Vec<String>,
        /// Names of the running containers. A switch-over stops these in the source.
        running_containers: Vec<String>,
        /// Named volumes that the project's containers mount.
        volumes: Vec<String>,
    },
    /// A container that is not part of a Compose project.
    Container {
        id: String,
        name: String,
        image: String,
        running: bool,
        /// Bytes written to the container's own filesystem, outside volumes. These
        /// are lost when the container is recreated, unless it is snapshotted.
        size_rw: u64,
        /// Named volumes the container mounts.
        volumes: Vec<String>,
    },
}

impl MigrationItem {
    pub fn step(&self) -> Step {
        match self {
            Self::Network { .. } => Step::Networks,
            Self::Volume { .. } => Step::Volumes,
            Self::Image { .. } => Step::Images,
            Self::ComposeProject { .. } => Step::ComposeProjects,
            Self::Container { .. } => Step::Containers,
        }
    }

    /// A key that is unique within a plan, for example `volume:pgdata`.
    pub fn key(&self) -> String {
        match self {
            Self::Network { name, .. } => format!("network:{name}"),
            Self::Volume { name, .. } => format!("volume:{name}"),
            Self::Image { id, .. } => format!("image:{id}"),
            Self::ComposeProject { name, .. } => format!("project:{name}"),
            Self::Container { id, .. } => format!("container:{id}"),
        }
    }

    /// The name to show: the first tag or short ID for an image.
    pub fn label(&self) -> String {
        match self {
            Self::Network { name, .. }
            | Self::Volume { name, .. }
            | Self::ComposeProject { name, .. }
            | Self::Container { name, .. } => name.clone(),
            Self::Image { id, tags, .. } => tags.first().cloned().unwrap_or_else(|| {
                let hex = id.strip_prefix("sha256:").unwrap_or(id);
                hex[..hex.len().min(12)].to_string()
            }),
        }
    }

    /// Bytes to copy for the item itself. A snapshot adds [`Self::changed_bytes`].
    pub fn size(&self) -> u64 {
        match self {
            Self::Volume { size, .. } => size.unwrap_or(0),
            Self::Image { size, .. } => *size,
            _ => 0,
        }
    }

    /// Bytes in a container's own filesystem that recreating it would lose.
    pub fn changed_bytes(&self) -> u64 {
        match self {
            Self::Container { size_rw, .. } => *size_rw,
            _ => 0,
        }
    }

    /// True if the item can switch over: a running container, or a project with a
    /// running service. See docs/adr/0009-migration.md, "Switch-over mode".
    pub fn can_switch_over(&self) -> bool {
        match self {
            Self::Container { running, .. } => *running,
            Self::ComposeProject {
                running_services, ..
            } => !running_services.is_empty(),
            _ => false,
        }
    }

    /// Names of the containers that a switch-over stops in the source.
    pub fn running_containers(&self) -> Vec<String> {
        match self {
            Self::Container {
                name,
                running: true,
                ..
            } => vec![name.clone()],
            Self::ComposeProject {
                running_containers, ..
            } => running_containers.clone(),
            _ => Vec::new(),
        }
    }

    /// True if recreating the item loses changes made inside a container.
    pub fn loses_changes(&self) -> bool {
        match self {
            Self::Container { size_rw, .. } => *size_rw > 0,
            Self::ComposeProject { changed, .. } => !changed.is_empty(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests;

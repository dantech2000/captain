use std::path::Path;

use super::Container;

/// A Compose project, rebuilt from the labels of its containers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposeProject {
    /// From `com.docker.compose.project`.
    pub name: String,
    /// The folder `docker compose` ran in. `None` if no container has the label.
    pub working_dir: Option<String>,
    /// The Compose files, as the labels list them.
    pub config_files: Vec<String>,
    /// Services sorted by name.
    pub services: Vec<ComposeService>,
}

/// One service of a project and its containers, usually one, more when scaled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposeService {
    pub name: String,
    pub containers: Vec<Container>,
}

/// How much of a project runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProjectStatus {
    /// Every container is active.
    Running,
    /// Some containers are active, some are not.
    Partial,
    /// No container is active.
    Stopped,
}

impl ProjectStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Partial => "partial",
            Self::Stopped => "stopped",
        }
    }
}

impl ComposeProject {
    /// Every container of every service.
    pub fn containers(&self) -> impl Iterator<Item = &Container> {
        self.services.iter().flat_map(|s| s.containers.iter())
    }

    pub fn container_count(&self) -> usize {
        self.containers().count()
    }

    /// Containers that are running, paused, or restarting.
    pub fn active_count(&self) -> usize {
        self.containers().filter(|c| c.state.is_active()).count()
    }

    pub fn status(&self) -> ProjectStatus {
        match (self.active_count(), self.container_count()) {
            (0, _) => ProjectStatus::Stopped,
            (active, total) if active == total => ProjectStatus::Running,
            _ => ProjectStatus::Partial,
        }
    }

    /// `1 service` or `3 services`.
    pub fn services_label(&self) -> String {
        match self.services.len() {
            1 => "1 service".into(),
            n => format!("{n} services"),
        }
    }

    /// The working directory with `home` shortened to `~`, for example `~/code/shop`.
    pub fn short_working_dir(&self, home: Option<&Path>) -> Option<String> {
        let dir = self.working_dir.as_deref()?;
        let relative = home.and_then(|home| Path::new(dir).strip_prefix(home).ok());
        Some(match relative {
            Some(rest) if rest.as_os_str().is_empty() => "~".into(),
            Some(rest) => format!("~/{}", rest.display()),
            None => dir.to_string(),
        })
    }
}

#[cfg(test)]
mod tests;

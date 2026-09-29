use crate::model::Container;

/// What a card in the container list holds. The order is the order of the cards:
/// Compose projects, then Kubernetes namespaces, then standalone containers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GroupKey {
    /// A Compose project, from `com.docker.compose.project`.
    Project(String),
    /// The pod containers of a Kubernetes namespace.
    Namespace(String),
    Standalone,
}

impl GroupKey {
    /// The key of the card that shows `container`.
    pub fn of(container: &Container) -> Self {
        match (&container.compose_project, &container.kube_namespace) {
            (Some(project), _) => Self::Project(project.clone()),
            (None, Some(namespace)) => Self::Namespace(namespace.clone()),
            (None, None) => Self::Standalone,
        }
    }
}

/// Containers that belong together in the list: one Compose project, one Kubernetes
/// namespace, or the standalone containers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerGroup {
    pub key: GroupKey,
    pub containers: Vec<Container>,
}

impl ContainerGroup {
    /// The Compose project, if the card is one.
    pub fn project(&self) -> Option<&str> {
        match &self.key {
            GroupKey::Project(project) => Some(project),
            _ => None,
        }
    }

    pub fn running_count(&self) -> usize {
        self.containers
            .iter()
            .filter(|c| c.state.is_active())
            .count()
    }
}

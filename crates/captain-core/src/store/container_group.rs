use crate::model::Container;

/// Containers that belong together in the list: one Compose project, or the
/// standalone containers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerGroup {
    /// `None` for standalone containers.
    pub project: Option<String>,
    pub containers: Vec<Container>,
}

impl ContainerGroup {
    pub fn running_count(&self) -> usize {
        self.containers
            .iter()
            .filter(|c| c.state.is_active())
            .count()
    }
}

use crate::model::{Container, ContainerState};

/// Which containers the list shows.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ContainerFilter {
    #[default]
    All,
    /// Running, paused, or restarting.
    Running,
    /// Created, exited, or dead.
    Stopped,
}

impl ContainerFilter {
    pub const ALL: [ContainerFilter; 3] = [Self::All, Self::Running, Self::Stopped];

    pub fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Running => "Running",
            Self::Stopped => "Stopped",
        }
    }

    pub fn matches(self, container: &Container) -> bool {
        match self {
            Self::All => true,
            Self::Running => container.state.is_active(),
            Self::Stopped => {
                !container.state.is_active() && container.state != ContainerState::Removing
            }
        }
    }
}

#[cfg(test)]
mod tests;

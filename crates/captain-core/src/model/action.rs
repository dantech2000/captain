use super::ContainerState;

/// A lifecycle action on one container.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerAction {
    Start,
    Stop,
    Restart,
    /// Freeze every process in a running container.
    Pause,
    /// Thaw a paused container.
    Unpause,
    /// Remove a stopped container. The engine refuses a running one.
    Remove,
    /// Stop and remove a container in one call. Captain runs it only after the user
    /// confirms "Stop and delete".
    ForceRemove,
}

impl ContainerAction {
    pub fn label(self) -> &'static str {
        match self {
            Self::Start => "Start",
            Self::Stop => "Stop",
            Self::Restart => "Restart",
            Self::Pause => "Pause",
            Self::Unpause => "Resume",
            Self::Remove => "Delete",
            Self::ForceRemove => "Stop and delete",
        }
    }

    /// The label in a result, for example `Stopped`.
    pub fn done_label(self) -> &'static str {
        match self {
            Self::Start => "Started",
            Self::Stop => "Stopped",
            Self::Restart => "Restarted",
            Self::Pause => "Paused",
            Self::Unpause => "Resumed",
            Self::Remove | Self::ForceRemove => "Deleted",
        }
    }

    /// Stop for a container with live processes, Start for the rest.
    pub fn toggle_for(state: ContainerState) -> Self {
        if state.is_active() {
            Self::Stop
        } else {
            Self::Start
        }
    }

    /// Pause for a running container, Unpause for a paused one, and nothing otherwise.
    pub fn pause_toggle_for(state: ContainerState) -> Option<Self> {
        match state {
            ContainerState::Running => Some(Self::Pause),
            ContainerState::Paused => Some(Self::Unpause),
            _ => None,
        }
    }

    /// The removal that works for a container in `state`: a live container must be
    /// force-removed.
    pub fn removal_for(state: ContainerState) -> Self {
        if state.is_active() {
            Self::ForceRemove
        } else {
            Self::Remove
        }
    }

    /// True for the actions that delete the container.
    pub fn removes(self) -> bool {
        matches!(self, Self::Remove | Self::ForceRemove)
    }
}

#[cfg(test)]
mod tests;

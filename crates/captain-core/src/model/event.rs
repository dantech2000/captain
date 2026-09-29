/// One message from the engine event stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineEvent {
    pub kind: EventKind,
    /// The action, for example `start`, `die`, or `destroy`.
    pub action: String,
    /// The ID of the object the event is about.
    pub id: String,
}

/// The type of object an [`EngineEvent`] is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    Container,
    Image,
    Volume,
    Network,
    Other,
}

impl EngineEvent {
    /// Container actions that change what the container list shows.
    /// Frequent actions such as `exec_start` and `health_status` are left out.
    const LIST_ACTIONS: &[&str] = &[
        "create", "start", "restart", "stop", "die", "kill", "pause", "unpause", "rename",
        "destroy", "update",
    ];

    /// True if the container list must reload after this event.
    pub fn changes_container_list(&self) -> bool {
        self.kind == EventKind::Container && Self::LIST_ACTIONS.contains(&self.action.as_str())
    }
}

#[cfg(test)]
mod tests;

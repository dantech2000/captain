/// One message from the engine event stream.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EngineEvent {
    pub kind: EventKind,
    /// The action, for example `start`, `die`, or `destroy`.
    pub action: String,
    /// The ID of the object the event is about.
    pub id: String,
    /// When the engine sent the event, in Unix seconds. `None` when it sent no time.
    pub time: Option<i64>,
    /// The container name, from the actor's `name` attribute.
    pub name: Option<String>,
    /// The exit code of a `die` event, from the actor's `exitCode` attribute.
    pub exit_code: Option<i64>,
}

/// The type of object an [`EngineEvent`] is about.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum EventKind {
    Container,
    Image,
    Volume,
    Network,
    #[default]
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

    /// True if the engine's disk use may have changed: images pulled, built, or
    /// removed, volumes created or removed, containers created or removed.
    pub fn changes_disk_use(&self) -> bool {
        let action = self.action.as_str();
        match self.kind {
            EventKind::Image => !matches!(action, "push" | "save"),
            EventKind::Volume => matches!(action, "create" | "destroy" | "prune"),
            EventKind::Container => matches!(action, "create" | "destroy" | "prune"),
            EventKind::Network | EventKind::Other => action == "prune",
        }
    }
}

#[cfg(test)]
mod tests;

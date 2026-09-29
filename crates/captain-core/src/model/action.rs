/// A lifecycle action on one container.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerAction {
    Start,
    Stop,
    Restart,
    /// Remove a stopped container. Captain never force-removes a running one.
    Remove,
}

impl ContainerAction {
    pub fn label(self) -> &'static str {
        match self {
            Self::Start => "Start",
            Self::Stop => "Stop",
            Self::Restart => "Restart",
            Self::Remove => "Delete",
        }
    }
}

/// The state of the engine's machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostStatus {
    /// The tool that runs the machine is missing or too old. The text says why.
    NotInstalled(String),
    /// The tool exists, but the machine was never set up.
    NotCreated,
    Stopped,
    Starting,
    Running,
    Stopping,
    /// The machine is broken. The text says why.
    Failed(String),
}

impl HostStatus {
    /// A short word or two for status lines.
    pub fn label(&self) -> &'static str {
        match self {
            Self::NotInstalled(_) => "Not installed",
            Self::NotCreated => "Not set up",
            Self::Stopped => "Stopped",
            Self::Starting => "Starting",
            Self::Running => "Running",
            Self::Stopping => "Stopping",
            Self::Failed(_) => "Failed",
        }
    }

    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }

    /// True while a start or a stop is under way.
    pub fn is_busy(&self) -> bool {
        matches!(self, Self::Starting | Self::Stopping)
    }

    /// True if Start makes sense now. A failed machine can try again.
    pub fn can_start(&self) -> bool {
        matches!(self, Self::NotCreated | Self::Stopped | Self::Failed(_))
    }

    /// True if Stop makes sense now.
    pub fn can_stop(&self) -> bool {
        matches!(self, Self::Running | Self::Starting)
    }
}

#[cfg(test)]
mod tests;

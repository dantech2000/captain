//! The engine's state as the window and the menu bar show it: one decision for
//! the status bar, the Diagnostics Engine card, and the menu bar icon. See
//! features 0009 and 0013.

use captain_core::HostStatus;

use super::{Connection, Workspace};

/// The engine's state, from Captain Engine's status (if the settings choose it) and
/// the connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineHealth {
    /// The engine answers.
    Running,
    /// Captain connects for the first time.
    Connecting,
    /// Captain Engine starts or stops.
    Starting,
    /// A working connection dropped and Captain reconnects by itself. The workspace
    /// keeps this for 15 seconds after the drop.
    Reconnecting,
    /// The engine should answer but does not: after those 15 seconds, or when it
    /// never answered.
    NotAnswering,
    /// Captain Engine did not start, or Lima is missing.
    CannotRun,
    Stopped,
    /// Captain Engine was never set up.
    NotSetUp,
}

impl EngineHealth {
    /// `host` is Captain Engine's status, or `None` for another engine. `retrying`
    /// says that Captain reconnects by itself.
    pub fn of(host: Option<&HostStatus>, connection: &Connection, retrying: bool) -> Self {
        match (host, connection) {
            (Some(HostStatus::Starting | HostStatus::Stopping), _) => Self::Starting,
            (Some(HostStatus::Stopped), _) => Self::Stopped,
            (Some(HostStatus::NotCreated), _) => Self::NotSetUp,
            (Some(HostStatus::Failed(_) | HostStatus::NotInstalled(_)), _) => Self::CannotRun,
            (_, Connection::Connected(_)) => Self::Running,
            (_, Connection::Connecting) if retrying => Self::Reconnecting,
            (_, Connection::Connecting) => Self::Connecting,
            (_, Connection::Failed(_)) => Self::NotAnswering,
        }
    }
}

impl Workspace {
    /// The engine's state. `host` is Captain Engine's status when the settings
    /// choose it.
    pub fn engine_health(&self, host: Option<&HostStatus>) -> EngineHealth {
        EngineHealth::of(host, &self.connection, self.reconnecting().is_some())
    }
}

#[cfg(test)]
mod tests;

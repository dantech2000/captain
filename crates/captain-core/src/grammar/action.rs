use std::time::Duration;

use crate::kubernetes::ForwardKey;
use crate::model::{ContainerAction, PortLink, ProjectAction};

/// A page a command can open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Destination {
    Containers,
    Images,
    Volumes,
    Networks,
    Extensions,
    Snapshots,
    Storage,
    PortForwarding,
    Diagnostics,
    Settings,
}

impl Destination {
    pub const ALL: [Destination; 10] = [
        Destination::Containers,
        Destination::Images,
        Destination::Volumes,
        Destination::Networks,
        Destination::Extensions,
        Destination::Snapshots,
        Destination::Storage,
        Destination::PortForwarding,
        Destination::Diagnostics,
        Destination::Settings,
    ];

    /// The word for the page, for example `forwarding`.
    pub fn name(self) -> &'static str {
        match self {
            Destination::Containers => "containers",
            Destination::Images => "images",
            Destination::Volumes => "volumes",
            Destination::Networks => "networks",
            Destination::Extensions => "extensions",
            Destination::Snapshots => "snapshots",
            Destination::Storage => "storage",
            Destination::PortForwarding => "forwarding",
            Destination::Diagnostics => "diagnostics",
            Destination::Settings => "settings",
        }
    }

    pub fn parse(word: &str) -> Option<Destination> {
        Destination::ALL
            .into_iter()
            .find(|page| page.name() == word)
    }
}

/// What a complete command does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Runs `action` on each container: one, the replicas of a service, or the
    /// containers of a project.
    Containers {
        ids: Vec<String>,
        action: ContainerAction,
    },
    /// Runs a `docker compose` command on a project. Down asks first.
    Project {
        name: String,
        action: ProjectAction,
    },
    /// Shows the logs of a container in the inspector.
    Logs {
        id: String,
        /// Only lines from this long ago on.
        since: Option<Duration>,
        /// Only error lines.
        errors: bool,
    },
    /// Shows the Project page with its log.
    ProjectLog(String),
    /// Opens the Terminal tab of a container.
    Shell(String),
    /// Opens the floating log window of a container.
    Float(String),
    /// Opens a web port, or copies the address of another port.
    Open(PortLink),
    /// Forwards a Kubernetes service port, to `local_port` or a free port.
    Forward {
        key: ForwardKey,
        local_port: Option<u16>,
    },
    Go(Destination),
}

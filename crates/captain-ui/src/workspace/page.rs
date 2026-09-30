use gpui_kit::assets::IconName;

use crate::icons::{CaptainIcon, Glyph};

/// A top-level page in the sidebar.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Page {
    #[default]
    Containers,
    /// One sidebar entry: a Compose project, a Kubernetes namespace, or the loose
    /// containers. [`Workspace::focus`](super::Workspace::focus) says which.
    Project,
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

impl Page {
    /// The pages in the sidebar's main list. Extensions, Snapshots, Storage, Port Forwarding,
    /// Diagnostics, and Settings have their own entries at the bottom.
    pub const ALL: [Page; 4] = [
        Page::Containers,
        Page::Images,
        Page::Volumes,
        Page::Networks,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Page::Containers => "Containers",
            Page::Project => "Project",
            Page::Images => "Images",
            Page::Volumes => "Volumes",
            Page::Networks => "Networks",
            Page::Extensions => "Extensions",
            Page::Snapshots => "Snapshots",
            Page::Storage => "Storage",
            Page::PortForwarding => "Port Forwarding",
            Page::Diagnostics => "Diagnostics",
            Page::Settings => "Settings",
        }
    }

    /// Captain's glyph for a resource page; Diagnostics and Settings keep Lucide's.
    pub fn icon(self) -> Glyph {
        match self {
            Page::Containers => CaptainIcon::Container.into(),
            Page::Project => CaptainIcon::Stack.into(),
            Page::Images => CaptainIcon::Image.into(),
            Page::Volumes => CaptainIcon::Volume.into(),
            Page::Networks => CaptainIcon::Network.into(),
            Page::Extensions => CaptainIcon::Extension.into(),
            Page::Snapshots => CaptainIcon::Snapshot.into(),
            Page::Storage => CaptainIcon::Reclaim.into(),
            Page::PortForwarding => CaptainIcon::Forward.into(),
            Page::Diagnostics => IconName::Stethoscope.into(),
            Page::Settings => IconName::Settings.into(),
        }
    }
}

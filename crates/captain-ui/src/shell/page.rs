use gpui_kit::assets::IconName;

/// A top-level section in the sidebar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Containers,
    Images,
    Volumes,
    Networks,
    Compose,
}

impl Page {
    pub const ALL: [Page; 5] = [
        Page::Containers,
        Page::Images,
        Page::Volumes,
        Page::Networks,
        Page::Compose,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Page::Containers => "Containers",
            Page::Images => "Images",
            Page::Volumes => "Volumes",
            Page::Networks => "Networks",
            Page::Compose => "Compose",
        }
    }

    pub fn icon(self) -> IconName {
        match self {
            Page::Containers => IconName::Container,
            Page::Images => IconName::Layers,
            Page::Volumes => IconName::HardDrive,
            Page::Networks => IconName::Network,
            Page::Compose => IconName::Boxes,
        }
    }

    /// Pages that exist in this build. The rest show as disabled until their milestone.
    pub fn is_available(self) -> bool {
        self == Page::Containers
    }
}

use gpui_kit::assets::IconName;

/// A top-level page in the sidebar.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Page {
    #[default]
    Containers,
    Images,
    Volumes,
    Networks,
    Settings,
}

impl Page {
    /// The pages in the sidebar's main list. Settings has its own entry.
    pub const ALL: [Page; 4] = [
        Page::Containers,
        Page::Images,
        Page::Volumes,
        Page::Networks,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Page::Containers => "Containers",
            Page::Images => "Images",
            Page::Volumes => "Volumes",
            Page::Networks => "Networks",
            Page::Settings => "Settings",
        }
    }

    pub fn icon(self) -> IconName {
        match self {
            Page::Containers => IconName::Container,
            Page::Images => IconName::Layers,
            Page::Volumes => IconName::HardDrive,
            Page::Networks => IconName::Network,
            Page::Settings => IconName::Settings,
        }
    }
}

use serde::{Deserialize, Serialize};

/// Who puts `~/.captain/bin` on `PATH`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PathMode {
    /// Captain adds a marked block to each shell file it may write.
    Automatic,
    /// The user adds the line; Captain only shows it.
    #[default]
    Manual,
}

impl PathMode {
    pub const ALL: [PathMode; 2] = [PathMode::Automatic, PathMode::Manual];

    pub fn label(self) -> &'static str {
        match self {
            PathMode::Automatic => "Automatic",
            PathMode::Manual => "Manual",
        }
    }
}

/// The saved choices for the command-line tools.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CliToolsSettings {
    /// Keep the links, the plugin folder, and (in Automatic) the PATH blocks in
    /// place at each start. Off until the user clicks Install or runs
    /// `captain tools install`, so Captain changes no shell or docker file on its
    /// own. `captain tools uninstall` turns it off again.
    pub enabled: bool,
    pub path: PathMode,
}

impl Default for CliToolsSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            path: PathMode::Manual,
        }
    }
}

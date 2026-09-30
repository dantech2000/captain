use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Who puts `~/.captain/bin` on `PATH`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct CliToolsSettings {
    /// Keep the links in `~/.captain/bin`, the Docker CLI plugin folder, and (with
    /// `automatic`) the PATH lines in place at each launch. Off until you click
    /// Install or run `captain tools install`. Applies at the next launch.
    #[schemars(example = true)]
    pub enabled: bool,
    /// Who puts `~/.captain/bin` on PATH: `automatic` adds a marked block to your
    /// shell files, and `manual` leaves them to you. Applies at the next launch.
    #[schemars(example = PathMode::Automatic)]
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

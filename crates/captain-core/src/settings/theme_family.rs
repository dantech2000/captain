use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The color theme. Each one has a light and a dark version; [`super::Appearance`]
/// picks which. The UI holds the exact colors.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ThemeFamily {
    #[default]
    Dusk,
    Periwinkle,
    Harbor,
}

impl ThemeFamily {
    pub const ALL: [ThemeFamily; 3] = [
        ThemeFamily::Dusk,
        ThemeFamily::Periwinkle,
        ThemeFamily::Harbor,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ThemeFamily::Dusk => "Dusk",
            ThemeFamily::Periwinkle => "Periwinkle",
            ThemeFamily::Harbor => "Harbor",
        }
    }
}

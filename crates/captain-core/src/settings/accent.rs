use serde::{Deserialize, Serialize};

/// The accent color preset. The UI picks the exact color for light and dark mode.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Accent {
    #[default]
    Blue,
    Purple,
    Orange,
    Teal,
    Graphite,
}

impl Accent {
    pub const ALL: [Accent; 5] = [
        Accent::Blue,
        Accent::Purple,
        Accent::Orange,
        Accent::Teal,
        Accent::Graphite,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Accent::Blue => "Blue",
            Accent::Purple => "Purple",
            Accent::Orange => "Orange",
            Accent::Teal => "Teal",
            Accent::Graphite => "Graphite",
        }
    }
}

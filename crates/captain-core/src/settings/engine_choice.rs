use serde::{Deserialize, Serialize};

/// Which engine Captain uses. See ADR 0008.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EngineChoice {
    /// Captain Engine: a VM that Captain starts, stops, and configures.
    Captain,
    /// Another engine that Captain connects to but does not control: `DOCKER_HOST`,
    /// the current context, a known socket, or the custom endpoint.
    External,
}

impl EngineChoice {
    pub const ALL: [EngineChoice; 2] = [EngineChoice::Captain, EngineChoice::External];

    pub fn label(self) -> &'static str {
        match self {
            EngineChoice::Captain => "Captain Engine",
            EngineChoice::External => "Other engine",
        }
    }

    /// The engine to use when the settings have no choice yet. Captain Engine is the
    /// default when it can run here, unless the user already saved a custom endpoint.
    pub fn default_for(captain_available: bool, has_custom_endpoint: bool) -> Self {
        if captain_available && !has_custom_endpoint {
            EngineChoice::Captain
        } else {
            EngineChoice::External
        }
    }
}

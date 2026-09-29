use serde::{Deserialize, Deserializer, Serialize};

use super::{Accent, Appearance, EngineChoice};
use crate::HostResources;

/// The settings file format that this build writes. See ADR 0004.
pub const SETTINGS_VERSION: u32 = 1;

/// Everything the user can change on the Settings page.
///
/// Reading is lenient: a missing field gets its default, an unknown field is ignored,
/// and a field with a value this build does not know (for example a new accent from a
/// later version) falls back to its default instead of failing the whole file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// The format version of the file this came from.
    #[serde(deserialize_with = "lenient")]
    pub version: u32,
    #[serde(deserialize_with = "lenient")]
    pub appearance: Appearance,
    #[serde(deserialize_with = "lenient")]
    pub accent: Accent,
    /// A `DOCKER_HOST`-style URL that wins over discovery, for example
    /// `unix:///var/run/docker.sock` or `tcp://10.0.0.5:2375`.
    #[serde(deserialize_with = "lenient")]
    pub engine_endpoint: Option<String>,
    /// Captain Engine or another engine. `None` until the user picks one; see
    /// [`Settings::engine_choice`].
    #[serde(deserialize_with = "lenient")]
    pub engine: Option<EngineChoice>,
    /// Stop Captain Engine when Captain quits.
    #[serde(deserialize_with = "lenient_true")]
    pub stop_engine_on_quit: bool,
    /// The CPUs, memory, and disk for Captain Engine. `None` means the defaults for
    /// this computer.
    #[serde(deserialize_with = "lenient")]
    pub engine_resources: Option<HostResources>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: SETTINGS_VERSION,
            appearance: Appearance::default(),
            accent: Accent::default(),
            engine_endpoint: None,
            engine: None,
            stop_engine_on_quit: true,
            engine_resources: None,
        }
    }
}

impl Settings {
    /// Parses a settings file. Only malformed JSON is an error.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let mut settings: Settings = serde_json::from_str(json)?;
        settings.engine_endpoint = settings
            .engine_endpoint
            .map(|host| host.trim().to_string())
            .filter(|host| !host.is_empty());
        Ok(settings)
    }

    /// The engine to use: the saved choice, or the default when there is none.
    /// `captain_available` says whether Captain Engine can run on this computer.
    pub fn engine_choice(&self, captain_available: bool) -> EngineChoice {
        self.engine.unwrap_or_else(|| {
            EngineChoice::default_for(captain_available, self.engine_endpoint.is_some())
        })
    }

    /// The file contents, stamped with the current [`SETTINGS_VERSION`].
    pub fn to_json(&self) -> String {
        let current = Settings {
            version: SETTINGS_VERSION,
            ..self.clone()
        };
        let mut json = serde_json::to_string_pretty(&current).unwrap_or_default();
        json.push('\n');
        json
    }
}

/// Reads a field, or its default when the value has the wrong type or an unknown variant.
fn lenient<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(T::deserialize(value).unwrap_or_default())
}

/// Reads a flag that defaults to `true`, also when the value has the wrong type.
fn lenient_true<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(value.as_bool().unwrap_or(true))
}

#[cfg(test)]
mod tests;

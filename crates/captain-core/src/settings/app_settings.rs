use serde::{Deserialize, Deserializer, Serialize};

use super::{Appearance, EngineChoice, ThemeFamily};
use crate::HostResources;
use crate::cli_tools::CliToolsSettings;
use crate::daemon::DaemonSettings;
use crate::kubernetes::KubernetesSettings;

/// The settings file format that this build writes. See ADR 0004.
pub const SETTINGS_VERSION: u32 = 1;

/// Everything the user can change on the Settings page.
///
/// Reading is lenient: a missing field gets its default, an unknown field is ignored,
/// and a field with a value this build does not know (for example a new theme from a
/// later version) falls back to its default instead of failing the whole file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// The format version of the file this came from.
    #[serde(deserialize_with = "lenient")]
    pub version: u32,
    #[serde(deserialize_with = "lenient")]
    pub appearance: Appearance,
    /// The color theme. Files from before themes have an `accent` key instead;
    /// it is ignored.
    #[serde(deserialize_with = "lenient")]
    pub theme: ThemeFamily,
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
    /// Captain Engine's Docker daemon: registry mirrors, custom `daemon.json` keys,
    /// and the TCP socket. See feature 0020.
    #[serde(deserialize_with = "lenient")]
    pub engine_daemon: DaemonSettings,
    /// The k3s cluster in Captain Engine. Off by default. See ADR 0010.
    #[serde(deserialize_with = "lenient")]
    pub kubernetes: KubernetesSettings,
    /// Launch with only the menu bar icon, and no main window.
    #[serde(deserialize_with = "lenient")]
    pub start_in_background: bool,
    /// Show Captain's icon in the menu bar (macOS) or the notification area (Windows).
    #[serde(deserialize_with = "lenient_true")]
    pub show_menu_bar_icon: bool,
    /// Write debug-level logs. The Diagnostics page has the switch.
    #[serde(deserialize_with = "lenient")]
    pub debug_logging: bool,
    /// Remove build cache older than 14 days once a week while the engine runs. See
    /// feature 0031.
    #[serde(deserialize_with = "lenient")]
    pub weekly_build_cache_cleanup: bool,
    /// When the weekly cleanup last ran, in Unix seconds.
    #[serde(deserialize_with = "lenient")]
    pub build_cache_cleaned_at: Option<i64>,
    /// The tool links, the plugin folder, and PATH. See feature 0035.
    #[serde(deserialize_with = "lenient")]
    pub command_line_tools: CliToolsSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: SETTINGS_VERSION,
            appearance: Appearance::default(),
            theme: ThemeFamily::default(),
            engine_endpoint: None,
            engine: None,
            stop_engine_on_quit: true,
            engine_resources: None,
            engine_daemon: DaemonSettings::default(),
            kubernetes: KubernetesSettings::default(),
            start_in_background: false,
            show_menu_bar_icon: true,
            debug_logging: false,
            weekly_build_cache_cleanup: false,
            build_cache_cleaned_at: None,
            command_line_tools: CliToolsSettings::default(),
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

    /// True if Captain opens its main window at launch. It stays hidden only when the
    /// user asked for the background and the menu bar icon is up to reach Captain.
    pub fn opens_window_at_launch(&self, tray_up: bool) -> bool {
        !(self.start_in_background && tray_up)
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

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};

use super::{Appearance, EngineChoice, ThemeFamily};
use crate::HostResources;
use crate::agent_tools::AgentToolsSettings;
use crate::cli_tools::CliToolsSettings;
use crate::daemon::DaemonSettings;
use crate::kubernetes::KubernetesSettings;

/// The settings file format that this build writes. Version 2 holds only the values
/// that differ from the defaults, and may have comments. See ADR 0004 and ADR 0013.
pub const SETTINGS_VERSION: u32 = 2;

/// Everything the user can change on the Settings page or in `settings.json`.
///
/// Reading is lenient: a missing field gets its default, an unknown field is ignored,
/// and a field with a value this build does not know (for example a new theme from a
/// later version) falls back to its default instead of failing the whole file.
///
/// The doc comments on the fields are the descriptions in `settings.schema.json` and
/// docs/reference/settings.md, so they speak to the user. `x-captain-group` puts a
/// key under a heading in that reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
#[schemars(
    title = "Captain settings",
    description = "Captain's settings.json. It holds only the settings you change; every other key has its default. See https://github.com/dantech2000/captain/blob/main/docs/reference/settings.md"
)]
pub struct Settings {
    /// The format of this file. Captain writes it; leave it as it is.
    #[serde(deserialize_with = "lenient")]
    pub version: u32,
    /// Light or dark mode. `system` follows the operating system. Applies at once.
    #[serde(deserialize_with = "lenient")]
    #[schemars(extend("x-captain-group" = "Appearance"), example = Appearance::Dark)]
    pub appearance: Appearance,
    /// The color theme. Each theme has a light and a dark version. Applies at once.
    // Files from before themes have an `accent` key instead; it is ignored.
    #[serde(deserialize_with = "lenient")]
    #[schemars(extend("x-captain-group" = "Appearance"), example = ThemeFamily::Harbor)]
    pub theme: ThemeFamily,
    /// A Docker API address that wins over the engines Captain finds, such as
    /// `unix:///var/run/docker.sock` or `tcp://10.0.0.5:2375`. Used when `engine` is
    /// `external`. Applies at the next launch.
    #[serde(deserialize_with = "lenient")]
    #[schemars(extend("x-captain-group" = "Engine"), example = "tcp://10.0.0.5:2375")]
    pub engine_endpoint: Option<String>,
    /// `captain` runs Captain Engine, and `external` connects to an engine that
    /// Captain does not control. Without it, Captain picks Captain Engine when it can
    /// run on this computer. Applies at the next launch.
    // See [`Settings::engine_choice`].
    #[serde(deserialize_with = "lenient")]
    #[schemars(extend("x-captain-group" = "Engine"), example = EngineChoice::External)]
    pub engine: Option<EngineChoice>,
    /// Stop Captain Engine when Captain quits. Applies at once.
    #[serde(deserialize_with = "lenient_true")]
    #[schemars(extend("x-captain-group" = "Engine"), example = false)]
    pub stop_engine_on_quit: bool,
    /// The CPUs, memory, and disk of Captain Engine. Set all three, or leave the key
    /// out for this computer's defaults: half the CPUs (2 to 8), a quarter of the
    /// memory (4 to 16 GiB), and a 64 GiB disk. Applies at the next engine start.
    #[serde(deserialize_with = "lenient")]
    #[schemars(
        extend("x-captain-group" = "Engine"),
        example = HostResources { cpus: 4, memory_bytes: 8 * crate::GIB, disk_bytes: 64 * crate::GIB }
    )]
    pub engine_resources: Option<HostResources>,
    /// Captain Engine's Docker daemon. See the keys below.
    // Feature 0020.
    #[serde(deserialize_with = "lenient")]
    #[schemars(extend("x-captain-group" = "Docker daemon"))]
    pub engine_daemon: DaemonSettings,
    /// The k3s cluster in Captain Engine. See the keys below.
    // ADR 0010.
    #[serde(deserialize_with = "lenient")]
    #[schemars(extend("x-captain-group" = "Kubernetes"))]
    pub kubernetes: KubernetesSettings,
    /// Launch with only the menu bar icon and no main window. It needs
    /// `show_menu_bar_icon`. Applies at the next launch.
    #[serde(deserialize_with = "lenient")]
    #[schemars(extend("x-captain-group" = "Startup"), example = true)]
    pub start_in_background: bool,
    /// Show Captain's icon in the menu bar (macOS) or the notification area
    /// (Windows). Applies at once.
    #[serde(deserialize_with = "lenient_true")]
    #[schemars(extend("x-captain-group" = "Startup"), example = false)]
    pub show_menu_bar_icon: bool,
    /// Write debug-level lines to Captain's log file. Applies at once.
    #[serde(deserialize_with = "lenient")]
    #[schemars(extend("x-captain-group" = "Diagnostics"), example = true)]
    pub debug_logging: bool,
    /// Once a week, while the engine runs, remove build cache older than 14 days.
    // Feature 0031.
    #[serde(deserialize_with = "lenient")]
    #[schemars(extend("x-captain-group" = "Storage"), example = true)]
    pub weekly_build_cache_cleanup: bool,
    /// When the weekly cleanup last ran, in Unix seconds. Captain sets it.
    #[serde(deserialize_with = "lenient")]
    #[schemars(extend("x-captain-group" = "Storage"), example = 1790000000)]
    pub build_cache_cleaned_at: Option<i64>,
    /// The `docker`, Compose, and `captain` links for your terminal. See the keys
    /// below.
    // Feature 0035.
    #[serde(deserialize_with = "lenient")]
    #[schemars(extend("x-captain-group" = "Terminal"))]
    pub command_line_tools: CliToolsSettings,
    /// What AI agents may see and do through `captain mcp`. See the keys below.
    // Feature 0038.
    #[serde(deserialize_with = "lenient")]
    #[schemars(extend("x-captain-group" = "AI agents"))]
    pub agent_tools: AgentToolsSettings,
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
            agent_tools: AgentToolsSettings::default(),
        }
    }
}

impl Settings {
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

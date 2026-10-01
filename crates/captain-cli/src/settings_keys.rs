//! The settings that `captain set` changes, checked the way the Settings page
//! checks them. See docs/features/0022-command-line.md.

use captain_core::settings::{EngineChoice, Settings};
use captain_core::{GIB, HostResources};
use clap::ValueEnum;

/// A key for `captain set` and `captain list-settings`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SettingKey {
    Engine,
    Cpus,
    Memory,
    Disk,
    StopEngineOnQuit,
    StartInBackground,
    ShowMenuBarIcon,
    DebugLogging,
}

/// What the checks need to know about this computer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Machine {
    pub cpus: u32,
    pub memory_bytes: u64,
    /// Whether Captain Engine can run here, for the default engine choice.
    pub captain_available: bool,
}

/// The resources for the next start: the saved ones, or this computer's defaults.
pub fn engine_resources(settings: &Settings, machine: &Machine) -> HostResources {
    settings
        .engine_resources
        .unwrap_or_else(|| HostResources::recommended(machine.cpus, machine.memory_bytes))
}

impl SettingKey {
    /// The name on the command line, for example `stop-engine-on-quit`.
    pub fn name(self) -> String {
        self.to_possible_value()
            .map(|value| value.get_name().to_string())
            .unwrap_or_default()
    }

    /// The key in the settings file that holds this setting.
    pub fn file_key(self) -> &'static str {
        match self {
            Self::Engine => "engine",
            Self::Cpus | Self::Memory | Self::Disk => "engine_resources",
            Self::StopEngineOnQuit => "stop_engine_on_quit",
            Self::StartInBackground => "start_in_background",
            Self::ShowMenuBarIcon => "show_menu_bar_icon",
            Self::DebugLogging => "debug_logging",
        }
    }

    /// True for the keys that apply on the next start of the engine.
    pub fn is_resource(self) -> bool {
        matches!(self, Self::Cpus | Self::Memory | Self::Disk)
    }

    /// The current value, in the form `set` takes.
    pub fn value(self, settings: &Settings, machine: &Machine) -> String {
        let resources = engine_resources(settings, machine);
        match self {
            Self::Engine => match settings.engine_choice(machine.captain_available) {
                EngineChoice::Captain => "captain".into(),
                EngineChoice::External => "external".into(),
            },
            Self::Cpus => resources.cpus.to_string(),
            Self::Memory => format!("{}GiB", resources.memory_gib()),
            Self::Disk => format!("{}GiB", resources.disk_gib()),
            Self::StopEngineOnQuit => settings.stop_engine_on_quit.to_string(),
            Self::StartInBackground => settings.start_in_background.to_string(),
            Self::ShowMenuBarIcon => settings.show_menu_bar_icon.to_string(),
            Self::DebugLogging => settings.debug_logging.to_string(),
        }
    }

    /// Checks `value` and puts it in `settings`, or says why it is wrong.
    pub fn apply(
        self,
        settings: &mut Settings,
        value: &str,
        machine: &Machine,
    ) -> Result<(), String> {
        let value = value.trim();
        let resources = engine_resources(settings, machine);
        match self {
            Self::Engine => settings.engine = Some(parse_engine(value)?),
            Self::Cpus => settings.engine_resources = Some(cpus(resources, value, machine)?),
            Self::Memory => settings.engine_resources = Some(memory(resources, value, machine)?),
            Self::Disk => settings.engine_resources = Some(disk(resources, value)?),
            Self::StopEngineOnQuit => settings.stop_engine_on_quit = parse_bool(value)?,
            Self::StartInBackground => {
                let on = parse_bool(value)?;
                if on && !settings.show_menu_bar_icon {
                    return Err("Start in the background needs the menu bar icon. \
                         Run `captain set show-menu-bar-icon true` first."
                        .into());
                }
                settings.start_in_background = on;
            }
            Self::ShowMenuBarIcon => settings.show_menu_bar_icon = parse_bool(value)?,
            Self::DebugLogging => settings.debug_logging = parse_bool(value)?,
        }
        Ok(())
    }
}

fn parse_engine(value: &str) -> Result<EngineChoice, String> {
    match value {
        "captain" => Ok(EngineChoice::Captain),
        "external" => Ok(EngineChoice::External),
        _ => Err(format!("Use captain or external, not {value:?}.")),
    }
}

fn parse_bool(value: &str) -> Result<bool, String> {
    value
        .parse()
        .map_err(|_| format!("Use true or false, not {value:?}."))
}

/// Whole GiB: `8`, `8G`, `8GB`, or `8GiB`.
fn parse_gib(value: &str) -> Result<u64, String> {
    let lower = value.to_ascii_lowercase();
    let number = ["gib", "gb", "g"]
        .iter()
        .find_map(|unit| lower.strip_suffix(unit))
        .unwrap_or(&lower);
    number
        .trim()
        .parse()
        .map_err(|_| format!("Use a whole number of GiB, such as 8, not {value:?}."))
}

/// The stepper's range: 1 to this computer's CPUs.
fn cpus(resources: HostResources, value: &str, machine: &Machine) -> Result<HostResources, String> {
    let cpus: u32 = value
        .parse()
        .map_err(|_| format!("Use a whole number of CPUs, not {value:?}."))?;
    let wanted = HostResources { cpus, ..resources };
    if wanted.step_cpus(0, machine.cpus) != wanted {
        return Err(format!("CPUs must be 1 to {}.", machine.cpus.max(1)));
    }
    Ok(wanted)
}

/// The stepper's range: 2 GiB to three quarters of this computer's memory.
fn memory(
    resources: HostResources,
    value: &str,
    machine: &Machine,
) -> Result<HostResources, String> {
    let gib = parse_gib(value)?;
    let wanted = HostResources {
        memory_bytes: gib.saturating_mul(GIB),
        ..resources
    };
    if wanted.step_memory(0, machine.memory_bytes) != wanted {
        let low = wanted.step_memory(i64::MIN / 2, machine.memory_bytes);
        let high = wanted.step_memory(i64::MAX / 2, machine.memory_bytes);
        return Err(format!(
            "Memory must be {} to {} GiB.",
            low.memory_gib(),
            high.memory_gib()
        ));
    }
    Ok(wanted)
}

/// The stepper's range, 16 GiB to 1 TiB. [`check_disk_floor`] checks the disk
/// that exists.
fn disk(resources: HostResources, value: &str) -> Result<HostResources, String> {
    let gib = parse_gib(value)?;
    let wanted = HostResources {
        disk_bytes: gib.saturating_mul(GIB),
        ..resources
    };
    if wanted.step_disk(0, None) != wanted {
        let low = wanted.step_disk(i64::MIN / 2, None).disk_gib();
        let high = wanted.step_disk(i64::MAX / 2, None).disk_gib();
        return Err(format!("Disk must be {low} to {high} GiB."));
    }
    Ok(wanted)
}

/// Refuses a disk smaller than Captain Engine's disk now, `current`, because a disk
/// cannot shrink. The stepper on the Settings page stops at the same size.
pub fn check_disk_floor(
    settings: &Settings,
    machine: &Machine,
    current: Option<u64>,
) -> Result<(), String> {
    let wanted = engine_resources(settings, machine);
    match current {
        Some(current) if wanted.step_disk(0, Some(current)) != wanted => {
            let now = wanted.step_disk(0, Some(current)).disk_gib();
            Err(format!(
                "Captain Engine's disk is {now} GiB, and a disk cannot shrink. Use {now} GiB or more."
            ))
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests;

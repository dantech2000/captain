use captain_core::GIB;
use captain_core::settings::{EngineChoice, Settings};

use super::{Machine, SettingKey, check_disk_floor, engine_resources};

const MACHINE: Machine = Machine {
    cpus: 8,
    memory_bytes: 32 * GIB,
    captain_available: true,
};

fn set(key: SettingKey, value: &str) -> Result<Settings, String> {
    let mut settings = Settings::default();
    key.apply(&mut settings, value, &MACHINE).map(|()| settings)
}

#[test]
fn cpus_stay_within_this_computer() {
    assert_eq!(
        set(SettingKey::Cpus, "8")
            .unwrap()
            .engine_resources
            .unwrap()
            .cpus,
        8
    );
    assert_eq!(
        set(SettingKey::Cpus, "9"),
        Err("CPUs must be 1 to 8.".into())
    );
    assert!(set(SettingKey::Cpus, "0").is_err());
    assert!(set(SettingKey::Cpus, "two").is_err());
    // A new resource keeps the other defaults.
    let saved = set(SettingKey::Cpus, "2")
        .unwrap()
        .engine_resources
        .unwrap();
    let defaults = engine_resources(&Settings::default(), &MACHINE);
    assert_eq!(
        (saved.memory_bytes, saved.disk_bytes),
        (defaults.memory_bytes, defaults.disk_bytes)
    );
}

#[test]
fn memory_takes_gib_up_to_three_quarters_of_the_computer() {
    let settings = set(SettingKey::Memory, "12GiB").unwrap();
    assert_eq!(settings.engine_resources.unwrap().memory_bytes, 12 * GIB);
    assert_eq!(
        set(SettingKey::Memory, "25"),
        Err("Memory must be 2 to 24 GiB.".into())
    );
    assert!(set(SettingKey::Memory, "1g").is_err());
}

#[test]
fn the_disk_cannot_shrink_below_the_engine_disk() {
    let settings = set(SettingKey::Disk, "48").unwrap();
    assert_eq!(check_disk_floor(&settings, &MACHINE, None), Ok(()));
    assert_eq!(
        check_disk_floor(&settings, &MACHINE, Some(32 * GIB)),
        Ok(())
    );
    assert_eq!(
        check_disk_floor(&settings, &MACHINE, Some(64 * GIB)),
        Err(
            "Captain Engine's disk is 64 GiB, and a disk cannot shrink. Use 64 GiB or more.".into()
        )
    );
    assert_eq!(
        set(SettingKey::Disk, "2048"),
        Err("Disk must be 16 to 1024 GiB.".into())
    );
}

#[test]
fn background_start_needs_the_menu_bar_icon() {
    let mut settings = Settings {
        show_menu_bar_icon: false,
        ..Settings::default()
    };
    let key = SettingKey::StartInBackground;
    assert!(key.apply(&mut settings, "true", &MACHINE).is_err());
    settings.show_menu_bar_icon = true;
    assert!(key.apply(&mut settings, "true", &MACHINE).is_ok());
    assert!(settings.start_in_background);
}

#[test]
fn flags_and_the_engine_take_only_known_words() {
    assert!(
        !set(SettingKey::StopEngineOnQuit, "false")
            .unwrap()
            .stop_engine_on_quit
    );
    assert!(set(SettingKey::DebugLogging, "yes").is_err());
    let settings = set(SettingKey::Engine, "external").unwrap();
    assert_eq!(settings.engine, Some(EngineChoice::External));
    assert!(set(SettingKey::Engine, "docker").is_err());
}

#[test]
fn values_read_back_in_the_form_set_takes() {
    let settings = set(SettingKey::Memory, "12").unwrap();
    assert_eq!(SettingKey::Memory.value(&settings, &MACHINE), "12GiB");
    assert_eq!(SettingKey::Engine.value(&settings, &MACHINE), "captain");
    assert_eq!(SettingKey::StopEngineOnQuit.name(), "stop-engine-on-quit");
}

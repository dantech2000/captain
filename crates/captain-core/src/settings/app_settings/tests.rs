use super::{SETTINGS_VERSION, Settings};
use crate::daemon::DaemonSettings;
use crate::settings::{Appearance, EngineChoice, ThemeFamily};
use crate::{GIB, HostResources};

#[test]
fn missing_fields_get_defaults() {
    let settings = Settings::from_json(r#"{"version": 1, "theme": "harbor"}"#).unwrap();
    assert_eq!(settings.theme, ThemeFamily::Harbor);
    assert_eq!(settings.appearance, Appearance::System);
    assert_eq!(settings.engine_endpoint, None);
}

#[test]
fn unknown_fields_are_ignored() {
    let json = r#"{"version": 1, "appearance": "light", "window": {"width": 900}}"#;
    let settings = Settings::from_json(json).unwrap();
    assert_eq!(settings.appearance, Appearance::Light);
}

#[test]
fn unknown_values_fall_back_to_defaults() {
    let json = r#"{"appearance": "sepia", "theme": 7, "engine_endpoint": false}"#;
    let settings = Settings::from_json(json).unwrap();
    assert_eq!(settings.appearance, Appearance::System);
    assert_eq!(settings.theme, ThemeFamily::Dusk);
    assert_eq!(settings.engine_endpoint, None);
}

#[test]
fn a_file_with_the_old_accent_gets_the_default_theme() {
    let json = r#"{"version": 1, "appearance": "dark", "accent": "purple"}"#;
    let settings = Settings::from_json(json).unwrap();
    assert_eq!(settings.appearance, Appearance::Dark);
    assert_eq!(settings.theme, ThemeFamily::Dusk);
    assert!(!settings.to_json().contains("accent"));
}

#[test]
fn blank_endpoint_is_none() {
    let settings = Settings::from_json(r#"{"engine_endpoint": "  "}"#).unwrap();
    assert_eq!(settings.engine_endpoint, None);
}

#[test]
fn malformed_json_is_an_error() {
    assert!(Settings::from_json("{").is_err());
}

#[test]
fn writes_the_current_version() {
    let old = Settings {
        version: 0,
        ..Settings::default()
    };
    let written = Settings::from_json(&old.to_json()).unwrap();
    assert_eq!(written.version, SETTINGS_VERSION);
}

#[test]
fn writes_lowercase_names() {
    let json = Settings {
        theme: ThemeFamily::Periwinkle,
        ..Settings::default()
    }
    .to_json();
    assert!(json.contains(r#""theme": "periwinkle""#), "{json}");
    assert!(json.contains(r#""appearance": "system""#), "{json}");
}

#[test]
fn round_trips() {
    let settings = Settings {
        appearance: Appearance::Dark,
        theme: ThemeFamily::Harbor,
        debug_logging: true,
        engine: Some(EngineChoice::External),
        stop_engine_on_quit: false,
        engine_resources: Some(HostResources {
            cpus: 6,
            memory_bytes: 8 * GIB,
            disk_bytes: 100 * GIB,
        }),
        engine_daemon: DaemonSettings {
            registry_mirrors: vec!["https://mirror.gcr.io".into()],
            custom: serde_json::json!({"log-level": "warn"})
                .as_object()
                .cloned()
                .unwrap(),
            tcp: true,
            ..DaemonSettings::default()
        },
        ..Settings::default()
    };
    let json = settings.to_json();
    assert!(json.contains(r#""engine": "external""#), "{json}");
    assert_eq!(Settings::from_json(&json).unwrap(), settings);
}

#[test]
fn engine_choice_defaults_to_captain_when_it_can_run() {
    let settings = Settings::default();
    assert_eq!(settings.engine_choice(true), EngineChoice::Captain);
    assert_eq!(settings.engine_choice(false), EngineChoice::External);
}

#[test]
fn a_saved_endpoint_keeps_the_other_engine() {
    let settings = Settings {
        engine_endpoint: Some("unix:///var/run/docker.sock".into()),
        ..Settings::default()
    };
    assert_eq!(settings.engine_choice(true), EngineChoice::External);
}

#[test]
fn a_saved_choice_wins() {
    let settings = Settings {
        engine: Some(EngineChoice::Captain),
        engine_endpoint: Some("tcp://10.0.0.5:2375".into()),
        ..Settings::default()
    };
    assert_eq!(settings.engine_choice(false), EngineChoice::Captain);
}

#[test]
fn old_files_get_the_behavior_defaults() {
    let settings = Settings::from_json(r#"{"version": 1, "show_menu_bar_icon": "no"}"#).unwrap();
    assert!(!settings.start_in_background);
    assert!(settings.show_menu_bar_icon);
}

#[test]
fn background_launch_needs_the_menu_bar_icon() {
    let settings = Settings {
        start_in_background: true,
        ..Settings::default()
    };
    assert!(!settings.opens_window_at_launch(true));
    assert!(settings.opens_window_at_launch(false));
    assert!(Settings::default().opens_window_at_launch(true));
}

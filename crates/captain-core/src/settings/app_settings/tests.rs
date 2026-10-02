use super::Settings;
use crate::daemon::DaemonSettings;
use crate::settings::{Appearance, EngineChoice, ThemeFamily};
use crate::{GIB, HostResources};

#[test]
fn old_and_partial_files_get_defaults() {
    let settings = Settings::from_json(
        r#"{"version": 1, "appearance": "light", "accent": "purple", "show_menu_bar_icon": "no", "window": {"width": 900}}"#,
    )
    .unwrap();
    assert_eq!(settings.appearance, Appearance::Light);
    assert_eq!(settings.theme, ThemeFamily::Dusk);
    assert_eq!(settings.engine_endpoint, None);
    assert!(!settings.start_in_background);
    assert!(settings.show_menu_bar_icon);
    assert!(!settings.show_extension_containers);
    let settings = Settings::from_json(r#"{"version": 1, "theme": "harbor"}"#).unwrap();
    assert_eq!(settings.theme, ThemeFamily::Harbor);
    assert_eq!(settings.appearance, Appearance::System);
}

#[test]
fn bad_values_fall_back_to_defaults_and_bad_json_is_an_error() {
    let json = r#"{"appearance": "sepia", "theme": 7, "engine_endpoint": false}"#;
    let settings = Settings::from_json(json).unwrap();
    assert_eq!(settings.appearance, Appearance::System);
    assert_eq!(settings.theme, ThemeFamily::Dusk);
    assert_eq!(settings.engine_endpoint, None);
    let settings = Settings::from_json(r#"{"engine_endpoint": "  "}"#).unwrap();
    assert_eq!(settings.engine_endpoint, None);
    assert!(Settings::from_json("{").is_err());
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
    let json = serde_json::to_string(&settings).unwrap();
    assert!(json.contains(r#""engine":"external""#), "{json}");
    assert_eq!(Settings::from_json(&json).unwrap(), settings);
}

#[test]
fn engine_choice_is_the_saved_choice_then_a_saved_endpoint_then_captain_if_it_can_run() {
    let settings = Settings::default();
    assert_eq!(settings.engine_choice(true), EngineChoice::Captain);
    assert_eq!(settings.engine_choice(false), EngineChoice::External);
    let settings = Settings {
        engine_endpoint: Some("unix:///var/run/docker.sock".into()),
        ..Settings::default()
    };
    assert_eq!(settings.engine_choice(true), EngineChoice::External);
    let settings = Settings {
        engine: Some(EngineChoice::Captain),
        engine_endpoint: Some("tcp://10.0.0.5:2375".into()),
        ..Settings::default()
    };
    assert_eq!(settings.engine_choice(false), EngineChoice::Captain);
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

#[test]
fn projects_dir_expands_the_home_folder() {
    let home = std::path::Path::new("/Users/me");
    assert_eq!(
        Settings::default().projects_dir_in(home),
        home.join("Captain")
    );
    let settings = Settings::from_json(r#"{"projects_dir": "/srv/projects"}"#).unwrap();
    assert_eq!(
        settings.projects_dir_in(home),
        std::path::PathBuf::from("/srv/projects")
    );
    let settings = Settings::from_json(r#"{"projects_dir": "projects"}"#).unwrap();
    assert_eq!(settings.projects_dir_in(home), home.join("projects"));
}

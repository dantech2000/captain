use super::{SETTINGS_VERSION, Settings};
use crate::settings::{Accent, Appearance};

#[test]
fn round_trips() {
    let settings = Settings {
        appearance: Appearance::Dark,
        accent: Accent::Teal,
        engine_endpoint: Some("tcp://10.0.0.5:2375".into()),
        ..Settings::default()
    };
    assert_eq!(Settings::from_json(&settings.to_json()).unwrap(), settings);
}

#[test]
fn empty_object_is_the_default() {
    assert_eq!(Settings::from_json("{}").unwrap(), Settings::default());
}

#[test]
fn missing_fields_get_defaults() {
    let settings = Settings::from_json(r#"{"version": 1, "accent": "purple"}"#).unwrap();
    assert_eq!(settings.accent, Accent::Purple);
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
    let json = r#"{"appearance": "sepia", "accent": 7, "engine_endpoint": false}"#;
    let settings = Settings::from_json(json).unwrap();
    assert_eq!(settings.appearance, Appearance::System);
    assert_eq!(settings.accent, Accent::Blue);
    assert_eq!(settings.engine_endpoint, None);
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
        accent: Accent::Graphite,
        ..Settings::default()
    }
    .to_json();
    assert!(json.contains(r#""accent": "graphite""#), "{json}");
    assert!(json.contains(r#""appearance": "system""#), "{json}");
}

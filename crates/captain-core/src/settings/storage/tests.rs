use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use super::{Settings, SettingsError};
use crate::settings::{Appearance, ThemeFamily};

/// A fresh directory under the system temp dir, removed when dropped.
struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let unique = format!(
            "captain-settings-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        );
        let dir = std::env::temp_dir().join(unique);
        fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn missing_file_gives_defaults() {
    let dir = TempDir::new();
    let settings = Settings::load(&dir.0.join("settings.json")).unwrap();
    assert_eq!(settings, Settings::default());
}

#[test]
fn save_then_load_leaves_no_temp_file() {
    let dir = TempDir::new();
    let path = dir.0.join("Captain").join("settings.json");
    let settings = Settings {
        appearance: Appearance::Light,
        theme: ThemeFamily::Periwinkle,
        engine_endpoint: Some("unix:///tmp/engine.sock".into()),
        ..Settings::default()
    };
    settings.save(&path).unwrap();
    assert_eq!(Settings::load(&path).unwrap(), settings);
    let names: Vec<_> = fs::read_dir(path.parent().unwrap())
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(names, ["settings.json"]);
}

#[test]
fn a_file_with_bad_json_is_never_overwritten() {
    let dir = TempDir::new();
    let path = dir.0.join("settings.json");
    fs::write(
        &path,
        "{\n  \"theme\": \"harbor\"\n  \"appearance\": \"dark\"\n}\n",
    )
    .unwrap();
    let Err(SettingsError::Parse { problem, .. }) = Settings::load(&path) else {
        panic!("expected a parse error");
    };
    assert_eq!(
        problem.to_string().split(". See").next(),
        Some("settings.json line 2: Expected comma")
    );
    let dark = Settings {
        appearance: Appearance::Dark,
        ..Settings::default()
    };
    assert!(dark.save(&path).is_err());
    assert!(fs::read_to_string(&path).unwrap().contains("harbor"));
}

/// A file from format 1 held every key. The migration keeps only the changed ones
/// and saves the old file once.
#[test]
fn migration_drops_defaults_and_keeps_one_backup() {
    let dir = TempDir::new();
    let path = dir.0.join("settings.json");
    let mut old = serde_json::to_value(Settings {
        theme: ThemeFamily::Harbor,
        ..Settings::default()
    })
    .unwrap();
    old["version"] = 1.into();
    old["accent"] = "purple".into();
    let old = serde_json::to_string_pretty(&old).unwrap();
    fs::write(&path, &old).unwrap();

    Settings::migrate_file(&path).unwrap();
    let text = fs::read_to_string(&path).unwrap();
    assert_eq!(
        text,
        "{\n  \"$schema\": \"./settings.schema.json\",\n  \"version\": 2,\n  \"theme\": \"harbor\",\n  \"accent\": \"purple\"\n}"
    );
    let loaded = Settings::read(&path).unwrap();
    assert!(loaded.sets("theme") && !loaded.sets("appearance"));
    let backup = dir.0.join("settings.json.captain-backup");
    assert_eq!(fs::read_to_string(&backup).unwrap(), old);

    fs::write(&path, old.replace("harbor", "dusk")).unwrap();
    Settings::migrate_file(&path).unwrap();
    assert_eq!(fs::read_to_string(&backup).unwrap(), old);
}

#[test]
fn directory_in_place_of_file_is_an_io_error() {
    let dir = TempDir::new();
    assert!(matches!(
        Settings::load(&dir.0),
        Err(SettingsError::Io { .. })
    ));
}

#[test]
fn prepare_creates_a_starter_file_whose_examples_are_off_until_uncommented() {
    let dir = TempDir::new();
    let path = dir.0.join("settings.json");
    Settings::prepare_file(&path).unwrap();
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains("\"$schema\""));
    assert_eq!(Settings::load(&path).unwrap(), Settings::default());

    fs::write(&path, text.replace("// \"theme\"", "\"theme\"")).unwrap();
    assert_eq!(Settings::load(&path).unwrap().theme, ThemeFamily::Harbor);

    // A second start leaves the file, and the user's edit, as they are.
    let edited = fs::read_to_string(&path).unwrap();
    Settings::prepare_file(&path).unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), edited);
}

#[test]
fn a_change_from_the_app_keeps_the_starter_comments() {
    let dir = TempDir::new();
    let path = dir.0.join("settings.json");
    Settings::prepare_file(&path).unwrap();
    let before = Settings::default();
    let after = Settings {
        theme: ThemeFamily::Periwinkle,
        ..Settings::default()
    };
    Settings::save_change(&path, &before, &after).unwrap();
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains("// \"stop_engine_on_quit\": false,"), "{text}");
    assert_eq!(Settings::load(&path).unwrap().theme, ThemeFamily::Periwinkle);
}

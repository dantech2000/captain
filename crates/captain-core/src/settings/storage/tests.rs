use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use super::{Settings, SettingsError};
use crate::settings::{Accent, Appearance};

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
fn save_then_load() {
    let dir = TempDir::new();
    let path = dir.0.join("Captain").join("settings.json");
    let settings = Settings {
        appearance: Appearance::Light,
        accent: Accent::Orange,
        engine_endpoint: Some("unix:///tmp/engine.sock".into()),
        ..Settings::default()
    };
    settings.save(&path).unwrap();
    assert_eq!(Settings::load(&path).unwrap(), settings);
}

#[test]
fn save_replaces_the_file_and_leaves_no_temp_file() {
    let dir = TempDir::new();
    let path = dir.0.join("settings.json");
    fs::write(&path, "old contents").unwrap();
    Settings::default().save(&path).unwrap();

    let names: Vec<_> = fs::read_dir(&dir.0)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(names, ["settings.json"]);
    assert_eq!(Settings::load(&path).unwrap(), Settings::default());
}

#[test]
fn malformed_file_is_a_parse_error() {
    let dir = TempDir::new();
    let path = dir.0.join("settings.json");
    fs::write(&path, "not json").unwrap();
    assert!(matches!(
        Settings::load(&path),
        Err(SettingsError::Parse { .. })
    ));
}

#[test]
fn directory_in_place_of_file_is_an_io_error() {
    let dir = TempDir::new();
    assert!(matches!(
        Settings::load(&dir.0),
        Err(SettingsError::Io { .. })
    ));
}

use std::path::PathBuf;

use captain_core::settings::{Accent, Appearance, Settings};
use gpui_kit::component::{Theme, ThemeMode};
use gpui_kit::*;

/// The loaded settings and the file they save to. Views read them with [`current`]
/// and change them with [`update`], which saves at once.
#[derive(Default)]
pub struct SettingsStore {
    settings: Settings,
    /// Where [`update`] writes. `None` keeps changes in memory only.
    path: Option<PathBuf>,
    /// Why the last save failed, for the Settings page.
    save_error: Option<SharedString>,
}

impl Global for SettingsStore {}

/// Installs the settings that the app loaded from `path`. Call it before opening a
/// window. Without it, views see the defaults.
pub fn init(cx: &mut App, settings: Settings, path: Option<PathBuf>) {
    cx.set_global(SettingsStore {
        settings,
        path,
        save_error: None,
    });
}

/// The current settings, or the defaults before [`init`].
pub fn current(cx: &App) -> Settings {
    cx.try_global::<SettingsStore>()
        .map(|store| store.settings.clone())
        .unwrap_or_default()
}

/// The accent preset. [`crate::theme::Palette::of`] reads it on every render, so it
/// skips the copy that [`current`] makes.
pub fn accent(cx: &App) -> Accent {
    cx.try_global::<SettingsStore>()
        .map(|store| store.settings.accent)
        .unwrap_or_default()
}

/// Calls `f` after each change to the settings. The app uses it to follow the menu
/// bar icon switch.
pub fn observe(cx: &mut App, f: impl FnMut(&mut App) + 'static) -> Subscription {
    cx.observe_global::<SettingsStore>(f)
}

/// Why the last save failed, if it did.
pub fn save_error(cx: &App) -> Option<SharedString> {
    cx.try_global::<SettingsStore>()
        .and_then(|store| store.save_error.clone())
}

/// Changes the settings, saves them, and applies the appearance. Every window
/// redraws, so a new accent shows at once.
pub fn update(cx: &mut App, change: impl FnOnce(&mut Settings)) {
    let store = cx.default_global::<SettingsStore>();
    let before = store.settings.clone();
    change(&mut store.settings);
    if store.settings == before {
        return;
    }
    if let Some(path) = &store.path {
        store.save_error = match store.settings.save(path) {
            Ok(()) => None,
            Err(error) => {
                tracing::warn!(%error, "cannot save settings");
                Some(error.to_string().into())
            }
        };
    }
    if store.settings.appearance != before.appearance {
        apply_appearance(None, cx);
    }
    cx.refresh_windows();
}

/// Sets light or dark mode from the settings. `System` follows the window, or the
/// OS when there is no window.
pub fn apply_appearance(window: Option<&mut Window>, cx: &mut App) {
    match current(cx).appearance {
        Appearance::System => Theme::sync_system_appearance(window, cx),
        Appearance::Light => Theme::change(ThemeMode::Light, window, cx),
        Appearance::Dark => Theme::change(ThemeMode::Dark, window, cx),
    }
}

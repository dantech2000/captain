use std::path::PathBuf;

use captain_core::process_lock::{ProcessLock, settings_lock_path};
use captain_core::settings::{Appearance, Settings, ThemeFamily};
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

/// The color theme. [`crate::theme::Palette::of`] reads it on every render, so it
/// skips the copy that [`current`] makes.
pub fn theme_family(cx: &App) -> ThemeFamily {
    cx.try_global::<SettingsStore>()
        .map(|store| store.settings.theme)
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
/// redraws, so a new theme shows at once.
pub fn update(cx: &mut App, change: impl FnOnce(&mut Settings)) {
    let store = cx.default_global::<SettingsStore>();
    let before = store.settings.clone();
    change(&mut store.settings);
    if store.settings == before {
        return;
    }
    if let Some(path) = &store.path {
        // The `captain` CLI changes the file under the same lock.
        let _lock = ProcessLock::acquire(&settings_lock_path(path))
            .inspect_err(|error| tracing::warn!(%error, "cannot lock the settings"));
        store.save_error = match store.settings.save(path) {
            Ok(()) => None,
            Err(error) => {
                tracing::warn!(%error, "cannot save settings");
                Some(error.to_string().into())
            }
        };
    }
    let settings = &store.settings;
    if settings.appearance != before.appearance || settings.theme != before.theme {
        apply_appearance(None, cx);
    }
    cx.refresh_windows();
}

/// Sets the theme and light or dark mode from the settings, for Captain's views and
/// gpui-kit's components. `System` follows the window, or the OS when there is no
/// window.
pub fn apply_appearance(window: Option<&mut Window>, cx: &mut App) {
    crate::theme::install_kit_themes(theme_family(cx), cx);
    match current(cx).appearance {
        Appearance::System => Theme::sync_system_appearance(window, cx),
        Appearance::Light => Theme::change(ThemeMode::Light, window, cx),
        Appearance::Dark => Theme::change(ThemeMode::Dark, window, cx),
    }
}

//! Captain: a native desktop client for Docker.

mod actions;
mod connect;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod tray;
mod window;

use std::path::{Path, PathBuf};
use std::rc::Rc;

use captain_core::settings::Settings;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "captain=info,captain_docker=info".into()),
        )
        .init();

    let path = settings_path();
    let settings = load_settings(path.as_deref());
    let endpoint = settings.engine_endpoint.clone();

    let app = gpui_kit::application().with_assets(gpui_kit::assets::AllAssets);
    // Clicking the Dock icon with no window open brings the window back.
    app.on_reopen(window::show);
    app.run(move |cx| {
        gpui_kit::init(cx);
        captain_ui::settings_init(cx, settings, path);
        captain_ui::engine_source_init(cx, Rc::new(connect::DockerSource));
        captain_ui::palette_init(cx);
        actions::register(cx);
        window::init(endpoint, cx);
        // The app has launched, so the platform run loop is up; see ADR 0006.
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        tray::start(cx);
        cx.activate(true);
    });
}

/// `Captain/settings.json` in the user's config directory, for example
/// `~/Library/Application Support/Captain/settings.json` on macOS. See ADR 0004.
fn settings_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("Captain").join("settings.json"))
}

/// The saved settings, or the defaults when there is no file or it cannot be read.
fn load_settings(path: Option<&Path>) -> Settings {
    let Some(path) = path else {
        return Settings::default();
    };
    Settings::load(path).unwrap_or_else(|error| {
        tracing::warn!(%error, "cannot read the settings; using the defaults");
        Settings::default()
    })
}

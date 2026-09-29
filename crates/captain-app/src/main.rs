//! Captain: a native desktop client for Docker.

mod actions;
mod connect;
mod diagnostics;
#[cfg(unix)]
mod docker_socket;
mod engine;
mod logging;
mod login_item;
mod quit;
mod system;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod tray;
mod window;

use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

use captain_core::process_lock::{ProcessLock, app_lock_path};
use captain_core::settings::Settings;

fn main() {
    let logging = logging::init();
    let path = settings_path();
    // Held until Captain exits; `captain set` refuses while it is held.
    let _app_lock = path.as_deref().and_then(app_lock);
    let settings = load_settings(path.as_deref());
    if settings.debug_logging {
        logging.set_debug(true);
    }
    let diagnostics = diagnostics::setup(&logging);
    let endpoint = settings.engine_endpoint.clone();
    let engine = engine::EngineSetup::new(&settings);

    let app = gpui_kit::application().with_assets(gpui_kit::assets::AllAssets);
    // Clicking the Dock icon with no window open brings the window back.
    app.on_reopen(window::show);
    app.run(move |cx| {
        gpui_kit::init(cx);
        captain_ui::settings_init(cx, settings, path);
        captain_ui::engine_source_init(cx, Rc::new(connect::DockerSource));
        captain_ui::OpenMigrationAssistant::set_backend(
            cx,
            Arc::new(captain_docker::DockerMigrator),
        );
        captain_ui::system_init(cx, Arc::new(system::System));
        captain_ui::palette_init(cx);
        actions::register(cx);
        window::init(endpoint, engine, diagnostics, cx);
        // The app has launched, so the platform run loop is up; see ADR 0006.
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        tray::follow_settings(cx);
        // Without the icon, a background start would leave Captain out of reach.
        if captain_ui::current_settings(cx).opens_window_at_launch(actions::has_tray(cx)) {
            window::show(cx);
        } else {
            tracing::info!("starting in the background");
        }
    });
}

/// `Captain/settings.json` in the user's config directory, for example
/// `~/Library/Application Support/Captain/settings.json` on macOS. See ADR 0004.
fn settings_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("Captain").join("settings.json"))
}

/// Locks `app.lock` next to the settings file, so the `captain` CLI knows that the
/// app runs and holds the settings. See docs/features/0022-command-line.md.
fn app_lock(settings: &Path) -> Option<ProcessLock> {
    let path = app_lock_path(settings);
    ProcessLock::try_acquire(&path, "running")
        .inspect_err(|error| tracing::warn!(%error, "cannot lock {}", path.display()))
        .ok()
        .flatten()
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

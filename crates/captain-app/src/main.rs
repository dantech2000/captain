//! Captain: a native desktop client for Docker.

mod actions;
mod cli_tools;
mod connect;
mod diagnostics;
#[cfg(target_os = "macos")]
mod dock_badge;
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

use captain_core::process_lock::{
    CLI_RESTORE_NOTE, ProcessLock, app_lock_path, settings_lock_path,
};
use captain_core::settings::Settings;

fn main() {
    refuse_cli_arguments();
    let logging = logging::init();
    let path = settings_path();
    // Held until Captain exits; `captain set` refuses while it is held.
    let _app_lock = path.as_deref().and_then(app_lock);
    let settings = load_settings(path.as_deref());
    if settings.debug_logging {
        logging.set_debug(true);
    }
    let diagnostics = diagnostics::setup(&logging);
    cli_tools::refresh(&settings.command_line_tools);
    let endpoint = settings.engine_endpoint.clone();
    let engine = engine::EngineSetup::new(&settings);

    let app = gpui_kit::application().with_assets(captain_ui::CaptainAssets);
    // Clicking the Dock icon with no window open brings the window back.
    app.on_reopen(window::show);
    app.run(move |cx| {
        gpui_kit::init(cx);
        captain_ui::settings_init(cx, settings, path);
        captain_ui::settings_watch_init(cx);
        captain_ui::engine_source_init(cx, Rc::new(connect::DockerSource));
        captain_ui::OpenMigrationAssistant::set_backend(
            cx,
            Arc::new(captain_docker::DockerMigrator),
        );
        captain_ui::system_init(cx, Arc::new(system::System));
        captain_ui::palette_init(cx);
        actions::register(cx);
        // A tunnel to an ssh:// engine must not outlive Captain; see feature 0026.
        cx.on_app_quit(|_| {
            captain_docker::close_ssh_tunnel();
            async {}
        })
        .detach();
        window::init(endpoint, engine, diagnostics, cx);
        #[cfg(target_os = "macos")]
        dock_badge::follow(cx);
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

/// Exits if Captain was started with command-line arguments. The app binary is
/// named `captain` like the CLI, and running it with CLI arguments by mistake would
/// start the full app on the real engine. macOS may pass `-psn_…` and `-NS…`.
fn refuse_cli_arguments() {
    let unexpected = std::env::args()
        .skip(1)
        .find(|arg| !arg.starts_with("-psn_") && !arg.starts_with("-NS"));
    if let Some(arg) = unexpected {
        eprintln!(
            "This is the Captain app, which takes no arguments (got {arg:?}). \
             For the command line, run captain-cli, or Captain.app/Contents/Resources/bin/captain."
        );
        std::process::exit(2);
    }
}

/// `Captain/settings.json` in the user's config directory, for example
/// `~/Library/Application Support/Captain/settings.json` on macOS. See ADR 0004.
fn settings_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("Captain").join("settings.json"))
}

/// Locks `app.lock` next to the settings file, so the `captain` CLI knows that the
/// app runs and holds the settings. See docs/features/0022-command-line.md. Exits
/// when another Captain holds it, because two apps would overwrite each other's
/// settings. Any other failure only logs.
fn app_lock(settings: &Path) -> Option<ProcessLock> {
    let path = app_lock_path(settings);
    // A `captain status` probe holds the lock for an instant, so try a few times.
    for _ in 0..10 {
        match ProcessLock::try_acquire(&path, "running") {
            Ok(Some(lock)) => return Some(lock),
            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(100)),
            Err(error) => {
                tracing::warn!(%error, "cannot lock {}", path.display());
                return None;
            }
        }
    }
    if ProcessLock::holder(&path).as_deref() == Some(CLI_RESTORE_NOTE) {
        tracing::warn!("the captain CLI is restoring a snapshot; quitting");
        eprintln!("The captain command is restoring a snapshot. Open Captain when it finishes.");
        std::process::exit(1);
    }
    tracing::warn!("another Captain is running; quitting");
    eprintln!("Captain is already running.");
    std::process::exit(0);
}

/// The saved settings, or the defaults when there is no file or it cannot be read.
/// A missing file is created as a starter file with examples in comments, a file
/// from before format 2 drops its default values first, and the schema for
/// editors goes next to it. See ADR 0013.
fn load_settings(path: Option<&Path>) -> Settings {
    let Some(path) = path else {
        return Settings::default();
    };
    // The `captain` CLI changes the file under this lock.
    let _lock = ProcessLock::acquire(&settings_lock_path(path))
        .inspect_err(|error| tracing::warn!(%error, "cannot lock the settings"));
    if let Err(error) = Settings::prepare_file(path) {
        tracing::warn!(%error, "cannot create or migrate the settings file");
    }
    if let Err(error) = captain_core::settings::write_schema(path) {
        tracing::warn!(%error, "cannot write the settings schema");
    }
    Settings::load(path).unwrap_or_else(|error| {
        tracing::warn!(%error, "cannot read the settings; using the defaults");
        Settings::default()
    })
}

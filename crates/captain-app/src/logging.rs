//! Logs go to stderr and to a file that rotates by size. The Diagnostics page shows
//! the folder and switches debug logging on and off. See
//! docs/features/0016-diagnostics.md.

mod rotating_file;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry, fmt, reload};

use rotating_file::RotatingFile;

/// Captain's crates at the normal level.
const NORMAL: &str = "captain=info,captain_ui=info,captain_docker=info,captain_host=info";
/// Captain's crates with debug logging on.
const DEBUG: &str =
    "captain=debug,captain_ui=debug,captain_core=debug,captain_docker=debug,captain_host=debug";

/// The installed logger. It changes the level while Captain runs.
#[derive(Clone)]
pub struct Logging {
    filter: reload::Handle<EnvFilter, Registry>,
    /// The folder with the log files, if Captain could create it.
    pub dir: Option<PathBuf>,
}

/// Starts logging to stderr and, if the folder can be created, to `captain.log`.
pub fn init() -> Logging {
    let (filter, handle) = reload::Layer::new(filter(false));
    let dir = log_dir().filter(|dir| std::fs::create_dir_all(dir).is_ok());
    let file = dir
        .as_ref()
        .and_then(|dir| RotatingFile::open(dir.join("captain.log")).ok())
        .map(|file| fmt::layer().with_ansi(false).with_writer(Mutex::new(file)));
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer())
        .with(file)
        .init();
    Logging {
        filter: handle,
        dir,
    }
}

impl Logging {
    /// Switches debug logging on or off.
    pub fn set_debug(&self, on: bool) {
        if let Err(error) = self.filter.reload(filter(on)) {
            tracing::warn!(%error, "cannot change the log level");
        }
        tracing::info!(debug_logging = on, "log level changed");
    }

    /// [`Logging::set_debug`] as a callback for the UI.
    pub fn debug_switch(&self) -> Arc<dyn Fn(bool) + Send + Sync> {
        let logging = self.clone();
        Arc::new(move |debug| logging.set_debug(debug))
    }
}

/// `RUST_LOG` wins over the normal level; debug logging wins over both.
fn filter(debug: bool) -> EnvFilter {
    if debug {
        return EnvFilter::new(DEBUG);
    }
    EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(NORMAL))
}

/// `~/Library/Logs/Captain` on macOS, where Console.app finds it; elsewhere
/// `Captain/logs` in the local data folder.
fn log_dir() -> Option<PathBuf> {
    if cfg!(target_os = "macos") {
        dirs::home_dir().map(|home| home.join("Library/Logs/Captain"))
    } else {
        dirs::data_local_dir().map(|dir| dir.join("Captain").join("logs"))
    }
}

//! [`LimaHost`]: Captain Engine as one Lima instance named `captain`. See ADR 0008.

mod steps;

use std::path::{Path, PathBuf};
use std::process::Child;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, MutexGuard};

use captain_core::{EngineHost, HostError, HostFuture, HostResources, HostStatus, HostStream};
use futures::StreamExt;
use futures::channel::mpsc;

use super::limactl::Limactl;
use super::locate::locate_limactl;
use super::paths::LimaPaths;
use super::version::check_version;
use crate::blocking::blocking;

/// A start or a stop that Captain is running now. Lima's own status lags behind it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Idle,
    Starting,
    Stopping,
}

/// Captain Engine on Lima.
#[derive(Clone)]
pub struct LimaHost {
    inner: Arc<Inner>,
}

struct Inner {
    paths: LimaPaths,
    /// Where Captain runs from, to find a bundled `limactl`.
    exe: Option<PathBuf>,
    /// Rosetta for x86_64 containers; Apple Silicon only.
    rosetta: bool,
    phase: Mutex<Phase>,
    resources: Mutex<HostResources>,
    /// The `limactl` that passed the version check.
    checked: Mutex<Option<PathBuf>>,
    /// The `limactl` command that is running now, so a stop can kill a start.
    running: Mutex<Option<Child>>,
    /// Set by a stop, so a start does not run its next step.
    cancel: AtomicBool,
}

impl LimaHost {
    /// Captain Engine in `~/.captain/lima`, with `resources` for the next start.
    pub fn new(resources: HostResources) -> Self {
        let home = std::env::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        Self::with_paths(LimaPaths::for_home(&home), resources)
    }

    /// Captain Engine with its files in `paths`. The live test uses a temporary folder.
    pub fn with_paths(paths: LimaPaths, resources: HostResources) -> Self {
        Self {
            inner: Arc::new(Inner {
                paths,
                exe: std::env::current_exe().ok(),
                rosetta: cfg!(target_arch = "aarch64"),
                phase: Mutex::new(Phase::Idle),
                resources: Mutex::new(resources),
                checked: Mutex::new(None),
                running: Mutex::new(None),
                cancel: AtomicBool::new(false),
            }),
        }
    }

    /// True if `limactl` exists on this computer. It does not check the version.
    pub fn is_installed() -> bool {
        let exe = std::env::current_exe().ok();
        let path = std::env::var_os("PATH");
        locate_limactl(exe.as_deref(), path.as_deref(), Path::is_file).is_some()
    }

    pub fn paths(&self) -> &LimaPaths {
        &self.inner.paths
    }
}

impl Inner {
    /// A checked `limactl`, or why there is none.
    fn limactl(&self) -> Result<Limactl, String> {
        let make = |binary: PathBuf| Limactl::new(binary, self.paths.lima_home.clone());
        if let Some(binary) = lock(&self.checked).clone() {
            return Ok(make(binary));
        }
        let path = std::env::var_os("PATH");
        let binary = locate_limactl(self.exe.as_deref(), path.as_deref(), Path::is_file)
            .ok_or_else(|| {
                "Captain Engine needs Lima. Install it with `brew install lima`.".to_string()
            })?;
        let output = std::process::Command::new(&binary)
            .arg("--version")
            .output()
            .map_err(|error| format!("Cannot run {}: {error}", binary.display()))?;
        check_version(&String::from_utf8_lossy(&output.stdout))?;
        *lock(&self.checked) = Some(binary.clone());
        Ok(make(binary))
    }

    fn phase(&self) -> Phase {
        *lock(&self.phase)
    }

    /// Marks a start or a stop as running. Fails if one is running already.
    fn begin(&self, phase: Phase) -> Result<PhaseGuard<'_>, HostError> {
        let mut current = lock(&self.phase);
        match *current {
            Phase::Idle => {
                *current = phase;
                Ok(PhaseGuard(&self.phase))
            }
            Phase::Starting => Err(HostError("Captain Engine is starting.".into())),
            Phase::Stopping => Err(HostError("Captain Engine is stopping.".into())),
        }
    }
}

/// Sets the phase back to idle when the action ends, also on an early return.
struct PhaseGuard<'a>(&'a Mutex<Phase>);

impl Drop for PhaseGuard<'_> {
    fn drop(&mut self) {
        *lock(self.0) = Phase::Idle;
    }
}

impl EngineHost for LimaHost {
    fn can_control(&self) -> bool {
        true
    }

    fn status(&self) -> HostFuture<HostStatus> {
        let inner = self.inner.clone();
        blocking(move || Ok(steps::status(&inner)))
    }

    fn start(&self) -> HostStream<String> {
        let inner = self.inner.clone();
        let (tx, rx) = mpsc::unbounded();
        std::thread::spawn(move || {
            let mut sink = |line: String| {
                tx.unbounded_send(Ok(line)).ok();
            };
            if let Err(error) = steps::start(&inner, &mut sink) {
                tracing::warn!(%error, "Captain Engine did not start");
                tx.unbounded_send(Err(error)).ok();
            }
        });
        rx.boxed()
    }

    fn stop(&self) -> HostFuture<()> {
        let inner = self.inner.clone();
        blocking(move || steps::stop(&inner))
    }

    fn endpoint(&self) -> Option<String> {
        Some(format!(
            "unix://{}",
            self.inner.paths.docker_socket().display()
        ))
    }

    fn resources(&self) -> HostResources {
        *lock(&self.inner.resources)
    }

    fn set_resources(&self, resources: HostResources) -> HostFuture<()> {
        *lock(&self.inner.resources) = resources;
        let inner = self.inner.clone();
        blocking(move || steps::apply_resources(&inner))
    }

    fn reset(&self) -> HostFuture<()> {
        let inner = self.inner.clone();
        blocking(move || steps::reset(&inner))
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

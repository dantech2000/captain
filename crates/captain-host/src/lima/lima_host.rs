//! [`LimaHost`]: Captain Engine as one Lima instance named `captain`. See ADR 0008.

mod daemon_steps;
mod engine_lock;
mod kube_steps;
mod steps;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use captain_core::daemon::{DaemonSettings, DaemonState};
use captain_core::kubernetes::{KubernetesHost, KubernetesSettings};
use captain_core::process_lock::ProcessLock;
use captain_core::snapshot::EngineSnapshots;
use captain_core::{EngineHost, HostError, HostFuture, HostResources, HostStatus, HostStream};
use futures::StreamExt;
use futures::channel::mpsc;

use super::instance::LimaInstance;
use super::kubernetes::LimaKubernetes;
use super::limactl::Limactl;
use super::locate::{current_exe, locate_limactl};
use super::paths::LimaPaths;
use super::snapshot::LimaSnapshots;
use super::version::check_version;
use crate::blocking::blocking;
use crate::cancel::Cancel;
use crate::probe::{free_space, output_within};

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
    /// The Docker daemon settings for the next start.
    daemon: Mutex<DaemonSettings>,
    /// What the running engine uses, once a start applied it or a status check read
    /// it. `None` while stopped or not known yet.
    running_daemon: Mutex<Option<DaemonState>>,
    /// The instance as the last `limactl list` saw it: whether it ran, and its
    /// resources. `None` when it did not exist or was not read yet.
    seen: Mutex<Option<Seen>>,
    /// The Kubernetes settings for the next start. See ADR 0010.
    kubernetes: Mutex<KubernetesSettings>,
    /// The `limactl` that passed the version check.
    checked: Mutex<Option<PathBuf>>,
    /// The command a start runs now, so a stop can kill it and keep the start from
    /// running its next step.
    cancel: Cancel,
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
                exe: current_exe(),
                rosetta: cfg!(target_arch = "aarch64"),
                phase: Mutex::new(Phase::Idle),
                resources: Mutex::new(resources),
                daemon: Mutex::new(DaemonSettings::default()),
                running_daemon: Mutex::new(None),
                seen: Mutex::new(None),
                kubernetes: Mutex::new(KubernetesSettings::default()),
                checked: Mutex::new(None),
                cancel: Cancel::default(),
            }),
        }
    }

    /// True if `limactl` exists on this computer. It does not check the version.
    pub fn is_installed() -> bool {
        let exe = current_exe();
        let path = std::env::var_os("PATH");
        locate_limactl(exe.as_deref(), path.as_deref(), Path::is_file).is_some()
    }

    pub fn paths(&self) -> &LimaPaths {
        &self.inner.paths
    }

    /// The checked `limactl` that this host runs, or why there is none.
    pub fn limactl_path(&self) -> Result<PathBuf, String> {
        self.inner
            .limactl()
            .map(|limactl| limactl.binary().to_path_buf())
    }

    /// `limactl shell <instance> [command...]` with the caller's terminal, for the
    /// `captain shell` command.
    pub fn shell(&self, command: &[String]) -> Result<std::process::Command, HostError> {
        let mut args = vec!["shell".to_string(), self.inner.paths.instance.clone()];
        args.extend(command.iter().cloned());
        self.inner.limactl().map_err(HostError)?.interactive(&args)
    }

    /// The instance as `limactl list` reports it, or `None` when it does not exist.
    pub(crate) fn instance(&self) -> Result<Option<LimaInstance>, HostError> {
        let limactl = self.inner.limactl().map_err(HostError)?;
        steps::find(&self.inner, &limactl)
    }

    /// Takes the engine lock for a snapshot step, or fails when another process
    /// holds it.
    pub(crate) fn lock_for_snapshot(&self) -> Result<ProcessLock, HostError> {
        engine_lock::acquire_with(&self.inner, engine_lock::SNAPSHOT)
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
        let mut version = std::process::Command::new(&binary);
        version.arg("--version");
        let output = output_within(version, steps::QUICK_TIMEOUT)
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

/// What the last `limactl list` said about the instance.
#[derive(Debug, Clone, Copy)]
struct Seen {
    running: bool,
    resources: HostResources,
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

    fn set_daemon(&self, daemon: DaemonSettings) {
        *lock(&self.inner.daemon) = daemon;
    }

    fn running_daemon(&self) -> Option<DaemonState> {
        lock(&self.inner.running_daemon).clone()
    }

    fn running_resources(&self) -> Option<HostResources> {
        lock(&self.inner.seen)
            .filter(|seen| seen.running)
            .map(|seen| seen.resources)
    }

    fn current_disk(&self) -> Option<u64> {
        lock(&self.inner.seen).map(|seen| seen.resources.disk_bytes)
    }

    fn snapshots(&self) -> Option<Arc<dyn EngineSnapshots>> {
        Some(Arc::new(LimaSnapshots::new(self.clone())))
    }

    fn set_kubernetes(&self, kubernetes: KubernetesSettings) {
        self.set_kubernetes_settings(kubernetes);
    }

    fn kubernetes(&self) -> Option<Arc<dyn KubernetesHost>> {
        Some(Arc::new(LimaKubernetes::new(self.clone())))
    }

    fn files_dir(&self) -> Option<PathBuf> {
        let dir = self.inner.paths.instance_dir();
        Some(match dir.exists() {
            true => dir,
            false => self.inner.paths.lima_home.clone(),
        })
    }

    fn runtime_version(&self) -> Option<String> {
        let file = self.inner.paths.instance_dir().join("lima-version");
        let version = std::fs::read_to_string(file).ok()?;
        let version = version.trim();
        (!version.is_empty()).then(|| format!("Lima {version}"))
    }

    fn free_disk(&self) -> Option<u64> {
        let home = &self.inner.paths.lima_home;
        let existing = home.ancestors().find(|dir| dir.exists())?;
        free_space(existing)
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests;

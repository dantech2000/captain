//! Kubernetes on [`LimaHost`]: the step after each start, and the guard that the
//! other Kubernetes actions run under. See ADR 0010.

use captain_core::HostError;
use captain_core::kubernetes::{KubernetesSettings, KubernetesStatus};

use super::{Inner, Phase, engine_lock, lock, steps};
use crate::LimaHost;
use crate::lima::kubernetes;
use crate::lima::limactl::Limactl;
use crate::lima::paths::LimaPaths;

/// The engine lock note while a Kubernetes action runs.
const KUBERNETES: &str = "kubernetes";

/// Starts k3s after the engine started, or makes sure it is off. A failure does not
/// fail the start: the engine runs, and the Kubernetes status shows the problem. A
/// stop kills it; the start checks for that next.
pub fn on_start(inner: &Inner, limactl: &Limactl, sink: &mut dyn FnMut(String)) {
    let settings = lock(&inner.kubernetes).clone();
    let result = if settings.enabled {
        kubernetes::install(limactl, &inner.paths, &settings, &inner.cancel, sink).map(drop)
    } else {
        kubernetes::disable(limactl, &inner.paths, &inner.cancel)
    };
    if let Err(error) = result {
        tracing::warn!(%error, "Kubernetes did not start");
        sink(format!("warning: Kubernetes did not start. {error}"));
    }
}

impl LimaHost {
    pub(crate) fn kubernetes_settings(&self) -> KubernetesSettings {
        lock(&self.inner.kubernetes).clone()
    }

    pub(crate) fn set_kubernetes_settings(&self, settings: KubernetesSettings) {
        *lock(&self.inner.kubernetes) = settings;
    }

    /// k3s's state, or [`KubernetesStatus::Off`] while the engine does not run.
    pub(crate) fn kubernetes_status(&self) -> KubernetesStatus {
        let inner = &self.inner;
        if inner.phase() != Phase::Idle {
            return KubernetesStatus::Off;
        }
        let Ok(limactl) = inner.limactl() else {
            return KubernetesStatus::Off;
        };
        match steps::find(inner, &limactl) {
            Ok(Some(instance)) if instance.status == "Running" => {
                kubernetes::status(&limactl, &inner.paths)
            }
            _ => KubernetesStatus::Off,
        }
    }

    /// Runs `work` while the engine runs and no start, stop, or other process acts
    /// on it.
    pub(crate) fn with_running_engine<T>(
        &self,
        work: impl FnOnce(&Limactl, &LimaPaths) -> Result<T, HostError>,
    ) -> Result<T, HostError> {
        let inner = &self.inner;
        let not_running = || HostError("Start Captain Engine first.".into());
        if inner.phase() != Phase::Idle {
            return Err(HostError("Captain Engine is starting or stopping.".into()));
        }
        let _lock = engine_lock::acquire_with(inner, KUBERNETES)?;
        let limactl = inner.limactl().map_err(HostError)?;
        match steps::find(inner, &limactl)? {
            Some(instance) if instance.status == "Running" => work(&limactl, &inner.paths),
            _ => Err(not_running()),
        }
    }
}

use std::sync::Arc;
use std::time::Duration;

use captain_core::kubernetes::{
    K3sVersion, KubernetesHost, KubernetesSettings, KubernetesStatus, VersionList,
};
use gpui_kit::*;

use super::KubeEvent;
use crate::engine_host::{HostModel, host_model};
use crate::settings;

/// How often to ask for the cluster's state while the engine runs.
const POLL: Duration = Duration::from_secs(5);

/// The cluster's state, the version list, and the apply or reset that runs now.
/// Actions live in `kube_actions.rs`.
pub struct KubernetesModel {
    pub(super) host: Arc<dyn KubernetesHost>,
    pub(super) engine: Entity<HostModel>,
    pub(super) status: KubernetesStatus,
    pub(super) versions: VersionList,
    /// The progress line of the apply or reset that runs now.
    pub(super) step: Option<SharedString>,
    pub(super) task: Option<Task<()>>,
    pub(super) _poll: Option<Task<()>>,
}

impl EventEmitter<KubeEvent> for KubernetesModel {}

struct KubernetesHandle(Entity<KubernetesModel>);

impl Global for KubernetesHandle {}

/// Installs the model when Captain Engine has a cluster. Call it after the host
/// model's `init`.
pub fn init(cx: &mut App) {
    let Some(engine) = host_model(cx) else {
        return;
    };
    let Some(host) = engine.read(cx).kubernetes() else {
        return;
    };
    let model = cx.new(|cx| {
        let mut model = KubernetesModel {
            host,
            engine,
            status: KubernetesStatus::Off,
            versions: VersionList::default(),
            step: None,
            task: None,
            _poll: None,
        };
        // The host falls back to the cached list when the network fails.
        model.load_versions(true, cx);
        model.poll(cx);
        model
    });
    cx.set_global(KubernetesHandle(model));
}

pub fn kubernetes_model(cx: &App) -> Option<Entity<KubernetesModel>> {
    cx.try_global::<KubernetesHandle>()
        .map(|handle| handle.0.clone())
}

impl KubernetesModel {
    /// The saved settings.
    pub fn settings(&self, cx: &App) -> KubernetesSettings {
        settings::current(cx).kubernetes
    }

    pub fn status(&self) -> &KubernetesStatus {
        &self.status
    }

    pub fn versions(&self) -> &VersionList {
        &self.versions
    }

    pub fn step(&self) -> Option<SharedString> {
        self.step.clone()
    }

    /// True while an apply or a reset runs.
    pub fn is_busy(&self) -> bool {
        self.task.is_some()
    }

    pub fn engine_running(&self, cx: &App) -> bool {
        self.engine.read(cx).status().is_running()
    }

    /// True if `version` is lower than the one the cluster runs, so it needs a reset.
    pub fn is_downgrade(&self, version: &K3sVersion) -> bool {
        self.status
            .version()
            .and_then(|running| running.parse::<K3sVersion>().ok())
            .is_some_and(|running| *version < running)
    }

    /// Asks for the state every few seconds while the engine runs.
    fn poll(&mut self, cx: &mut Context<Self>) {
        self._poll = Some(cx.spawn(async move |this, cx| {
            loop {
                let Ok(check) = this.update(cx, |model, cx| {
                    let idle = model.task.is_none();
                    if !model.engine_running(cx) {
                        model.set_status(KubernetesStatus::Off, cx);
                    }
                    (idle && model.engine_running(cx)).then(|| model.host.status())
                }) else {
                    return;
                };
                if let Some(check) = check {
                    let status = check.await;
                    this.update(cx, |model, cx| {
                        let status =
                            status.unwrap_or_else(|error| KubernetesStatus::Failed(error.0));
                        model.set_status(status, cx);
                    })
                    .ok();
                }
                cx.background_executor().timer(POLL).await;
            }
        }));
    }

    pub(super) fn set_status(&mut self, status: KubernetesStatus, cx: &mut Context<Self>) {
        if self.status != status {
            self.status = status;
            cx.notify();
        }
    }
}

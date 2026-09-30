use std::sync::Arc;
use std::time::Duration;

use captain_core::HostError;
use captain_core::kubernetes::{Forward, ForwardKey, KubeService, PortForwarding};
use gpui_kit::*;

use crate::kubernetes::KubeEvent;

/// How often the page reads the Services while it shows.
const POLL: Duration = Duration::from_secs(5);

struct Backend(Arc<dyn PortForwarding>);

impl Global for Backend {}

/// Installs the forwarder. The app calls it once Captain Engine has a cluster.
pub fn init(cx: &mut App, backend: Arc<dyn PortForwarding>) {
    cx.set_global(Backend(backend));
}

struct Shared(Entity<ForwardingModel>);

impl Global for Shared {}

/// The one model that the Port Forwarding page and the ⌘K palette share.
pub fn forwarding_model(cx: &mut App) -> Entity<ForwardingModel> {
    if let Some(shared) = cx.try_global::<Shared>() {
        return shared.0.clone();
    }
    let model = cx.new(ForwardingModel::new);
    cx.set_global(Shared(model.clone()));
    model
}

/// The Services and the forwards that run.
pub struct ForwardingModel {
    backend: Option<Arc<dyn PortForwarding>>,
    services: Vec<KubeService>,
    forwards: Vec<Forward>,
    /// Why the last read of the Services failed.
    error: Option<SharedString>,
    loaded: bool,
    showing: bool,
    _poll: Option<Task<()>>,
}

impl EventEmitter<KubeEvent> for ForwardingModel {}

impl ForwardingModel {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            backend: cx.try_global::<Backend>().map(|backend| backend.0.clone()),
            services: Vec::new(),
            forwards: Vec::new(),
            error: None,
            loaded: false,
            showing: false,
            _poll: None,
        }
    }

    pub fn services(&self) -> &[KubeService] {
        &self.services
    }

    pub fn error(&self) -> Option<SharedString> {
        self.error.clone()
    }

    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    /// The local port of the forward for `key`, if it runs.
    pub fn local_port(&self, key: &ForwardKey) -> Option<u16> {
        self.forwards
            .iter()
            .find(|forward| forward.key == *key)
            .map(|forward| forward.local_port)
    }

    pub fn forward_count(&self) -> usize {
        self.forwards.len()
    }

    /// Reads the Services every few seconds while the page shows.
    pub fn set_showing(&mut self, showing: bool, cx: &mut Context<Self>) {
        if showing == self.showing {
            return;
        }
        self.showing = showing;
        self._poll = showing.then(|| {
            cx.spawn(async move |this, cx| {
                loop {
                    let Ok(Some(read)) = this.update(cx, |model, _| {
                        model.backend.as_ref().map(|backend| backend.services())
                    }) else {
                        return;
                    };
                    let result = read.await;
                    this.update(cx, |model, cx| model.take(result, cx)).ok();
                    cx.background_executor().timer(POLL).await;
                }
            })
        });
    }

    /// Reads the Services once, for the ⌘K palette.
    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        let Some(read) = self.backend.as_ref().map(|backend| backend.services()) else {
            return;
        };
        cx.spawn(async move |this, cx| {
            let result = read.await;
            this.update(cx, |model, cx| model.take(result, cx)).ok();
        })
        .detach();
    }

    fn take(&mut self, result: Result<Vec<KubeService>, HostError>, cx: &mut Context<Self>) {
        self.loaded = true;
        match result {
            Ok(services) => {
                self.services = services;
                self.error = None;
            }
            Err(error) => self.error = Some(error.0.into()),
        }
        self.refresh_forwards(cx);
    }

    /// Starts a forward on `local_port`, or a free port when it is `None`.
    pub fn forward(&mut self, key: ForwardKey, local_port: Option<u16>, cx: &mut Context<Self>) {
        let Some(backend) = self.backend.clone() else {
            return;
        };
        let start = backend.forward(key, local_port);
        cx.spawn(async move |this, cx| {
            let result = start.await;
            this.update(cx, |model, cx| {
                if let Err(error) = result {
                    cx.emit(KubeEvent {
                        action: "Forward",
                        message: error.0,
                    });
                }
                model.refresh_forwards(cx);
            })
            .ok();
        })
        .detach();
    }

    pub fn stop(&mut self, key: &ForwardKey, cx: &mut Context<Self>) {
        if let Some(backend) = &self.backend {
            backend.stop(key);
        }
        self.refresh_forwards(cx);
    }

    fn refresh_forwards(&mut self, cx: &mut Context<Self>) {
        self.forwards = self
            .backend
            .as_ref()
            .map(|backend| backend.forwards())
            .unwrap_or_default();
        cx.notify();
    }
}

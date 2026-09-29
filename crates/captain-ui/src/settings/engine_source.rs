use std::rc::Rc;

use gpui_kit::*;

use super::store;
use crate::workspace::{Connector, Workspace, active_workspace};

/// An engine endpoint found on this machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedEndpoint {
    /// Where it was found, for example "DOCKER_HOST" or "Socket".
    pub source: SharedString,
    /// The `DOCKER_HOST`-style URL.
    pub host: SharedString,
}

/// How the UI finds and reaches engines. The app implements it, because only the app
/// knows the engine crate; `captain-ui` sees only the [`captain_core::Engine`] trait.
pub trait EngineSource {
    /// A connector for `endpoint`, or for discovery when it is `None`.
    fn connector(&self, endpoint: Option<&str>) -> Connector;
    /// Checks a `DOCKER_HOST`-style URL. The error is a message for the user.
    fn check_endpoint(&self, host: &str) -> Result<(), String>;
    /// The endpoints that exist on this machine, in discovery order.
    fn detected(&self) -> Vec<DetectedEndpoint>;
}

struct EngineSourceGlobal(Rc<dyn EngineSource>);

impl Global for EngineSourceGlobal {}

/// Installs the app's engine source. Without it, the Settings page cannot switch
/// engines and Retry does nothing.
pub fn init_engine_source(cx: &mut App, source: Rc<dyn EngineSource>) {
    cx.set_global(EngineSourceGlobal(source));
}

pub fn engine_source(cx: &App) -> Option<Rc<dyn EngineSource>> {
    cx.try_global::<EngineSourceGlobal>()
        .map(|global| global.0.clone())
}

/// Drops the current connection and connects again, to the endpoint in the settings
/// or by discovery.
pub fn reconnect(workspace: &Entity<Workspace>, cx: &mut App) {
    let Some(source) = engine_source(cx) else {
        tracing::warn!("no engine source, so Captain cannot reconnect");
        return;
    };
    let endpoint = store::current(cx).engine_endpoint;
    let connector = source.connector(endpoint.as_deref());
    workspace.update(cx, |workspace, cx| workspace.reconnect(connector, cx));
}

/// Saves `endpoint` as the engine to use (`None` means discovery) and reconnects.
pub fn use_engine(workspace: &Entity<Workspace>, endpoint: Option<String>, cx: &mut App) {
    store::update(cx, |settings| settings.engine_endpoint = endpoint);
    reconnect(workspace, cx);
}

/// Reconnects the workspace that connected last. The Retry button uses it.
pub fn retry(cx: &mut App) {
    if let Some(workspace) = active_workspace(cx) {
        reconnect(&workspace, cx);
    }
}

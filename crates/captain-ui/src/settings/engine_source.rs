use std::rc::Rc;

use captain_core::docker_context::ContextList;
use captain_core::settings::EngineChoice;
use gpui_kit::*;

use super::store;
use crate::engine_host;
use crate::workspace::{Connector, Workspace, active_workspace, expected_running};

/// An engine endpoint found on this machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedEndpoint {
    /// Where it was found, for example "DOCKER_HOST" or "Socket".
    pub source: SharedString,
    /// The `DOCKER_HOST`-style URL.
    pub host: SharedString,
}

/// A change to the Docker CLI contexts. It may block, so the UI runs it on a
/// background thread. The error is a message for the user.
pub type ContextJob = Box<dyn FnOnce() -> Result<(), String> + Send>;

/// How the UI finds and reaches engines. The app implements it, because only the app
/// knows the engine crate; `captain-ui` sees only the [`captain_core::Engine`] trait.
pub trait EngineSource {
    /// A connector for `endpoint`, or for discovery when it is `None`.
    fn connector(&self, endpoint: Option<&str>) -> Connector;
    /// Checks a `DOCKER_HOST`-style URL. The error is a message for the user.
    fn check_endpoint(&self, host: &str) -> Result<(), String>;
    /// The endpoints that exist on this machine, in discovery order.
    fn detected(&self) -> Vec<DetectedEndpoint>;
    /// The Docker CLI contexts and the default one.
    fn contexts(&self) -> ContextList;
    /// Creates the context `name`, or updates it, to point at `host`.
    fn save_context(&self, name: &str, description: &str, host: &str) -> ContextJob;
    /// Makes `name` the Docker CLI's default context.
    fn use_context(&self, name: &str) -> ContextJob;
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

/// Drops the current connection and connects again: to Captain Engine when the
/// settings choose it, else to the endpoint in the settings or by discovery.
pub fn reconnect(workspace: &Entity<Workspace>, cx: &mut App) {
    let captain = engine_host::captain_endpoint(cx);
    reconnect_to(workspace, captain, cx);
}

/// Reconnects to `captain` (the Captain Engine endpoint, when the settings choose it),
/// or else to the saved or discovered engine.
///
/// The engine host model calls this from inside its own update with the endpoint it
/// already knows, because reading the model again there would panic (GPUI does not
/// allow reading an entity while it is being updated).
pub fn reconnect_to(workspace: &Entity<Workspace>, captain: Option<String>, cx: &mut App) {
    let Some(source) = engine_source(cx) else {
        tracing::warn!("no engine source, so Captain cannot reconnect");
        return;
    };
    let endpoint = captain.or_else(|| store::current(cx).engine_endpoint);
    let connector = source.connector(endpoint.as_deref());
    workspace.update(cx, |workspace, cx| workspace.reconnect(connector, cx));
}

/// The automatic reconnect after a drop: connects again to the same engine, or
/// stops trying when that engine should not answer now (Captain Engine stopped).
/// See feature 0013.
pub(crate) fn resume(workspace: &Entity<Workspace>, cx: &mut App) {
    let source = engine_source(cx).filter(|_| expected_running(cx));
    let Some(source) = source else {
        workspace.update(cx, |workspace, cx| workspace.stop_reconnecting(cx));
        return;
    };
    let endpoint = engine_host::captain_endpoint(cx).or_else(|| store::current(cx).engine_endpoint);
    let connector = source.connector(endpoint.as_deref());
    workspace.update(cx, |workspace, cx| workspace.resume(connector, cx));
}

/// Saves `endpoint` as the engine to use (`None` means discovery), switches away
/// from Captain Engine, and reconnects.
pub fn use_engine(workspace: &Entity<Workspace>, endpoint: Option<String>, cx: &mut App) {
    store::update(cx, |settings| {
        settings.engine_endpoint = endpoint;
        settings.engine = Some(EngineChoice::External);
    });
    reconnect(workspace, cx);
}

/// Reconnects the workspace that connected last. The Retry button uses it.
pub fn retry(cx: &mut App) {
    if let Some(workspace) = active_workspace(cx) {
        reconnect(&workspace, cx);
    }
}

use std::sync::Arc;

use captain_core::migration::MigrationBackend;
use gpui_kit::*;

use super::OpenMigrationAssistant;

struct BackendGlobal(Arc<dyn MigrationBackend>);

impl Global for BackendGlobal {}

impl OpenMigrationAssistant {
    /// Installs the app's migration backend. The app implements it, because only the
    /// app knows the engine crate. Without it, the assistant says it is unavailable.
    pub fn set_backend(cx: &mut App, backend: Arc<dyn MigrationBackend>) {
        cx.set_global(BackendGlobal(backend));
    }
}

/// The installed migration backend, if any.
pub fn backend(cx: &App) -> Option<Arc<dyn MigrationBackend>> {
    cx.try_global::<BackendGlobal>()
        .map(|global| global.0.clone())
}

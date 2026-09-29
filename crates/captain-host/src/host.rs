//! Picks the [`EngineHost`] for this platform.

use std::sync::Arc;

use captain_core::{EngineHost, HostResources};

/// The Captain Engine host for this platform, with `resources` for the next start.
#[cfg(target_os = "macos")]
pub fn default_host(resources: HostResources) -> Arc<dyn EngineHost> {
    Arc::new(crate::LimaHost::new(resources))
}

#[cfg(target_os = "linux")]
pub fn default_host(resources: HostResources) -> Arc<dyn EngineHost> {
    Arc::new(crate::SystemHost::new(resources))
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub fn default_host(resources: HostResources) -> Arc<dyn EngineHost> {
    Arc::new(crate::UnavailableHost::new(
        "Captain Engine is not available on this platform yet.",
        resources,
    ))
}

/// True if Captain Engine can run here, so it is the default engine choice. On
/// macOS that needs `limactl`. On Linux the system engine is found by discovery
/// anyway, so the other-engine path stays the default.
pub fn captain_engine_available() -> bool {
    cfg!(target_os = "macos") && crate::LimaHost::is_installed()
}

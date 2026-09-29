//! How the UI reaches the operating system for app behavior: the login item and the
//! `/var/run/docker.sock` link. The app implements it. See feature 0015.

use std::path::Path;
use std::sync::Arc;

use captain_core::behavior::docker_socket::{SocketLink, SocketProbe};
use gpui_kit::*;

/// System work that only the app can do. Errors are messages for the user. The
/// socket calls can block on a password prompt, so the UI runs them in the background.
pub trait SystemIntegration: Send + Sync {
    /// True if Captain starts when the user logs in, read from the system.
    fn login_item(&self) -> Result<bool, String>;
    fn set_login_item(&self, enabled: bool) -> Result<(), String>;
    /// True if this platform has a menu bar or notification area icon.
    fn has_menu_bar_icon(&self) -> bool;
    /// What `/var/run/docker.sock` is now, or `None` where it does not apply.
    fn docker_socket(&self) -> Option<SocketProbe>;
    /// Links `/var/run/docker.sock` to `target`, with administrator rights, if it is
    /// still what `replacing` says.
    fn link_docker_socket(&self, target: &Path, replacing: &SocketLink) -> Result<(), String>;
    /// Removes `/var/run/docker.sock`, with administrator rights, if it still links
    /// to Captain Engine's socket at `captain`.
    fn unlink_docker_socket(&self, captain: &Path) -> Result<(), String>;
}

struct SystemGlobal(Arc<dyn SystemIntegration>);

impl Global for SystemGlobal {}

/// Installs the app's system integration. Without it, Settings has no Behavior card.
pub fn init(cx: &mut App, system: Arc<dyn SystemIntegration>) {
    cx.set_global(SystemGlobal(system));
}

pub fn system(cx: &App) -> Option<Arc<dyn SystemIntegration>> {
    cx.try_global::<SystemGlobal>()
        .map(|global| global.0.clone())
}

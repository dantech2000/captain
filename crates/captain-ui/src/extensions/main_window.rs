use gpui_kit::*;

use crate::workspace::Workspace;

/// The main window and its workspace, which an extension's
/// `ddClient.desktopUI.navigate` calls bring forward and act on.
#[derive(Clone)]
// Only the macOS extension window reads it.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub struct MainWindow {
    pub workspace: Entity<Workspace>,
    pub window: AnyWindowHandle,
}

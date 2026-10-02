//! The icon column at the far left of the window: the app icon, the button that
//! hides the projects list, and the pages.

mod app_mark;
mod page_keys;
mod rail_button;
mod rail_view;

pub use page_keys::{
    ShowContainers, ShowDiagnostics, ShowExtensions, ShowImages, ShowNetworks, ShowPortForwarding,
    ShowSettings, ShowSnapshots, ShowStorage, ShowVolumes, on_page_actions, page_bindings,
};
pub use rail_button::{sidebar_help, terminal_help};
pub use rail_view::render;

gpui_kit::actions!(captain, [ToggleSidebar]);

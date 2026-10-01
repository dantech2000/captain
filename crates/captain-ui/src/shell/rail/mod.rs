//! The icon column at the far left of the window: the app icon, the button that
//! hides the projects list, and the pages.

mod app_mark;
mod rail_button;
mod rail_view;

pub use rail_button::sidebar_help;
pub use rail_view::render;

gpui_kit::actions!(captain, [ToggleSidebar]);

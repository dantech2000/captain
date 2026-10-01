mod known_entry;
mod page_help;
mod projects;
mod search_button;

pub(crate) use page_help::page_help;

use gpui_kit::*;

use crate::new_project::known_projects;
use crate::theme::Palette;
use crate::widgets::drag_region;
use crate::workspace::Workspace;

/// Search and the projects, to the right of the icon rail. The projects list takes
/// the free height. The engine's state shows in the status bar only.
pub fn render(
    handle: &Entity<Workspace>,
    workspace: &Workspace,
    palette: &Palette,
    cx: &App,
) -> impl IntoElement {
    div()
        .w(px(super::SIDEBAR_WIDTH))
        .h_full()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .px(px(12.))
        .pb(px(12.))
        .bg(palette.side)
        .border_r_1()
        .border_color(palette.sep)
        .child(drag_region("sidebar-drag").h(px(48.)).flex_shrink_0())
        .child(search_button::render(palette))
        .child(projects::render(
            handle,
            workspace,
            &known_projects(cx),
            palette,
        ))
}

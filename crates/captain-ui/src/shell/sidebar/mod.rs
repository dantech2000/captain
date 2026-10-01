mod engine_header;
mod page_help;
mod projects;
mod search_button;

pub(crate) use page_help::page_help;

use gpui_kit::*;

use crate::engine_host::HostSummary;
use crate::theme::Palette;
use crate::widgets::drag_region;
use crate::workspace::Workspace;

/// The engine header, search, and the projects, to the right of the icon rail. The
/// projects list takes the free height.
pub fn render(
    handle: &Entity<Workspace>,
    workspace: &Workspace,
    host: Option<&HostSummary>,
    palette: &Palette,
) -> impl IntoElement {
    div()
        .w(px(256.))
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
        .child(engine_header::render(handle, workspace, host, palette))
        .child(search_button::render(palette))
        .child(projects::render(handle, workspace, palette))
}

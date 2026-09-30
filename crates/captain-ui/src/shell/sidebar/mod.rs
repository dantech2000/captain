mod engine_card;
mod engine_header;
mod page_help;
mod projects;
mod search_button;
mod tools;

use gpui_kit::*;

use crate::engine_host::HostSummary;
use crate::theme::Palette;
use crate::widgets::drag_region;
use crate::workspace::Workspace;

/// The engine header, search, the projects, the resource pages as icons, and the
/// engine card. The projects list takes the free height, so the parts below it do
/// not move when entries come and go.
pub fn render(
    handle: &Entity<Workspace>,
    workspace: &Workspace,
    host: Option<&HostSummary>,
    failures: usize,
    forwarding: bool,
    palette: &Palette,
) -> impl IntoElement {
    div()
        .w(px(272.))
        .h_full()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .px(px(12.))
        .pb(px(14.))
        .bg(palette.side)
        .border_r_1()
        .border_color(palette.sep)
        .child(drag_region("sidebar-drag").h(px(48.)).flex_shrink_0())
        .child(engine_header::render(workspace, host, palette))
        .child(search_button::render(palette))
        .child(projects::render(handle, workspace, palette))
        // Feature 0031 puts the Disk card here, above the resource icons.
        .child(tools::render(
            handle, workspace, failures, forwarding, palette,
        ))
        .child(engine_card::render(workspace, host, palette))
}

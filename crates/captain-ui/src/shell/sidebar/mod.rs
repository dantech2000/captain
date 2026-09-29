mod brand;
mod engine_card;
mod nav;
mod page_link;
mod projects;
mod search_button;

pub use projects::project_badge;

use gpui_kit::*;

use crate::engine_host::HostSummary;
use crate::theme::Palette;
use crate::widgets::drag_region;
use crate::workspace::{Page, Workspace};

pub fn render(
    handle: &Entity<Workspace>,
    workspace: &Workspace,
    host: Option<&HostSummary>,
    failures: usize,
    forwarding: bool,
    palette: &Palette,
) -> impl IntoElement {
    let badge = (failures > 0).then_some(failures);
    div()
        .w(px(244.))
        .h_full()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap(px(2.))
        .px(px(12.))
        .pb(px(14.))
        .bg(palette.side)
        .border_r_1()
        .border_color(palette.sep)
        .child(drag_region("sidebar-drag").h(px(48.)).flex_shrink_0())
        .child(brand::render(workspace.connection(), host, palette))
        .child(search_button::render(palette))
        .child(nav::render(handle, workspace, palette))
        .child(projects::render(workspace.store(), palette))
        .child(div().flex_1())
        .child(page_link::render(
            handle,
            workspace,
            Page::Extensions,
            None,
            palette,
        ))
        .child(page_link::render(
            handle,
            workspace,
            Page::Snapshots,
            None,
            palette,
        ))
        .children(
            forwarding
                .then(|| page_link::render(handle, workspace, Page::PortForwarding, None, palette)),
        )
        .child(page_link::render(
            handle,
            workspace,
            Page::Diagnostics,
            badge,
            palette,
        ))
        .child(page_link::render(handle, workspace, Page::Settings, None, palette).mb(px(8.)))
        .child(engine_card::render(workspace, host, palette))
}

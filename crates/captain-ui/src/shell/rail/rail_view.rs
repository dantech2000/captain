use gpui_kit::*;

use super::app_mark;
use super::rail_button::{page_button, toggle_button};
use crate::engine_host::HostSummary;
use crate::shell::engine_state::EngineState;
use crate::theme::Palette;
use crate::widgets::drag_region;
use crate::workspace::{Page, Workspace};

/// The pages in the upper part of the rail. Port Forwarding shows only while the
/// cluster runs.
const PAGES: [Page; 8] = [
    Page::Containers,
    Page::Images,
    Page::Volumes,
    Page::Networks,
    Page::Snapshots,
    Page::Storage,
    Page::Extensions,
    Page::PortForwarding,
];

/// The pages pinned to the bottom of the rail.
const PINNED: [Page; 2] = [Page::Diagnostics, Page::Settings];

/// A slim column at the far left: room for the window buttons, the app icon with
/// the engine's state dot, the button that hides the projects list, the pages, and
/// Diagnostics and Settings at the bottom. `failures` is a red badge on Diagnostics.
pub fn render(
    handle: &Entity<Workspace>,
    workspace: &Workspace,
    host: Option<&HostSummary>,
    failures: usize,
    forwarding: bool,
    palette: &Palette,
) -> impl IntoElement {
    let engine = EngineState::of(workspace, host, palette);
    let badge = (failures > 0).then_some(failures);
    let button = |page: Page| {
        let (count, badge) = match page {
            Page::Diagnostics => (badge, badge),
            _ => (workspace.page_count(page), None),
        };
        let selected = workspace.page() == page;
        page_button(handle, page, selected, count, badge, palette)
    };
    let column = || div().flex().flex_col().items_center().gap(px(4.));
    div()
        .w(px(super::super::RAIL_WIDTH))
        .h_full()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .items_center()
        .pb(px(12.))
        .bg(palette.rail)
        .border_r_1()
        .border_color(palette.sep)
        // The window buttons sit here, over the rail and the sidebar.
        .child(drag_region("rail-drag").w_full().h(px(44.)).flex_shrink_0())
        .child(app_mark::render(handle, &engine, palette))
        .child(toggle_button(handle, workspace.sidebar_hidden(), palette))
        .child(
            div()
                .flex_shrink_0()
                .w(px(24.))
                .h(px(1.))
                .my(px(8.))
                .bg(palette.sep),
        )
        .child(
            column()
                .id("rail-pages")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .children(
                    PAGES
                        .into_iter()
                        .filter(|page| *page != Page::PortForwarding || forwarding)
                        .map(button),
                ),
        )
        .child(
            column()
                .flex_shrink_0()
                .pt(px(8.))
                .children(PINNED.map(button)),
        )
}

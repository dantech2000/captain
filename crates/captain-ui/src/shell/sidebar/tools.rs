use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::page_help::page_help;
use crate::help::HelpExt;
use crate::icons::glyph;
use crate::theme::Palette;
use crate::workspace::{Page, Workspace};

/// A row of icon buttons for the pages of all resources, Diagnostics, and Settings.
/// Port Forwarding shows only while the cluster runs. `failures` is a red badge on
/// Diagnostics.
pub fn render(
    handle: &Entity<Workspace>,
    workspace: &Workspace,
    failures: usize,
    forwarding: bool,
    palette: &Palette,
) -> impl IntoElement {
    let pages = [
        Page::Images,
        Page::Volumes,
        Page::Networks,
        Page::Snapshots,
        Page::Storage,
        Page::Extensions,
        Page::PortForwarding,
        Page::Diagnostics,
        Page::Settings,
    ];
    let badge = (failures > 0).then_some(failures);
    div()
        .flex_shrink_0()
        .flex()
        .justify_between()
        .pt(px(8.))
        .pb(px(8.))
        .children(
            pages
                .into_iter()
                .filter(|page| *page != Page::PortForwarding || forwarding)
                .map(|page| {
                    let count = match page {
                        Page::Diagnostics => badge,
                        _ => workspace.page_count(page),
                    };
                    let badge = badge.filter(|_| page == Page::Diagnostics);
                    tool(
                        handle,
                        page,
                        workspace.page() == page,
                        count,
                        badge,
                        palette,
                    )
                }),
        )
}

fn tool(
    handle: &Entity<Workspace>,
    page: Page,
    selected: bool,
    count: Option<usize>,
    badge: Option<usize>,
    palette: &Palette,
) -> Stateful<Div> {
    let handle = handle.clone();
    let hover = palette.nav_selected;
    let help = page_help(page, count);
    div()
        .id(SharedString::from(format!("sidebar-tool-{}", page.label())))
        .relative()
        .w(px(30.))
        .h(px(32.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(8.))
        .cursor_pointer()
        .when(selected, |this| this.bg(hover))
        .hover(move |style| style.bg(hover))
        .on_click(move |_, _, cx| handle.update(cx, |workspace, cx| workspace.set_page(page, cx)))
        .child(glyph(
            page.icon(),
            px(18.),
            if selected {
                palette.accent
            } else {
                palette.text2
            },
        ))
        .children(badge.map(|count| {
            div()
                .absolute()
                .top(px(1.))
                .right(px(0.))
                .min_w(px(14.))
                .h(px(14.))
                .px(px(3.))
                .flex()
                .items_center()
                .justify_center()
                .rounded_full()
                .bg(palette.red)
                .text_color(palette.on_red)
                .text_size(px(9.))
                .font_weight(FontWeight::BOLD)
                .child(count.to_string())
        }))
        .help(help)
}

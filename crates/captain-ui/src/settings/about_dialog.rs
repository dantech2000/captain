//! About Captain, from the Captain menu: the app mark, the versions that the
//! Settings page's About line shows, the license, and links. See
//! docs/features/0042-app-menus.md.

use gpui_kit::component::WindowExt;
use gpui_kit::*;

use super::about_section::{REPOSITORY, open_licenses, versions};
use crate::engine_host::host_model;
use crate::help::HelpExt;
use crate::theme::Palette;
use crate::widgets::brand_mark;
use crate::workspace::Workspace;

const LICENSE: &str = env!("CARGO_PKG_LICENSE");

/// Opens the About dialog in `window`. Does nothing while a dialog is open.
pub fn open(workspace: Entity<Workspace>, window: &mut Window, cx: &mut App) {
    if window.has_active_dialog(cx) {
        return;
    }
    window.open_dialog(cx, move |dialog, _, cx| {
        dialog
            .w(px(360.))
            .margin_top(px(120.))
            .child(body(&workspace, cx))
    });
}

/// Built on each render, so the Docker version shows once the engine connects.
fn body(workspace: &Entity<Workspace>, cx: &App) -> Div {
    let palette = Palette::of(cx);
    let mut lines = versions(host_model(cx).as_ref(), workspace, cx).into_iter();
    let title = lines.next().unwrap_or_default();
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.))
        .pb(px(8.))
        .child(brand_mark(px(64.)))
        .child(
            div()
                .pt(px(6.))
                .text_size(px(15.))
                .font_weight(FontWeight::SEMIBOLD)
                .child(title),
        )
        .children(lines.map(|line| {
            div()
                .text_size(px(12.))
                .text_color(palette.text2)
                .child(line)
        }))
        .child(
            div()
                .text_size(px(12.))
                .text_color(palette.text3)
                .child(format!("License: {LICENSE}")),
        )
        .child(
            div()
                .pt(px(6.))
                .flex()
                .gap(px(16.))
                .text_size(px(12.))
                .text_color(palette.link)
                .child(
                    link("about-licenses", "Licenses")
                        .on_click(|_, _, cx| open_licenses(cx))
                        .help("Show the licenses of Captain and the tools it ships."),
                )
                .child(
                    link("about-repository", "Source code")
                        .on_click(|_, _, cx| cx.open_url(REPOSITORY))
                        .help("Open Captain's repository on GitHub."),
                ),
        )
}

fn link(id: &'static str, label: &'static str) -> Stateful<Div> {
    div().id(id).underline().cursor_pointer().child(label)
}

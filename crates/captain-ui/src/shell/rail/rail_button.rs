use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::help::{CMD, HelpExt};
use crate::icons::glyph;
use crate::shell::page_help;
use crate::theme::Palette;
use crate::workspace::{Page, Workspace};

/// A rail entry for `page`. `badge` is the red count on Diagnostics.
pub fn page_button(
    handle: &Entity<Workspace>,
    page: Page,
    selected: bool,
    count: Option<usize>,
    badge: Option<usize>,
    palette: &Palette,
) -> Stateful<Div> {
    let handle = handle.clone();
    let color = if selected {
        palette.accent_fg
    } else {
        palette.text2
    };
    button(format!("rail-page-{}", page.label()), palette)
        .when(selected, |this| this.bg(palette.nav_selected))
        .on_click(move |_, _, cx| handle.update(cx, |workspace, cx| workspace.set_page(page, cx)))
        .child(glyph(page.icon(), px(18.), color))
        .children(badge.map(|count| {
            div()
                .absolute()
                .top(px(2.))
                .right(px(1.))
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
        .help(page_help(page, count))
}

/// Hides the projects list, or shows it again when `hidden`. ⌘B does the same.
pub fn toggle_button(handle: &Entity<Workspace>, hidden: bool, palette: &Palette) -> Stateful<Div> {
    let handle = handle.clone();
    let icon = if hidden {
        IconName::PanelLeftOpen
    } else {
        IconName::PanelLeftClose
    };
    button("rail-toggle-sidebar", palette)
        .text_color(palette.text2)
        .on_click(move |_, _, cx| handle.update(cx, |workspace, cx| workspace.toggle_sidebar(cx)))
        .child(Icon::new(icon).size(px(18.)))
        .help_keys(sidebar_help(hidden), &[CMD, "B"])
}

/// The status bar sentence for the sidebar button, and for the palette's row.
pub fn sidebar_help(hidden: bool) -> &'static str {
    if hidden {
        "Show the projects list, the Disk card, and the engine card."
    } else {
        "Hide the projects list, so the page gets the width."
    }
}

/// The square that every rail entry shares.
fn button(id: impl Into<ElementId>, palette: &Palette) -> Stateful<Div> {
    let hover = palette.nav_selected;
    div()
        .id(id)
        .relative()
        .flex_shrink_0()
        .size(px(36.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(8.))
        .cursor_pointer()
        .hover(move |style| style.bg(hover))
}

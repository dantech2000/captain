use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::component::kbd::Kbd;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::help::{CMD, HelpExt};
use crate::icons::glyph;
use crate::shell::page_help;
use crate::terminal_panel::TOGGLE_KEYS;
use crate::theme::Palette;
use crate::workspace::{Page, Workspace};

use super::page_keys::page_key;

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
        .tooltip(move |window, cx| {
            Tooltip::new(page.label())
                .key_binding(page_kbd(page))
                .build(window, cx)
        })
        .help_keys(page_help(page, count), page_keys(page))
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
        .tooltip(move |window, cx| {
            let label = if hidden {
                "Show sidebar"
            } else {
                "Hide sidebar"
            };
            // Both cmd-b and ctrl-b are bound; show the one this platform uses.
            let kbd = Keystroke::parse(&format!("{}-b", modifier()))
                .ok()
                .map(Kbd::new);
            Tooltip::new(label).key_binding(kbd).build(window, cx)
        })
        .help_keys(sidebar_help(hidden), &[CMD, "B"])
}

/// Shows the terminal panel, or hides it. ⌃` does the same.
pub fn terminal_button(handle: &Entity<Workspace>, open: bool, palette: &Palette) -> Stateful<Div> {
    let handle = handle.clone();
    let color = if open {
        palette.accent_fg
    } else {
        palette.text2
    };
    button("rail-terminal", palette)
        .when(open, |this| this.bg(palette.nav_selected))
        .text_color(color)
        .on_click(move |_, _, cx| handle.update(cx, |workspace, cx| workspace.toggle_terminal(cx)))
        .child(Icon::new(IconName::SquareTerminal).size(px(18.)))
        .tooltip(|window, cx| {
            let kbd = Keystroke::parse("ctrl-`").ok().map(Kbd::new);
            Tooltip::new("Terminal").key_binding(kbd).build(window, cx)
        })
        .help_keys(terminal_help(open), TOGGLE_KEYS)
}

/// The status bar sentence for the terminal button, and for the palette's row.
pub fn terminal_help(open: bool) -> &'static str {
    if open {
        "Hide the terminal panel. Its tabs keep running."
    } else {
        "Show the terminal panel: shells on this computer whose docker uses Captain's engine."
    }
}

/// The status bar sentence for the sidebar button, and for the palette's row.
pub fn sidebar_help(hidden: bool) -> &'static str {
    if hidden {
        "Show the projects list."
    } else {
        "Hide the projects list, so the page gets the width."
    }
}

/// The modifier this platform's shortcuts use, for tooltips.
fn modifier() -> &'static str {
    if cfg!(target_os = "macos") {
        "cmd"
    } else {
        "ctrl"
    }
}

/// The shortcut of `page` as a tooltip key chip.
fn page_kbd(page: Page) -> Option<Kbd> {
    let key = page_key(page)?;
    Keystroke::parse(&format!("{}-{key}", modifier()))
        .ok()
        .map(Kbd::new)
}

/// The shortcut of `page` for the status bar's key chips.
fn page_keys(page: Page) -> &'static [&'static str] {
    match page_key(page) {
        Some("1") => &[CMD, "1"],
        Some("2") => &[CMD, "2"],
        Some("3") => &[CMD, "3"],
        Some("4") => &[CMD, "4"],
        Some("5") => &[CMD, "5"],
        Some("6") => &[CMD, "6"],
        Some("7") => &[CMD, "7"],
        Some("8") => &[CMD, "8"],
        Some("9") => &[CMD, "9"],
        Some(",") => &[CMD, ","],
        _ => &[],
    }
}

/// The square that every rail entry shares. Icon-only buttons also get a short
/// tooltip with their name; the status bar keeps the longer sentence.
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

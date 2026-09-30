//! The "Everything else" row: the settings file and the list of all options. The
//! settings that left the page live there. See feature 0037.

use gpui_kit::*;

use super::open_file::open_settings_file;
use super::options_sheet;
use super::page_section::{fill_note, row};
use crate::help::HelpExt;
use crate::theme::Palette;

pub fn render(palette: &Palette) -> Div {
    div()
        .px(px(18.))
        .py(px(14.))
        .rounded(px(14.))
        .border_1()
        .border_dashed()
        .border_color(palette.border_strong)
        .child(
            row("Everything else", palette)
                .child(fill_note(
                    "Registry mirrors, the Docker socket, daemon.json, Kubernetes port and Traefik, and more live in settings.json.",
                    palette,
                ))
                .child(open_button("settings-open-file", palette))
                .child(
                    div()
                        .id("settings-all-options")
                        .text_size(px(12.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(palette.link)
                        .cursor_pointer()
                        .on_click(|_, window, cx| options_sheet::open(window, cx))
                        .child("All options")
                        .help("List every setting in the settings file, with a search field."),
                ),
        )
}

/// Open settings file, which shows an error when the editor does not open.
pub fn open_button(id: &'static str, palette: &Palette) -> Stateful<Div> {
    div()
        .id(id)
        .h(px(32.))
        .px(px(12.))
        .flex_shrink_0()
        .flex()
        .items_center()
        .rounded(px(8.))
        .border_1()
        .border_color(palette.border_strong)
        .bg(palette.button)
        .text_size(px(12.5))
        .font_weight(FontWeight::SEMIBOLD)
        .cursor_pointer()
        .hover(|style| style.opacity(0.85))
        .on_click(|_, _, cx| open_settings_file(cx))
        .child("Open settings file")
        .help("Open settings.json in your editor. Captain applies it when you save.")
}

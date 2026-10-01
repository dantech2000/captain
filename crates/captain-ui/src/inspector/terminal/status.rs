//! What the Terminal tab shows when there is no live shell.

use gpui_kit::*;

use crate::icons::{CaptainIcon, cap_icon};
use crate::theme::Palette;

/// The body for a stopped container.
pub fn not_running(palette: &Palette) -> Div {
    div()
        .flex_1()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(8.))
        .p(px(24.))
        .child(cap_icon(CaptainIcon::Exec, px(28.), palette.text3))
        .child(
            div()
                .font_weight(FontWeight::SEMIBOLD)
                .child("Container is not running"),
        )
        .child(
            div()
                .text_size(px(12.))
                .text_color(palette.text2)
                .text_center()
                .child("Start the container to open a shell in it."),
        )
}

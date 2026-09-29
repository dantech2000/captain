use gpui_kit::*;

use crate::theme::Palette;

/// A result message in green text, for finished bulk actions such as a prune.
pub fn inline_notice(message: impl Into<SharedString>, palette: &Palette) -> Div {
    div()
        .text_size(px(12.))
        .text_color(palette.green)
        .child(message.into())
}

use gpui_kit::*;

use crate::theme::Palette;

/// An error message in red text, for failed loads and actions.
pub fn inline_error(message: impl Into<SharedString>, palette: &Palette) -> Div {
    div()
        .text_size(px(12.))
        .text_color(palette.red)
        .child(message.into())
}

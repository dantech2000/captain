use gpui_kit::*;

use crate::theme::Palette;

/// A label above a control, for the Images dialogs.
pub fn field(label: &'static str, control: AnyElement, palette: &Palette) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(px(6.))
        .child(
            div()
                .text_size(px(11.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(palette.text3)
                .child(label),
        )
        .child(control)
}

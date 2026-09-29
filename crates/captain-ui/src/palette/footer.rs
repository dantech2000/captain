use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::*;

use crate::theme::Palette;

/// The bottom row: the Captain mark and the keys the palette understands.
pub fn render(palette: &Palette) -> impl IntoElement {
    div()
        .flex_shrink_0()
        .h(px(40.))
        .px(px(18.))
        .flex()
        .items_center()
        .gap(px(16.))
        .border_t_1()
        .border_color(palette.sep)
        .bg(palette.group)
        .text_size(px(11.))
        .text_color(palette.text3)
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.))
                .child(
                    Icon::new(IconName::ShipWheel)
                        .size(px(14.))
                        .text_color(palette.accent),
                )
                .child("Captain"),
        )
        .child(div().flex_1())
        .child("↑↓ Move")
        .child("↵ Run")
        .child("esc Close")
}

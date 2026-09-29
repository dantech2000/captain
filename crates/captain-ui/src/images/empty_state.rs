use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::*;

use crate::theme::Palette;

/// Shown when no image passes the filter, or the engine has no images.
pub fn render(filtered: bool, palette: &Palette) -> impl IntoElement {
    let (title, hint) = if filtered {
        (
            "No images match this filter",
            "Choose All to see every image.",
        )
    } else {
        (
            "No images yet",
            "Pull one above, for example busybox or nginx:alpine.",
        )
    };
    div()
        .pt(px(80.))
        .flex()
        .flex_col()
        .items_center()
        .gap(px(8.))
        .text_color(palette.text2)
        .child(
            Icon::new(IconName::Layers)
                .size(px(32.))
                .text_color(palette.text3),
        )
        .child(
            div()
                .text_color(palette.text)
                .font_weight(FontWeight::SEMIBOLD)
                .child(title),
        )
        .child(div().text_size(px(12.)).child(hint))
}

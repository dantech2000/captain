use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::*;

use crate::theme::Palette;

pub fn render(filtered: bool, palette: &Palette) -> impl IntoElement {
    let (title, hint) = if filtered {
        (
            "No containers match this filter",
            "Choose All to see every container.",
        )
    } else {
        (
            "No containers yet",
            "Run `docker run hello-world` and it shows up here.",
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
            Icon::new(IconName::Container)
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

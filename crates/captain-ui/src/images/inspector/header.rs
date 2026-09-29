use captain_core::format::bytes_label;
use captain_core::model::Image;
use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::*;

use crate::images::ImagesState;
use crate::theme::Palette;
use crate::widgets::{icon_button, pill};

/// The image icon, repository, tag, short ID, size, and usage, with a close button.
pub fn render(image: &Image, handle: &Entity<ImagesState>, palette: &Palette) -> impl IntoElement {
    let (repository, tag) = image.repository_and_tag();
    let color = if image.dangling {
        palette.orange
    } else {
        palette.indigo
    };
    let usage = match image.containers {
        0 if image.dangling => pill("dangling", palette.orange, palette.tint(palette.orange)),
        0 => pill("unused", palette.gray, palette.tint(palette.gray)),
        1 => pill("in use by 1", palette.green, palette.tint(palette.green)),
        n => pill(
            format!("in use by {n}"),
            palette.green,
            palette.tint(palette.green),
        ),
    };
    let tag = match (tag, image.extra_tag_count()) {
        ("", _) => "no tag".to_string(),
        (tag, 0) => tag.to_string(),
        (tag, extra) => format!("{tag} +{extra} more"),
    };
    let close = {
        let handle = handle.clone();
        icon_button(
            "close-image-inspector",
            IconName::Close,
            palette,
            move |_, _, cx| {
                handle.update(cx, |state, cx| state.deselect(cx));
            },
        )
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .tooltip(|window, cx| Tooltip::new("Close").build(window, cx))
    };

    div()
        .flex()
        .items_start()
        .gap(px(12.))
        .child(
            div()
                .size(px(44.))
                .flex_shrink_0()
                .rounded(px(12.))
                .bg(palette.tint(color))
                .flex()
                .items_center()
                .justify_center()
                .child(Icon::new(IconName::Layers).size(px(22.)).text_color(color)),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(4.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(
                            div()
                                .min_w_0()
                                .text_size(px(18.))
                                .font_weight(FontWeight::BOLD)
                                .truncate()
                                .child(repository.to_string()),
                        )
                        .child(usage),
                )
                .child(
                    div()
                        .font_family(palette.mono())
                        .text_size(px(12.))
                        .text_color(palette.text2)
                        .truncate()
                        .child(tag),
                )
                .child(
                    div()
                        .font_family(palette.mono())
                        .text_size(px(11.))
                        .text_color(palette.text3)
                        .child(format!(
                            "{} · {}",
                            image.short_id(),
                            bytes_label(image.size)
                        )),
                ),
        )
        .child(close)
}

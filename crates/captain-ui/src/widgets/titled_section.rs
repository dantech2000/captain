use gpui_kit::*;

use crate::theme::Palette;

/// A small grey heading, an optional note on its right, and the section body.
pub fn titled_section(
    title: &'static str,
    note: Option<String>,
    body: impl IntoElement,
    palette: &Palette,
) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .child(
            div()
                .flex()
                .justify_between()
                .text_size(px(11.))
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(palette.text3)
                        .child(title),
                )
                .children(note.map(|note| div().text_color(palette.text2).child(note))),
        )
        .child(body)
}

/// A grey line for an empty section, for example "No labels".
pub fn section_note(text: impl Into<SharedString>, palette: &Palette) -> Div {
    div()
        .text_size(px(12.))
        .text_color(palette.text3)
        .child(text.into())
}

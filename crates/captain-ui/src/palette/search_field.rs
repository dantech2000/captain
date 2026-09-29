use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::*;

use super::key_hint;
use crate::theme::Palette;

/// The top row: a search icon, the query field, and the `esc` hint.
pub fn render(input: &Entity<InputState>, palette: &Palette) -> impl IntoElement {
    div()
        .flex_shrink_0()
        .h(px(56.))
        .px(px(18.))
        .flex()
        .items_center()
        .gap(px(12.))
        .border_b_1()
        .border_color(palette.sep)
        .child(
            Icon::new(IconName::Search)
                .size(px(18.))
                .text_color(palette.text2),
        )
        .child(
            div().flex_1().min_w_0().child(
                Input::new(input)
                    .appearance(false)
                    .aria_label("Command")
                    .px_0()
                    .text_size(px(17.)),
            ),
        )
        .child(key_hint("esc", palette))
}

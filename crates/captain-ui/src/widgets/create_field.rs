use gpui_kit::component::Sizable;
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::*;

use super::{ButtonTone, text_button};
use crate::theme::Palette;

/// A name field with a Create button, and a red hint under it when the name is not
/// valid. The Create button is off while `busy`.
pub fn create_field(
    id: &'static str,
    input: &Entity<InputState>,
    hint: Option<SharedString>,
    busy: bool,
    palette: &Palette,
    on_create: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Div {
    let label = if busy { "Creating..." } else { "Create" };
    div()
        .flex()
        .flex_col()
        .gap(px(4.))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.))
                .child(div().w(px(240.)).child(Input::new(input).small()))
                .child(text_button(
                    id,
                    label,
                    ButtonTone::Accent,
                    !busy,
                    palette,
                    on_create,
                )),
        )
        .children(hint.map(|hint| div().text_size(px(11.)).text_color(palette.red).child(hint)))
}

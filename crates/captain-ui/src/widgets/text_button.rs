use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::kit_button::{Look, kit_button};
use crate::theme::Palette;

/// The color of a [`text_button`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonTone {
    Accent,
    Danger,
}

/// A small button with a text label, tinted with its tone. A disabled button is gray
/// and ignores clicks. It takes keyboard focus, like every kit button.
pub fn text_button(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    tone: ButtonTone,
    enabled: bool,
    palette: &Palette,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let color = match tone {
        ButtonTone::Accent => palette.accent,
        ButtonTone::Danger => palette.red,
    };
    let look = Look {
        bg: palette.tint(color),
        fg: palette.readable(color),
        hover: color.alpha(if palette.dark { 0.26 } else { 0.18 }),
    };
    let label: SharedString = label.into();
    let (field, text3) = (palette.field, palette.text3);
    kit_button(id, label.clone(), look, enabled, on_click, |button| {
        button
            .flex_grow(1.)
            .h_full()
            .px(px(10.))
            .rounded(px(7.))
            .font_weight(FontWeight::MEDIUM)
            .when(enabled, |button| button.cursor_pointer())
            .when(!enabled, |button| button.bg(field).text_color(text3))
            .child(div().text_size(px(12.)).child(label))
    })
    .h(px(26.))
    .flex_shrink_0()
}

use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::theme::Palette;

/// The color of a [`text_button`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonTone {
    Accent,
    Danger,
}

/// A small button with a text label, tinted with its tone. A disabled button is gray
/// and ignores clicks.
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
    let hover = color.alpha(if palette.dark { 0.26 } else { 0.18 });
    div()
        .id(id)
        .h(px(26.))
        .px(px(10.))
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(7.))
        .text_size(px(12.))
        .font_weight(FontWeight::MEDIUM)
        .when(enabled, |button| {
            button
                .bg(palette.tint(color))
                .text_color(palette.readable(color))
                .cursor_pointer()
                .hover(move |style| style.bg(hover))
                .on_click(on_click)
        })
        .when(!enabled, |button| {
            button.bg(palette.field).text_color(palette.text3)
        })
        .child(label.into())
}

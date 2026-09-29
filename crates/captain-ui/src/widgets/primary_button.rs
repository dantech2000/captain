use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::theme::Palette;

/// A large button filled with the accent color, for the main action of a screen.
/// A disabled button is gray and ignores clicks.
pub fn primary_button(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    enabled: bool,
    palette: &Palette,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let hover = palette.accent.opacity(0.85);
    div()
        .id(id)
        .h(px(34.))
        .px(px(18.))
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(9.))
        .text_size(px(13.))
        .font_weight(FontWeight::SEMIBOLD)
        .when(enabled, |button| {
            button
                .bg(palette.accent)
                .text_color(white())
                .cursor_pointer()
                .hover(move |style| style.bg(hover))
                .on_click(on_click)
        })
        .when(!enabled, |button| {
            button.bg(palette.field).text_color(palette.text3)
        })
        .child(label.into())
}

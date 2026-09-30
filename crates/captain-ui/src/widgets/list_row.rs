use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::theme::Palette;

/// The frame of one selectable row in a list card, styled like a container row.
/// The caller adds the cells and the click handler.
pub fn list_row(id: impl Into<ElementId>, selected: bool, palette: &Palette) -> Stateful<Div> {
    let hover = palette.hover;
    div()
        .id(id)
        .h(px(50.))
        .px(px(12.))
        .flex()
        .items_center()
        .gap(px(12.))
        .rounded(px(9.))
        .cursor_pointer()
        .when(selected, |row| {
            row.bg(palette.accent.alpha(if palette.dark { 0.15 } else { 0.08 }))
                .border_1()
                .border_color(palette.accent.alpha(0.35))
        })
        .when(!selected, |row| row.hover(move |style| style.bg(hover)))
}

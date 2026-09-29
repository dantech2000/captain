use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

/// A round color choice. The selected one has a ring around it.
pub fn swatch(
    id: impl Into<ElementId>,
    color: Hsla,
    selected: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let ring = if selected {
        color.alpha(0.6)
    } else {
        transparent_black()
    };
    div()
        .id(id)
        .size(px(24.))
        .flex_shrink_0()
        .rounded_full()
        .border_2()
        .border_color(ring)
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .when(!selected, |this| {
            this.hover(move |style| style.border_color(color.alpha(0.35)))
        })
        .on_click(on_click)
        .child(div().size(px(16.)).rounded_full().bg(color))
}

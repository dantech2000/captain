use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::*;

use crate::theme::Palette;

/// A clickable container in a detail panel: a state dot, the name, and a monospace
/// line below it, such as a mount path or an address.
pub fn container_link(
    id: impl Into<ElementId>,
    name: impl Into<SharedString>,
    detail: impl Into<SharedString>,
    dot: Hsla,
    palette: &Palette,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let hover = palette.hover;
    div()
        .id(id)
        .flex()
        .items_center()
        .gap(px(10.))
        .px(px(10.))
        .py(px(7.))
        .rounded(px(9.))
        .border_1()
        .border_color(palette.sep)
        .cursor_pointer()
        .hover(move |style| style.bg(hover))
        .on_click(on_click)
        .child(div().size(px(8.)).flex_shrink_0().rounded_full().bg(dot))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .truncate()
                        .child(name.into()),
                )
                .child(
                    div()
                        .font_family(palette.mono())
                        .text_size(px(11.))
                        .text_color(palette.text2)
                        .truncate()
                        .child(detail.into()),
                ),
        )
        .child(
            Icon::new(IconName::ChevronRight)
                .size(px(14.))
                .text_color(palette.text3),
        )
}

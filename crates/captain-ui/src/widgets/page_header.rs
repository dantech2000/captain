use gpui_kit::*;

use super::drag_region;
use crate::theme::Palette;

/// The title row at the top of a page: a large title, a summary line, and an
/// optional element on the right (filters, buttons). The empty parts drag the window.
/// Elements in `trailing` that handle clicks should stop mouse-down propagation.
pub fn page_header(
    id: &'static str,
    title: &'static str,
    summary: impl Into<SharedString>,
    trailing: Option<AnyElement>,
    palette: &Palette,
) -> Stateful<Div> {
    drag_region(id)
        .flex_shrink_0()
        .flex()
        .items_center()
        .gap(px(12.))
        .px(px(24.))
        .pt(px(18.))
        .pb(px(14.))
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(
                    div()
                        .text_size(px(22.))
                        .font_weight(FontWeight::BOLD)
                        .child(title),
                )
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(palette.text2)
                        .child(summary.into()),
                ),
        )
        .children(trailing)
}

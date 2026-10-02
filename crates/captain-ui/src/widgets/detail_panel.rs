use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;

use super::{drag_region, pill};
use crate::icons::{CaptainIcon, cap_icon};
use crate::theme::Palette;

/// The top of a detail panel: a tinted icon, the title with an optional badge, and a
/// line below it.
pub struct DetailHeader {
    pub icon: CaptainIcon,
    pub color: Hsla,
    pub title: SharedString,
    pub badge: Option<(SharedString, Hsla)>,
    pub subtitle: SharedString,
}

/// A 400 px panel on the right, like the container inspector: a header that drags the
/// window, and a scrolling body.
pub fn detail_panel(
    id: &'static str,
    header: DetailHeader,
    body: impl IntoElement,
    palette: &Palette,
) -> Div {
    div()
        .w(px(400.))
        .h_full()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .bg(palette.panel)
        .border_l_1()
        .border_color(palette.sep)
        .child(
            drag_region(id)
                .px(px(20.))
                .pt(px(20.))
                .pb(px(16.))
                .border_b_1()
                .border_color(palette.sep)
                .child(header_row(header, palette)),
        )
        .child(
            div()
                .id(SharedString::from(format!("{id}-body")))
                .flex_1()
                .min_h_0()
                .overflow_y_scrollbar()
                .px(px(20.))
                .pt(px(16.))
                .pb(px(20.))
                .child(body),
        )
}

fn header_row(header: DetailHeader, palette: &Palette) -> Div {
    let title = div()
        .flex()
        .items_center()
        .gap(px(8.))
        .child(
            div()
                .min_w_0()
                .text_size(px(18.))
                .font_weight(FontWeight::BOLD)
                .truncate()
                .child(header.title),
        )
        .children(
            header
                .badge
                .map(|(label, color)| pill(label, palette.readable(color), palette.tint(color))),
        );
    div()
        .flex()
        .items_start()
        .gap(px(12.))
        .child(
            div()
                .size(px(44.))
                .flex_shrink_0()
                .rounded(px(12.))
                .bg(palette.tint(header.color))
                .flex()
                .items_center()
                .justify_center()
                .child(cap_icon(header.icon, px(22.), header.color)),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(4.))
                .child(title)
                .child(
                    div()
                        .font_family(palette.mono())
                        .text_size(px(12.))
                        .text_color(palette.text2)
                        .truncate()
                        .child(header.subtitle),
                ),
        )
}

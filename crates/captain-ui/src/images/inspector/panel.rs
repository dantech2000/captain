use captain_core::model::Image;
use gpui_kit::*;

use super::{actions, header, layers, sections};
use crate::images::ImagesState;
use crate::theme::Palette;
use crate::widgets::{drag_region, section_note};

/// The inspector panel, 400 px wide, for `image`. The sections appear once the
/// details have loaded.
pub fn render(
    image: &Image,
    handle: &Entity<ImagesState>,
    state: &ImagesState,
    palette: &Palette,
) -> impl IntoElement {
    let body = div().flex().flex_col().gap(px(18.));
    let body = match (state.detail(), state.layers(), state.detail_error()) {
        (_, _, Some(error)) => body.child(section_note(error.to_string(), palette)),
        (Some(detail), Some(layers), None) => body
            .child(sections::details(image, detail, palette))
            .child(sections::config(detail, palette))
            .child(sections::ports(detail, palette))
            .child(sections::environment(detail, palette))
            .child(sections::labels(detail, palette))
            .child(layers::render(layers, palette)),
        _ => body.child(section_note("Loading details...", palette)),
    };

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
            drag_region("image-inspector-header")
                .flex()
                .flex_col()
                .gap(px(16.))
                .px(px(20.))
                .pt(px(20.))
                .pb(px(16.))
                .border_b_1()
                .border_color(palette.sep)
                .child(header::render(image, handle, palette))
                .child(actions::render(image, handle, state, palette)),
        )
        .child(
            div()
                .id("image-inspector-body")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .px(px(20.))
                .pt(px(16.))
                .pb(px(20.))
                .child(body),
        )
}

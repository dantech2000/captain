//! Placeholders while data loads: gray bars that pulse, shaped like what will come.
//! See https://gpui-kit.com/component/skeleton.

use gpui_kit::component::skeleton::Skeleton;
use gpui_kit::*;

/// Widths of the bars in [`skeleton_lines`], as parts of the width, so the block
/// reads as text and not as a grid.
const LINE_WIDTHS: [f32; 4] = [0.9, 0.65, 0.8, 0.5];

/// `count` list rows of `height` while a list loads: an icon square, a name bar, and
/// a shorter detail bar, inside the list's own padding.
pub fn skeleton_rows(count: usize, height: Pixels) -> Div {
    div()
        .size_full()
        .flex()
        .flex_col()
        .gap(px(4.))
        .px(px(12.))
        .pt(px(10.))
        .children((0..count).map(move |ix| {
            div()
                .h(height)
                .flex_shrink_0()
                .flex()
                .items_center()
                .gap(px(12.))
                .px(px(14.))
                .child(Skeleton::new().size(px(22.)).rounded(px(6.)))
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .gap(px(6.))
                        .child(bar(LINE_WIDTHS[ix % LINE_WIDTHS.len()] * 0.5, px(11.)))
                        .child(
                            Skeleton::new()
                                .secondary()
                                .h(px(9.))
                                .w(relative(0.25))
                                .rounded(px(4.)),
                        ),
                )
        }))
}

/// `count` lines of text while a detail section loads.
pub fn skeleton_lines(count: usize) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(px(9.))
        .children((0..count).map(|ix| bar(LINE_WIDTHS[ix % LINE_WIDTHS.len()], px(10.))))
}

fn bar(width: f32, height: Pixels) -> Skeleton {
    Skeleton::new().h(height).w(relative(width)).rounded(px(4.))
}

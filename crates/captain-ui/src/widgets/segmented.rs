use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::theme::Palette;

/// Runs when a segment is clicked.
pub type SegmentHandler = Box<dyn Fn(&mut Window, &mut App)>;

/// One choice in a segmented control.
pub struct Segment {
    pub label: SharedString,
    pub selected: bool,
    pub on_click: SegmentHandler,
}

/// A macOS-style segmented control.
pub fn segmented(id: &'static str, segments: Vec<Segment>, palette: &Palette) -> Div {
    let shadow = if palette.dark { 0.4 } else { 0.12 };
    div()
        .flex()
        .p(px(2.))
        .rounded(px(9.))
        .bg(palette.field)
        .children(segments.into_iter().enumerate().map(|(ix, segment)| {
            let on_click = segment.on_click;
            div()
                .id((id, ix))
                .h(px(26.))
                .px(px(12.))
                .flex()
                .items_center()
                .rounded(px(7.))
                .cursor_pointer()
                .text_size(px(12.))
                .font_weight(FontWeight::MEDIUM)
                .text_color(if segment.selected {
                    palette.text
                } else {
                    palette.text2
                })
                .when(segment.selected, |this| {
                    this.bg(palette.segment).shadow(vec![BoxShadow {
                        color: hsla(0., 0., 0., shadow),
                        offset: point(px(0.), px(1.)),
                        blur_radius: px(2.),
                        spread_radius: px(0.),
                        inset: false,
                    }])
                })
                .on_click(move |_, window, cx| on_click(window, cx))
                .child(segment.label)
        }))
}

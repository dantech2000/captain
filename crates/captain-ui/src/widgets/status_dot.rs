use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::theme::Palette;

/// A state dot inside a soft ring. `glow` adds a halo, for running containers.
pub fn status_dot(color: Hsla, glow: bool, palette: &Palette) -> Div {
    div()
        .flex_shrink_0()
        .size(px(22.))
        .rounded_full()
        .bg(palette.tint(color))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .size(px(8.))
                .rounded_full()
                .bg(color)
                .when(glow, |dot| {
                    dot.shadow(vec![BoxShadow {
                        color: color.alpha(0.8),
                        offset: point(px(0.), px(0.)),
                        blur_radius: px(8.),
                        spread_radius: px(0.),
                        inset: false,
                    }])
                }),
        )
}

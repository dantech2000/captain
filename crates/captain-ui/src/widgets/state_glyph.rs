use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::icons::{CaptainIcon, cap_icon};

/// The container glyph in the state color, with a state dot at its lower right.
/// `glow` adds a halo to the dot, for running containers.
pub fn state_glyph(color: Hsla, glow: bool) -> Div {
    div()
        .relative()
        .flex_shrink_0()
        .size(px(22.))
        .child(cap_icon(CaptainIcon::Container, px(22.), color))
        .child(
            div()
                .absolute()
                .right(px(-1.))
                .bottom(px(-1.))
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

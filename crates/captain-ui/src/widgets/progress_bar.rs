use gpui_kit::*;

use crate::theme::Palette;

/// A thin horizontal bar filled to `fraction` (0.0 to 1.0) with `color`.
pub fn progress_bar(fraction: f32, color: Hsla, palette: &Palette) -> Div {
    let fraction = if fraction.is_finite() {
        fraction.clamp(0.0, 1.0)
    } else {
        0.0
    };
    div()
        .h(px(6.))
        .w_full()
        .rounded(px(3.))
        .bg(palette.track)
        .overflow_hidden()
        .child(
            div()
                .h_full()
                .w(relative(fraction))
                .rounded(px(3.))
                .bg(color),
        )
}

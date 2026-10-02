use gpui_kit::*;

use crate::icons::CaptainIcon;
use crate::theme::Palette;
use crate::widgets::empty_note;

/// Shown when no image passes the filter, or the engine has no images.
pub fn render(filtered: bool, palette: &Palette) -> impl IntoElement {
    let (title, hint) = if filtered {
        (
            "No images match this filter",
            "Choose All to see every image.",
        )
    } else {
        (
            "No images yet",
            "Pull one above, for example busybox or nginx:alpine.",
        )
    };
    empty_note(CaptainIcon::Image, title, hint, palette)
}

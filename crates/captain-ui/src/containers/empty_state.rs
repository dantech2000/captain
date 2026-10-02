use gpui_kit::*;

use crate::icons::CaptainIcon;
use crate::theme::Palette;
use crate::widgets::empty_note;

pub fn render(filtered: bool, palette: &Palette) -> impl IntoElement {
    let (title, hint) = if filtered {
        (
            "No containers match this filter",
            "Choose All to see every container.",
        )
    } else {
        (
            "No containers yet",
            "Run `docker run hello-world` and it shows up here.",
        )
    };
    empty_note(CaptainIcon::Container, title, hint, palette)
}

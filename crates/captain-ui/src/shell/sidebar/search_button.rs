use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::*;

use crate::palette::{ToggleCommandPalette, key_hint};
use crate::theme::Palette;

/// The shortcut that opens the command palette, as the key hint shows it.
const SHORTCUT: &str = if cfg!(target_os = "macos") {
    "⌘K"
} else {
    "Ctrl K"
};

/// A field-like button that opens the command palette.
pub fn render(palette: &Palette) -> impl IntoElement {
    let hover = palette.nav_selected;
    div()
        .id("search-button")
        .flex_shrink_0()
        .h(px(32.))
        .px(px(10.))
        .mb(px(10.))
        .flex()
        .items_center()
        .gap(px(8.))
        .rounded(px(8.))
        .border_1()
        .border_color(palette.sep)
        .bg(palette.field)
        .text_size(px(12.))
        .text_color(palette.text3)
        .cursor_pointer()
        .hover(move |style| style.bg(hover))
        .on_click(|_, window, cx| window.dispatch_action(Box::new(ToggleCommandPalette), cx))
        .child(Icon::new(IconName::Search).size(px(14.)))
        .child(div().flex_1().child("Search or run a command"))
        .child(key_hint(SHORTCUT, palette))
}

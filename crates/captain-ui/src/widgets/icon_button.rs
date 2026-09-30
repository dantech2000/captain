use gpui_kit::component::Icon;
use gpui_kit::*;

use crate::help::{HelpExt, Hint};
use crate::theme::Palette;

/// A square button that shows only an icon. `help` is its status bar sentence, the
/// only text that says what it does.
pub fn icon_button(
    id: impl Into<ElementId>,
    icon: impl Into<Icon>,
    help: impl Into<Hint>,
    palette: &Palette,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let hover = palette.nav_selected;
    div()
        .id(id)
        .size(px(26.))
        .flex_shrink_0()
        .rounded(px(7.))
        .border_1()
        .border_color(palette.sep)
        .bg(palette.button)
        .text_color(palette.text2)
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .hover(move |style| style.bg(hover))
        .on_click(on_click)
        .child(Icon::new(icon).size(px(13.)))
        .help(help)
}

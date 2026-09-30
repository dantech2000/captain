use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::help::{HelpExt, Hint};
use crate::theme::Palette;

/// A tall inspector button: an icon over a label. Buttons in a row share the width.
/// A disabled button is dimmed and ignores clicks. `help` is its status bar sentence.
pub fn action_button(
    label: &'static str,
    icon: IconName,
    help: impl Into<Hint>,
    color: Hsla,
    enabled: bool,
    palette: &Palette,
    on_click: impl Fn(&mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let hover = palette.nav_selected;
    div()
        .id(label)
        .flex_1()
        .min_w_0()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(5.))
        .pt(px(9.))
        .pb(px(7.))
        .rounded(px(10.))
        .border_1()
        .border_color(palette.sep)
        .bg(palette.button)
        .text_size(px(11.))
        .text_color(palette.readable(color))
        .when(!enabled, |this| this.opacity(0.4))
        .when(enabled, |this| {
            this.cursor_pointer()
                .hover(move |style| style.bg(hover))
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .on_click(move |_, window, cx| on_click(window, cx))
        })
        .child(Icon::new(icon).size(px(16.)))
        .child(label)
        .help(help)
}

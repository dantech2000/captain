use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::kit_button::{Look, kit_button};
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
    let fg = palette.readable(color);
    let look = Look {
        bg: palette.button,
        fg,
        hover: palette.nav_selected,
    };
    // The border is on the outer div: the kit paints a custom button's border in
    // its hover color.
    let (bg, border) = (palette.button, palette.sep);
    let on_click = move |_: &ClickEvent, window: &mut Window, cx: &mut App| on_click(window, cx);
    kit_button(label, label, look, enabled, on_click, |button| {
        button
            .w_full()
            .h_auto()
            .pt(px(9.))
            .pb(px(7.))
            .px_0()
            .rounded(px(9.))
            .line_height(phi())
            .when(enabled, |button| button.cursor_pointer())
            .when(!enabled, |button| button.bg(bg).text_color(fg))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(5.))
                    .text_size(px(11.))
                    .child(Icon::new(icon).size(px(16.)))
                    .child(label),
            )
    })
    .flex_1()
    .min_w_0()
    .rounded(px(10.))
    .border_1()
    .border_color(border)
    .when(!enabled, |this| this.opacity(0.4))
    .when(enabled, |this| {
        this.on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
    })
    .help(help)
}

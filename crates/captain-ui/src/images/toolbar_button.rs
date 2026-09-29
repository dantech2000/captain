use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::theme::Palette;

/// A small button with an icon and a label. A disabled button is dimmed and ignores clicks.
pub fn toolbar_button(
    id: &'static str,
    label: impl Into<SharedString>,
    icon: IconName,
    color: Hsla,
    enabled: bool,
    palette: &Palette,
    on_click: impl Fn(&mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let hover = palette.nav_selected;
    div()
        .id(id)
        .h(px(28.))
        .px(px(10.))
        .flex_shrink_0()
        .flex()
        .items_center()
        .gap(px(6.))
        .rounded(px(7.))
        .border_1()
        .border_color(palette.sep)
        .bg(palette.button)
        .text_size(px(12.))
        .font_weight(FontWeight::MEDIUM)
        .text_color(color)
        .when(!enabled, |this| this.opacity(0.4))
        .when(enabled, |this| {
            this.cursor_pointer()
                .hover(move |style| style.bg(hover))
                .on_click(move |_, window, cx| on_click(window, cx))
        })
        .child(Icon::new(icon).size(px(13.)))
        .child(label.into())
}

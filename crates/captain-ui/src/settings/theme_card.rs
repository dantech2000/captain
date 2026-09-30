use captain_core::settings::ThemeFamily;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::store;
use crate::theme::Palette;

/// A theme choice: a preview in the theme's own colors for the current mode, with
/// its name and a strip of its main colors. The selected one has a ring.
pub fn theme_card(family: ThemeFamily, selected: bool, palette: &Palette) -> Stateful<Div> {
    let theme = Palette::new(family, palette.dark);
    let ring = if selected {
        palette.accent
    } else {
        palette.sep
    };
    let hover = palette.border_strong;
    let strip = [
        theme.side,
        theme.accent,
        theme.green,
        theme.orange,
        theme.red,
    ];
    div()
        .id(SharedString::from(format!("theme-{}", family.label())))
        .w(px(104.))
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap(px(8.))
        .p(px(8.))
        .rounded(px(10.))
        .border_2()
        .border_color(ring)
        .bg(theme.bg)
        .text_color(theme.text)
        .text_size(px(12.))
        .font_weight(FontWeight::SEMIBOLD)
        .cursor_pointer()
        .when(!selected, |card| {
            card.hover(move |style| style.border_color(hover))
        })
        .on_click(move |_, _, cx| store::update(cx, |settings| settings.theme = family))
        .child(family.label())
        .child(
            div()
                .flex()
                .h(px(12.))
                .rounded(px(4.))
                .overflow_hidden()
                .children(strip.map(|color| div().flex_1().h_full().bg(color))),
        )
}

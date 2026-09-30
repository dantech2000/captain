use captain_core::settings::ThemeFamily;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::store;
use crate::help::HelpExt;
use crate::theme::Palette;

/// A theme choice: a strip of the theme's main colors for the current mode, and
/// its name. The selected one has an accent ring and a tint.
pub fn theme_card(family: ThemeFamily, selected: bool, palette: &Palette) -> Stateful<Div> {
    let theme = Palette::new(family, palette.dark);
    let hover = palette.accent;
    let strip = [theme.bg, theme.accent, theme.orange];
    div()
        .id(SharedString::from(format!("theme-{}", family.label())))
        .h(px(34.))
        .pl(px(8.))
        .pr(px(12.))
        .flex_shrink_0()
        .flex()
        .items_center()
        .gap(px(8.))
        .rounded(px(9.))
        .border_1()
        .text_size(px(12.5))
        .text_color(palette.text)
        .cursor_pointer()
        .when(selected, |card| {
            card.border_color(palette.accent)
                .bg(palette.tint(palette.accent))
                .font_weight(FontWeight::BOLD)
        })
        .when(!selected, |card| {
            card.border_color(palette.border_strong)
                .bg(palette.field)
                .font_weight(FontWeight::MEDIUM)
                .hover(move |style| style.border_color(hover))
        })
        .on_click(move |_, _, cx| store::update(cx, |settings| settings.theme = family))
        .child(
            div()
                .flex()
                .w(px(36.))
                .h(px(18.))
                .rounded(px(5.))
                .overflow_hidden()
                .children(strip.map(|color| div().flex_1().h_full().bg(color))),
        )
        .child(family.label())
        .help(format!(
            "Use the {} theme, in light and in dark.",
            family.label()
        ))
}

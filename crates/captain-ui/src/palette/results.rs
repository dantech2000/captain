use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::CommandPalette;
use super::key_hint;
use super::ranking::Ranked;
use crate::icons::glyph;
use crate::theme::Palette;

/// The result list: a header for each section, then its rows. The list scrolls
/// once it is taller than the window allows.
pub fn render(
    results: &[Ranked],
    selected: usize,
    query: &str,
    scroll: &ScrollHandle,
    palette: &Palette,
    cx: &mut Context<CommandPalette>,
) -> impl IntoElement {
    let mut list = div()
        .id("palette-results")
        .track_scroll(scroll)
        .overflow_y_scroll()
        .max_h(px(420.))
        .p(px(8.))
        .flex()
        .flex_col();

    if results.is_empty() {
        let text = if query.trim().is_empty() {
            "No commands yet".to_string()
        } else {
            format!("No commands match \u{201c}{}\u{201d}", query.trim())
        };
        return list.child(
            div()
                .py(px(22.))
                .flex()
                .justify_center()
                .text_color(palette.text3)
                .child(text),
        );
    }

    for (ix, ranked) in results.iter().enumerate() {
        let section = ranked.command.section;
        if ix == 0 || results[ix - 1].command.section != section {
            list = list.child(
                div()
                    .px(px(10.))
                    .pt(px(10.))
                    .pb(px(6.))
                    .text_size(px(11.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(palette.text3)
                    .child(section.label()),
            );
        }
        list = list.child(row(ix, ranked, ix == selected, palette, cx));
    }
    list
}

fn row(
    ix: usize,
    ranked: &Ranked,
    selected: bool,
    palette: &Palette,
    cx: &mut Context<CommandPalette>,
) -> impl IntoElement {
    let command = &ranked.command;
    let bold = HighlightStyle {
        font_weight: Some(FontWeight::BOLD),
        ..Default::default()
    };
    let title = StyledText::new(command.title.clone())
        .with_highlights(ranked.ranges.iter().map(|range| (range.clone(), bold)));

    div()
        .id(("palette-row", ix))
        .flex_shrink_0()
        .h(px(44.))
        .px(px(10.))
        .flex()
        .items_center()
        .gap(px(12.))
        .rounded(px(10.))
        .border_1()
        .border_color(transparent_black())
        .cursor_pointer()
        .when(selected, |this| {
            this.bg(palette.accent.alpha(0.22))
                .border_color(palette.accent.alpha(0.45))
        })
        .on_mouse_move(cx.listener(move |this, _, _, cx| this.hover(ix, cx)))
        .on_click(cx.listener(move |this, _, _, cx| this.run_at(ix, cx)))
        .child(
            div()
                .size(px(28.))
                .flex_shrink_0()
                .rounded(px(8.))
                .bg(palette.tint(command.color))
                .flex()
                .items_center()
                .justify_center()
                .child(glyph(command.icon, px(15.), command.color)),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .items_baseline()
                .gap(px(8.))
                .child(div().flex_shrink_0().child(title))
                .child(
                    div()
                        .min_w_0()
                        .truncate()
                        .text_size(px(12.))
                        .text_color(palette.text3)
                        .child(command.meta.clone()),
                ),
        )
        .when(selected, |this| {
            this.child(key_hint("↵", palette).bg(palette.field))
        })
}

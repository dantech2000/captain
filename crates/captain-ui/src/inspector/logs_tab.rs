use std::rc::Rc;

use captain_core::model::LogLevel;
use captain_core::store::LevelFilter;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::InspectorView;
use crate::theme::Palette;

/// Level filters and a virtual list of the log lines that pass them.
pub fn render(
    view: &InspectorView,
    palette: &Palette,
    cx: &mut Context<InspectorView>,
) -> impl IntoElement {
    let level = view.level();
    let lines = Rc::new(view.logs().filtered(level));
    let count = lines.len();
    let colors = *palette;
    let chips = LevelFilter::ALL.into_iter().map(|filter| {
        let selected = filter == level;
        div()
            .id(filter.label())
            .h(px(24.))
            .px(px(10.))
            .flex()
            .items_center()
            .rounded(px(12.))
            .border_1()
            .border_color(if selected {
                palette.accent.alpha(0.5)
            } else {
                palette.sep
            })
            .when(selected, |chip| chip.bg(palette.accent.alpha(0.2)))
            .text_size(px(11.))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(if selected {
                palette.text
            } else {
                palette.text2
            })
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, _, cx| this.set_level(filter, cx)))
            .child(filter.label())
    });

    let list = uniform_list("log-lines", count, move |range, _, _| {
        range
            .map(|ix| {
                let line = &lines[ix];
                let color = match line.level {
                    LogLevel::Info => colors.teal,
                    LogLevel::Warn => colors.orange,
                    LogLevel::Error => colors.red,
                };
                div()
                    .flex()
                    .gap(px(10.))
                    .px(px(12.))
                    .h(px(19.))
                    .items_center()
                    .border_l_2()
                    .border_color(if line.level == LogLevel::Info {
                        transparent_black()
                    } else {
                        color
                    })
                    .when(line.level == LogLevel::Error, |row| {
                        row.bg(colors.tint(colors.red))
                    })
                    .child(
                        div()
                            .w(px(40.))
                            .flex_shrink_0()
                            .text_color(color)
                            .child(line.level.label()),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .text_color(colors.text)
                            .child(line.text.clone()),
                    )
            })
            .collect()
    })
    .track_scroll(view.log_scroll())
    .flex_1();

    div()
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .gap(px(10.))
        .px(px(20.))
        .pt(px(14.))
        .pb(px(20.))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.))
                .children(chips)
                .child(div().flex_1())
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(palette.text2)
                        .child(format!("{count} lines")),
                ),
        )
        .child(
            div()
                .flex_1()
                .min_h_0()
                .flex()
                .flex_col()
                .py(px(8.))
                .rounded(px(10.))
                .bg(palette.terminal)
                .border_1()
                .border_color(palette.sep)
                .font_family(palette.mono())
                .text_size(px(11.))
                .child(list),
        )
}

use std::rc::Rc;

use captain_core::model::LogLevel;
use captain_core::store::{ProjectLogEntry, ServiceExit};
use chrono::{Local, TimeZone};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::theme::Palette;
use crate::widgets::list_scrollbar;

const ROW_HEIGHT: Pixels = px(20.);

/// The project log card: a header with the services and their colors, then the
/// entries in a virtual list.
pub fn render(
    rows: Rc<Vec<ProjectLogEntry>>,
    services: Vec<String>,
    following: bool,
    scroll: &UniformListScrollHandle,
    palette: &Palette,
) -> Stateful<Div> {
    let colors = *palette;
    let tags = Rc::new(services);
    let list_tags = tags.clone();
    let list = uniform_list("project-log", rows.len(), move |range, _, _| {
        range
            .map(|ix| row(&rows[ix], &list_tags, &colors))
            .collect()
    })
    .track_scroll(scroll)
    .flex_1();
    let (dot, state) = if following {
        (palette.green, "Following · kept across restarts")
    } else {
        (palette.gray, "Paused · scroll to the end to follow")
    };
    div()
        .id("project-log-card")
        .flex_1()
        .min_h(px(160.))
        .flex()
        .flex_col()
        .mx(px(28.))
        .mt(px(16.))
        .mb(px(16.))
        .rounded(px(14.))
        .bg(palette.terminal)
        .border_1()
        .border_color(palette.sep)
        .overflow_hidden()
        .child(
            div()
                .h(px(40.))
                .flex_shrink_0()
                .flex()
                .items_center()
                .gap(px(10.))
                .px(px(14.))
                .border_b_1()
                .border_color(palette.sep)
                .child(
                    div()
                        .text_size(px(12.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("All services"),
                )
                .children(tags.iter().enumerate().map(|(ix, name)| {
                    div()
                        .flex()
                        .items_center()
                        .gap(px(5.))
                        .text_size(px(11.))
                        .text_color(palette.text2)
                        .child(
                            div()
                                .size(px(7.))
                                .rounded(px(2.))
                                .bg(tag_color(ix, palette)),
                        )
                        .child(name.clone())
                }))
                .child(div().flex_1())
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .text_size(px(11.))
                        .text_color(palette.text2)
                        .child(div().size(px(6.)).rounded_full().bg(dot))
                        .child(state),
                ),
        )
        .child(
            div()
                .flex_1()
                .min_h_0()
                .flex()
                .flex_col()
                .py(px(6.))
                .font_family(palette.mono())
                .text_size(px(11.5))
                .child(list_scrollbar(list, scroll)),
        )
}

/// The color of the service at `ix` in the sorted list.
fn tag_color(ix: usize, palette: &Palette) -> Hsla {
    palette.labels[ix % palette.labels.len()]
}

fn clock(time: Option<i64>) -> String {
    time.and_then(|time| Local.timestamp_opt(time, 0).single())
        .map(|time| time.format("%H:%M:%S").to_string())
        .unwrap_or_default()
}

fn row(entry: &ProjectLogEntry, services: &[String], colors: &Palette) -> AnyElement {
    match entry {
        ProjectLogEntry::Line { service, line } => {
            let ix = services.iter().position(|s| s == service).unwrap_or(0);
            let text_color = match line.level {
                LogLevel::Error => colors.red,
                LogLevel::Warn => colors.warn_text,
                LogLevel::Info => colors.text,
            };
            div()
                .h(ROW_HEIGHT)
                .flex()
                .items_center()
                .gap(px(12.))
                .px(px(14.))
                .when(line.level == LogLevel::Error, |row| {
                    row.bg(colors.red.alpha(0.07))
                })
                .child(
                    div()
                        .w(px(56.))
                        .flex_shrink_0()
                        .text_color(colors.text3)
                        .child(clock(line.timestamp)),
                )
                .child(
                    div()
                        .w(px(72.))
                        .flex_shrink_0()
                        .truncate()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(tag_color(ix, colors))
                        .child(service.clone()),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .text_color(text_color)
                        .child(line.text.clone()),
                )
                .into_any_element()
        }
        ProjectLogEntry::Exit(exit) => divider(exit, colors).into_any_element(),
    }
}

/// A red rule with `worker exited 137 (out of memory) · 12:07:11`.
fn divider(exit: &ServiceExit, colors: &Palette) -> Div {
    let line = colors.red.alpha(0.4);
    let time = clock(exit.time);
    let text = if time.is_empty() {
        exit.label()
    } else {
        format!("{} · {time}", exit.label())
    };
    div()
        .h(ROW_HEIGHT)
        .flex()
        .items_center()
        .gap(px(10.))
        .px(px(14.))
        .text_size(px(11.))
        .text_color(colors.red)
        .child(div().w(px(24.)).h(px(1.)).bg(line))
        .child(text)
        .child(div().flex_1().h(px(1.)).bg(line))
}

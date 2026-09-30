use captain_core::store::LevelFilter;
use gpui_kit::assets::IconName;
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::{Icon, Sizable};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::LogsPane;
use crate::help::HelpExt;
use crate::theme::Palette;
use crate::widgets::icon_button;

/// The top row: the search field, the follow state, Copy, and Clear.
pub fn search_row(
    pane: &LogsPane,
    search: &Entity<InputState>,
    palette: &Palette,
    cx: &mut Context<LogsPane>,
) -> Div {
    let (copy_icon, copy_help) = if pane.copied() {
        (
            Icon::new(IconName::Check).text_color(palette.green),
            "Copied the lines shown to the clipboard.",
        )
    } else {
        (
            Icon::new(IconName::Copy),
            "Copy the lines shown, with the search and level filter applied.",
        )
    };
    div()
        .flex()
        .items_center()
        .gap(px(6.))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .h(px(26.))
                .pl(px(8.))
                .flex()
                .items_center()
                .gap(px(6.))
                .rounded(px(7.))
                .bg(palette.field)
                .child(
                    Icon::new(IconName::Search)
                        .size(px(12.))
                        .text_color(palette.text3),
                )
                .child(
                    div().flex_1().min_w_0().child(
                        Input::new(search)
                            .appearance(false)
                            .cleanable(true)
                            .aria_label("Search logs")
                            .small()
                            .px_0()
                            .text_size(px(12.)),
                    ),
                ),
        )
        .child(follow_toggle(pane.following(), palette, cx))
        .child(icon_button(
            "logs-copy",
            copy_icon,
            copy_help,
            palette,
            cx.listener(|this, _, _, cx| this.copy_visible(cx)),
        ))
        .child(icon_button(
            "logs-clear",
            IconName::Eraser,
            "Clear the lines in this view. The container keeps its logs.",
            palette,
            cx.listener(|this, _, _, cx| this.clear(cx)),
        ))
}

/// The second row: level chips, the time column toggle, and the line count.
pub fn filter_row(
    pane: &LogsPane,
    shown: usize,
    palette: &Palette,
    cx: &mut Context<LogsPane>,
) -> Div {
    let level = pane.level();
    let chips = LevelFilter::ALL.into_iter().map(|filter| {
        chip(filter.label(), filter == level, palette)
            .child(filter.label())
            .on_click(cx.listener(move |this, _, _, cx| this.set_level(filter, cx)))
            .help(match filter {
                LevelFilter::All => "Show lines of every level.".to_string(),
                _ => format!("Show only {} lines.", filter.label().to_lowercase()),
            })
    });
    let time = chip("logs-time", pane.show_time(), palette)
        .gap(px(4.))
        .child(Icon::new(IconName::Clock).size(px(11.)))
        .child("Time")
        .on_click(cx.listener(|this, _, _, cx| this.toggle_time(cx)))
        .help("Show or hide the time of each line.");

    div()
        .flex()
        .items_center()
        .gap(px(5.))
        .children(chips)
        .child(div().w(px(1.)).h(px(14.)).mx(px(2.)).bg(palette.sep))
        .child(time)
        .child(div().flex_1())
        .child(
            div()
                .flex_shrink_0()
                .text_size(px(11.))
                .text_color(palette.text2)
                .child(format!("{shown} of {} lines", pane.total())),
        )
}

/// A rounded filter chip with no content yet.
fn chip(id: &'static str, selected: bool, palette: &Palette) -> Stateful<Div> {
    div()
        .id(id)
        .h(px(22.))
        .px(px(8.))
        .flex()
        .items_center()
        .rounded(px(11.))
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
}

/// "Following" with a green dot, or "Paused". A click switches between them.
fn follow_toggle(following: bool, palette: &Palette, cx: &mut Context<LogsPane>) -> Stateful<Div> {
    let (label, color) = if following {
        ("Following", palette.green)
    } else {
        ("Paused", palette.orange)
    };
    let hover = palette.nav_selected;
    div()
        .id("logs-follow")
        .h(px(26.))
        .px(px(9.))
        .flex_shrink_0()
        .flex()
        .items_center()
        .gap(px(6.))
        .rounded(px(7.))
        .border_1()
        .border_color(palette.sep)
        .bg(palette.button)
        .text_size(px(11.))
        .font_weight(FontWeight::MEDIUM)
        .text_color(palette.text2)
        .cursor_pointer()
        .hover(move |style| style.bg(hover))
        .on_click(cx.listener(move |this, _, _, cx| this.set_following(!following, cx)))
        .help(if following {
            "New lines scroll into view. Click to pause at the current line."
        } else {
            "Paused at the current line. Click to follow new lines."
        })
        .child(if following {
            div()
                .size(px(6.))
                .rounded_full()
                .bg(color)
                .into_any_element()
        } else {
            Icon::new(IconName::Pause)
                .size(px(10.))
                .text_color(color)
                .into_any_element()
        })
        .child(label)
}

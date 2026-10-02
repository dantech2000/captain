use std::rc::Rc;

use captain_core::model::{LogLevel, LogStream};
use captain_core::store::LogMatch;
use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::LogsPane;
use crate::theme::Palette;
use crate::widgets::list_scrollbar;

/// Rows share this group so the copy icon shows on the hovered row only.
const ROW_GROUP: &str = "log-line";

/// A virtual list of the matched lines. With a zone `offset`, each row starts with
/// the local time. A right-click or the hover icon copies a row.
pub fn log_list(
    matches: Rc<Vec<LogMatch>>,
    offset: Option<i32>,
    scroll: &UniformListScrollHandle,
    palette: &Palette,
    pane: WeakEntity<LogsPane>,
) -> Div {
    let colors = *palette;
    let list = uniform_list("log-lines", matches.len(), move |range, _, _| {
        range
            .map(|ix| row(ix, &matches[ix], offset, &colors, pane.clone()))
            .collect()
    })
    .track_scroll(scroll)
    .flex_1();
    list_scrollbar(list, scroll)
}

/// The height of one row. A paused list shifts by it when the oldest line drops.
pub(super) const ROW_HEIGHT: Pixels = px(19.);

fn row(
    ix: usize,
    found: &LogMatch,
    offset: Option<i32>,
    colors: &Palette,
    pane: WeakEntity<LogsPane>,
) -> Stateful<Div> {
    let line = &found.line;
    let level_color = match line.level {
        LogLevel::Info => colors.teal,
        LogLevel::Warn => colors.orange,
        LogLevel::Error => colors.red,
    };
    // Stderr text gets a light red tint, on top of the level color in the gutter.
    let text_color = match line.stream {
        LogStream::Stdout => colors.text,
        LogStream::Stderr => colors.text.blend(colors.red.alpha(0.3)),
    };
    let highlight = HighlightStyle {
        background_color: Some(colors.orange.alpha(if colors.dark { 0.4 } else { 0.3 })),
        font_weight: Some(FontWeight::BOLD),
        ..Default::default()
    };
    let text = StyledText::new(line.text.clone())
        .with_highlights(found.ranges.iter().map(|range| (range.clone(), highlight)));
    let copy = line.copy_text(offset);
    let right_click = {
        let (pane, copy) = (pane.clone(), copy.clone());
        move |_: &MouseDownEvent, _: &mut Window, cx: &mut App| copy_line(&pane, &copy, cx)
    };
    let hover = colors.nav_selected;
    let icon_color = colors.text3;

    div()
        .id(("log-line", ix))
        .group(ROW_GROUP)
        .flex()
        .gap(px(10.))
        .px(px(12.))
        .h(ROW_HEIGHT)
        .items_center()
        .border_l_2()
        .border_color(if line.level == LogLevel::Info {
            transparent_black()
        } else {
            level_color
        })
        .when(line.level == LogLevel::Error, |row| {
            row.bg(colors.tint(colors.red))
        })
        .hover(move |style| style.bg(hover))
        .on_mouse_down(MouseButton::Right, right_click)
        .when_some(offset, |row, offset| {
            row.child(
                div()
                    .w(px(54.))
                    .flex_shrink_0()
                    .text_color(colors.text3)
                    .child(line.clock(offset).unwrap_or_default()),
            )
        })
        .child(
            div()
                .w(px(40.))
                .flex_shrink_0()
                .text_color(level_color)
                .child(line.level.label()),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .truncate()
                .text_color(text_color)
                .child(text),
        )
        .child(
            div()
                .id(("copy-log-line", ix))
                .flex_shrink_0()
                .cursor_pointer()
                .text_color(transparent_black())
                .group_hover(ROW_GROUP, move |style| style.text_color(icon_color))
                .on_click(move |_, _, cx| copy_line(&pane, &copy, cx))
                .child(Icon::new(IconName::Copy).size(px(11.))),
        )
}

fn copy_line(pane: &WeakEntity<LogsPane>, text: &str, cx: &mut App) {
    pane.update(cx, |pane, cx| pane.copy_to_clipboard(text.to_string(), cx))
        .ok();
}

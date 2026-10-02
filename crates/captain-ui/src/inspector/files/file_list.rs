use std::rc::Rc;

use captain_core::format::bytes_label;
use captain_core::model::{FileEntry, FileKind};
use chrono::{DateTime, Datelike, Local};
use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::files_pane::FilesPane;
use crate::theme::Palette;
use crate::widgets::list_scrollbar;

const SIZE_WIDTH: f32 = 56.;
const MODE_WIDTH: f32 = 72.;
const MODIFIED_WIDTH: f32 = 84.;
const ROW_HEIGHT: f32 = 26.;

/// The column header and a virtual list of the folder's entries. A click selects a
/// row; a double-click opens it.
pub fn render(
    entries: Rc<Vec<FileEntry>>,
    selected: Option<usize>,
    scroll: &UniformListScrollHandle,
    palette: &Palette,
    pane: WeakEntity<FilesPane>,
) -> Div {
    let colors = *palette;
    let now_year = Local::now().year();
    let list = uniform_list("file-entries", entries.len(), move |range, _, _| {
        range
            .map(|ix| {
                row(
                    ix,
                    &entries[ix],
                    selected == Some(ix),
                    now_year,
                    &colors,
                    &pane,
                )
            })
            .collect()
    })
    .track_scroll(scroll)
    .flex_1();

    div()
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .rounded(px(9.))
        .border_1()
        .border_color(palette.sep)
        .overflow_hidden()
        .text_size(px(11.))
        .child(cells(
            div()
                .h(px(24.))
                .border_b_1()
                .border_color(palette.sep)
                .text_color(palette.text3)
                .font_weight(FontWeight::SEMIBOLD),
            div().child("Name"),
            "Size".into(),
            "Mode".into(),
            "Modified".into(),
            palette.mono(),
        ))
        .child(list_scrollbar(list, scroll))
}

fn row(
    ix: usize,
    entry: &FileEntry,
    selected: bool,
    now_year: i32,
    colors: &Palette,
    pane: &WeakEntity<FilesPane>,
) -> Stateful<Div> {
    let icon = match entry.kind {
        _ if entry.opens => IconName::Folder,
        FileKind::Link => IconName::FileSymlink,
        FileKind::File => IconName::FileText,
        _ => IconName::File,
    };
    let size = if entry.kind == FileKind::File {
        bytes_label(entry.size)
    } else {
        "—".into()
    };
    let pane = pane.clone();
    let hover = colors.nav_selected;
    let name = div()
        .flex()
        .items_center()
        .gap(px(6.))
        .child(
            Icon::new(icon)
                .size(px(13.))
                .flex_shrink_0()
                .text_color(if entry.opens {
                    colors.accent_fg
                } else {
                    colors.text3
                }),
        )
        .child(div().min_w_0().truncate().child(entry.name.clone()));

    cells(
        div()
            .id(("file-entry", ix))
            .h(px(ROW_HEIGHT))
            .cursor_pointer()
            .when(selected, |row| {
                row.bg(colors.accent.alpha(if colors.dark { 0.2 } else { 0.12 }))
            })
            .when(!selected, |row| row.hover(move |style| style.bg(hover)))
            .on_click(move |event, window, cx| {
                pane.update(cx, |pane, cx| {
                    window.focus(&pane.focus, cx);
                    pane.select(ix, cx);
                    if event.click_count() >= 2 {
                        pane.open_selected(cx);
                    }
                })
                .ok();
            }),
        name,
        size,
        entry.mode_label(),
        modified_label(entry.modified, now_year),
        colors.mono(),
    )
    .text_color(colors.text)
}

/// Lays out the four columns: the name takes the rest of the width.
fn cells<E: ParentElement + Styled>(
    row: E,
    name: Div,
    size: String,
    mode: String,
    modified: String,
    mono: SharedString,
) -> E {
    row.w_full()
        .flex()
        .flex_shrink_0()
        .items_center()
        .gap(px(8.))
        .px(px(10.))
        .child(name.flex_1().min_w_0())
        .child(
            div()
                .w(px(SIZE_WIDTH))
                .flex_shrink_0()
                .text_right()
                .child(size),
        )
        .child(
            div()
                .w(px(MODE_WIDTH))
                .flex_shrink_0()
                .font_family(mono)
                .child(mode),
        )
        .child(
            div()
                .w(px(MODIFIED_WIDTH))
                .flex_shrink_0()
                .text_right()
                .child(modified),
        )
}

/// `Sep 29 14:02` for this year, `May 12 2025` for older times, in local time.
fn modified_label(seconds: i64, now_year: i32) -> String {
    let Some(time) = DateTime::from_timestamp(seconds, 0) else {
        return String::new();
    };
    let time = time.with_timezone(&Local);
    if time.year() == now_year {
        time.format("%b %d %H:%M").to_string()
    } else {
        time.format("%b %d %Y").to_string()
    }
}

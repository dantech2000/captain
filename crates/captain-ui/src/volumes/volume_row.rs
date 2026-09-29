use captain_core::format::bytes_label;
use captain_core::model::Volume;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::VolumesView;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, list_row, pill, select_mode, status_dot, text_button};

pub const SIZE_WIDTH: f32 = 80.;
pub const USAGE_WIDTH: f32 = 110.;
pub const CREATED_WIDTH: f32 = 92.;

/// One volume: usage dot, name, driver and mountpoint, size, users, and the creation
/// date, or Remove when selected. `highlighted` marks a row in a bulk selection.
pub fn render(
    volume: &Volume,
    selected: bool,
    highlighted: bool,
    removing: bool,
    handle: &Entity<VolumesView>,
    palette: &Palette,
) -> impl IntoElement {
    let dot = if volume.is_in_use() {
        palette.green
    } else {
        palette.gray
    };
    let size = volume.size_bytes.map_or_else(|| "—".into(), bytes_label);
    let select = handle.clone();
    let name = volume.name.clone();

    list_row(
        SharedString::from(format!("volume-{}", volume.name)),
        highlighted,
        palette,
    )
    .on_click(move |event, _, cx| {
        let mode = select_mode(event);
        select.update(cx, |view, cx| view.click_row(name.clone(), mode, cx));
    })
    .child(
        div()
            .flex_1()
            .min_w_0()
            .flex()
            .items_center()
            .gap(px(12.))
            .child(status_dot(dot, false, palette))
            .child(name_cell(volume, palette)),
    )
    .child(
        div()
            .w(px(SIZE_WIDTH))
            .flex_shrink_0()
            .text_right()
            .text_size(px(12.))
            .text_color(palette.text2)
            .child(size),
    )
    .child(
        div()
            .w(px(USAGE_WIDTH))
            .flex_shrink_0()
            .text_size(px(12.))
            .text_color(if volume.is_in_use() {
                palette.text
            } else {
                palette.text3
            })
            .child(volume.usage_label()),
    )
    .child(trailing_cell(volume, selected, removing, handle, palette))
}

fn name_cell(volume: &Volume, palette: &Palette) -> Div {
    div()
        .min_w_0()
        .flex()
        .flex_col()
        .gap(px(3.))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.))
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .truncate()
                        .child(volume.display_name().to_string()),
                )
                .when(volume.is_anonymous(), |this| {
                    this.child(pill("anonymous", palette.text2, palette.field))
                }),
        )
        .child(
            div()
                .font_family(palette.mono())
                .text_size(px(11.))
                .text_color(palette.text2)
                .truncate()
                .child(format!("{} · {}", volume.driver, volume.mountpoint)),
        )
}

fn trailing_cell(
    volume: &Volume,
    selected: bool,
    removing: bool,
    handle: &Entity<VolumesView>,
    palette: &Palette,
) -> Div {
    let cell = div()
        .w(px(CREATED_WIDTH))
        .flex_shrink_0()
        .flex()
        .justify_end()
        .items_center();
    if removing {
        return cell
            .text_size(px(12.))
            .text_color(palette.text3)
            .child("Removing...");
    }
    if !selected {
        return cell
            .text_size(px(12.))
            .text_color(palette.text3)
            .child(volume.created_date().to_string());
    }
    let handle = handle.clone();
    let name = volume.name.clone();
    cell.child(text_button(
        SharedString::from(format!("remove-volume-{}", volume.name)),
        "Remove",
        ButtonTone::Danger,
        volume.can_remove(),
        palette,
        move |_, _, cx| {
            cx.stop_propagation();
            handle.update(cx, |view, cx| view.remove(name.clone(), cx));
        },
    ))
}

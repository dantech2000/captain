use captain_core::format::bytes_label;
use captain_core::model::Volume;
use captain_core::store::{ResourceGroup, UsageFilter};
use gpui_kit::assets::IconName;
use gpui_kit::*;

use super::VolumesView;
use super::volume_row::{self, CREATED_WIDTH, SIZE_WIDTH, USAGE_WIDTH};
use crate::theme::Palette;
use crate::widgets::{Column, column_header, empty_note, group_card};

/// The column header and the volumes in project cards, or a note when there are none.
pub fn render(view: &VolumesView, cx: &mut Context<VolumesView>, palette: &Palette) -> AnyElement {
    if !view.loaded {
        return div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .text_color(palette.text2)
            .child("Loading volumes...")
            .into_any_element();
    }
    let groups = view.store.groups(view.filter);
    if groups.is_empty() {
        let (title, hint) = if view.filter == UsageFilter::All {
            (
                "No volumes yet",
                "Create one above, or run `docker volume create`.",
            )
        } else {
            (
                "No volumes match this filter",
                "Choose All to see every volume.",
            )
        };
        return empty_note(IconName::HardDrive, title, hint, palette).into_any_element();
    }

    let handle = cx.entity();
    let cards = groups
        .into_iter()
        .map(|group| card(group, view, &handle, palette));
    let columns = [
        Column {
            label: "Size",
            width: SIZE_WIDTH,
            right: true,
        },
        Column {
            label: "Used by",
            width: USAGE_WIDTH,
            right: false,
        },
        Column {
            label: "Created",
            width: CREATED_WIDTH,
            right: true,
        },
    ];
    div()
        .size_full()
        .flex()
        .flex_col()
        .child(column_header(&columns, palette))
        .child(
            div()
                .id("volume-list")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .px(px(12.))
                .pt(px(10.))
                .pb(px(16.))
                .child(div().flex().flex_col().gap(px(10.)).children(cards)),
        )
        .into_any_element()
}

fn card(
    group: ResourceGroup<Volume>,
    view: &VolumesView,
    handle: &Entity<VolumesView>,
    palette: &Palette,
) -> Div {
    let count = match group.items.len() {
        1 => "1 volume".to_string(),
        n => format!("{n} volumes"),
    };
    let size: Option<u64> = group
        .items
        .iter()
        .filter_map(|v| v.size_bytes)
        .reduce(|a, b| a + b);
    let summary = match size {
        Some(size) => format!("{count} · {}", bytes_label(size)),
        None => count,
    };
    let rows = group.items.iter().map(|volume| {
        let selected = view.selected.as_deref() == Some(volume.name.as_str());
        let removing = view.removing.contains(&volume.name);
        volume_row::render(volume, selected, removing, handle, palette).into_any_element()
    });
    group_card(group.project.as_deref(), summary, rows, palette)
}

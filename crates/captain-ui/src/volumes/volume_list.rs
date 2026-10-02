use captain_core::format::bytes_label;
use captain_core::model::Volume;
use captain_core::store::{ResourceGroup, UsageFilter};
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::*;

use super::VolumesView;
use super::volume_row::{self, CREATED_WIDTH, SIZE_WIDTH, USAGE_WIDTH};
use crate::engine_host::connection_screen;
use crate::icons::CaptainIcon;
use crate::theme::Palette;
use crate::widgets::{Column, column_header, empty_note, group_card, skeleton_rows};

/// The column header and the volumes in project cards, or a note when there are none,
/// or the engine connection screen while the engine does not answer.
pub fn render(view: &VolumesView, cx: &mut Context<VolumesView>, palette: &Palette) -> AnyElement {
    if let Some(screen) = connection_screen(view.workspace.read(cx), palette, cx) {
        return screen;
    }
    if !view.loaded {
        return skeleton_rows(6, px(50.)).into_any_element();
    }
    let groups = view.visible_groups();
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
        return empty_note(CaptainIcon::Volume, title, hint, palette).into_any_element();
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
                .overflow_y_scrollbar()
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
        let highlighted = selected || view.checked.contains(&volume.name);
        let removing = view.removing.contains(&volume.name);
        volume_row::render(
            volume,
            selected,
            highlighted,
            removing,
            view.generation,
            handle,
            palette,
        )
        .into_any_element()
    });
    group_card(group.project.as_deref(), summary, rows, palette)
}

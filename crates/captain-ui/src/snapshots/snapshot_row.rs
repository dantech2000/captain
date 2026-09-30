use captain_core::format::bytes_label;
use captain_core::snapshot::Snapshot;
use chrono::TimeZone;
use gpui_kit::*;

use super::{SnapshotsModel, delete_dialog, edit_dialog, restore_dialog};
use crate::help::HelpExt;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, text_button};

/// One snapshot: name, description, date and size, and Restore, Edit, and Delete.
pub fn render(
    model: &Entity<SnapshotsModel>,
    snapshot: &Snapshot,
    enabled: bool,
    palette: &Palette,
) -> AnyElement {
    let metadata = &snapshot.metadata;
    let details = format!(
        "{} · {}",
        date(metadata.created),
        bytes_label(metadata.disk_allocated)
    );
    let name = &metadata.name;
    let restore = {
        let (model, snapshot) = (model.clone(), snapshot.clone());
        text_button(
            SharedString::from(format!("snapshot-restore-{}", snapshot.id)),
            "Restore…",
            ButtonTone::Accent,
            enabled,
            palette,
            move |_, window, cx| restore_dialog::open(model.clone(), snapshot.clone(), window, cx),
        )
        .help(format!(
            "Put Captain Engine back to the state of {name}. Captain asks first."
        ))
    };
    let edit = {
        let (model, snapshot) = (model.clone(), snapshot.clone());
        text_button(
            SharedString::from(format!("snapshot-edit-{}", snapshot.id)),
            "Edit…",
            ButtonTone::Accent,
            enabled,
            palette,
            move |_, window, cx| edit_dialog::open(model.clone(), snapshot.clone(), window, cx),
        )
        .help(format!("Change the name and description of {name}."))
    };
    let delete = {
        let (model, snapshot) = (model.clone(), snapshot.clone());
        text_button(
            SharedString::from(format!("snapshot-delete-{}", snapshot.id)),
            "Delete…",
            ButtonTone::Danger,
            enabled,
            palette,
            move |_, window, cx| delete_dialog::open(model.clone(), snapshot.clone(), window, cx),
        )
        .help(format!("Delete the snapshot {name}. Captain asks first."))
    };
    div()
        .min_h(px(52.))
        .px(px(14.))
        .py(px(8.))
        .flex()
        .items_center()
        .gap(px(16.))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(
                    div()
                        .font_weight(FontWeight::MEDIUM)
                        .truncate()
                        .child(metadata.name.clone()),
                )
                .children((!metadata.description.is_empty()).then(|| {
                    div()
                        .text_size(px(12.))
                        .text_color(palette.text2)
                        .child(metadata.description.clone())
                }))
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(palette.text3)
                        .child(details),
                ),
        )
        .child(restore)
        .child(edit)
        .child(delete)
        .into_any_element()
}

/// Unix seconds as the local date and time, for example `2026-09-29 14:05`.
fn date(seconds: u64) -> String {
    chrono::Local
        .timestamp_opt(seconds as i64, 0)
        .single()
        .map_or_else(String::new, |time| {
            time.format("%Y-%m-%d %H:%M").to_string()
        })
}

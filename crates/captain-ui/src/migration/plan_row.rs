use captain_core::format::bytes_label;
use captain_core::migration::{MigrationItem, PlanEntry};
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::*;

use super::assistant::MigrationAssistant;
use crate::theme::Palette;
use crate::widgets::pill;

/// One item in the plan: a checkbox with the name, a detail line, the size, and for
/// a container with changes inside it, a Snapshot checkbox.
pub fn plan_row(
    ix: usize,
    entry: &PlanEntry,
    palette: &Palette,
    cx: &mut Context<MigrationAssistant>,
) -> AnyElement {
    let key = entry.item.key();
    let this = cx.entity().downgrade();
    let toggle = {
        let key = key.clone();
        let this = this.clone();
        Checkbox::new(("migration-item", ix))
            .checked(entry.selected)
            .on_click(move |_, _, cx| {
                this.update(cx, |view, cx| view.toggle(&key, cx)).ok();
            })
    };
    let size = entry.item.size();
    let mut row = div()
        .min_h(px(34.))
        .px(px(12.))
        .py(px(6.))
        .flex()
        .items_center()
        .gap(px(10.))
        .child(toggle)
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .child(
                    div()
                        .text_size(px(12.))
                        .font_weight(FontWeight::MEDIUM)
                        .truncate()
                        .child(entry.item.label()),
                )
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(palette.text2)
                        .truncate()
                        .child(detail(&entry.item)),
                ),
        );
    if entry.item.loses_changes() {
        row = row.child(pill(
            "Changes inside",
            palette.orange,
            palette.tint(palette.orange),
        ));
    }
    if matches!(entry.item, MigrationItem::Container { .. }) && entry.item.loses_changes() {
        let snapshot = Checkbox::new(("migration-snapshot", ix))
            .label("Snapshot")
            .checked(entry.snapshot)
            .on_click(move |checked, _, cx| {
                let checked = *checked;
                this.update(cx, |view, cx| view.set_snapshot(&key, checked, cx))
                    .ok();
            });
        row = row.child(snapshot);
    }
    row.child(
        div()
            .w(px(70.))
            .flex()
            .justify_end()
            .text_size(px(11.))
            .text_color(palette.text2)
            .child(if size > 0 {
                bytes_label(size)
            } else {
                String::new()
            }),
    )
    .into_any_element()
}

/// The line under the name.
fn detail(item: &MigrationItem) -> String {
    match item {
        MigrationItem::Network { driver, .. } => format!("{driver} network"),
        MigrationItem::Volume { size: None, .. } => "Size unknown".into(),
        MigrationItem::Volume { .. } => "Volume".into(),
        MigrationItem::Image { tags, in_use, .. } => {
            let usage = if *in_use {
                "In use"
            } else {
                "Not used by a container"
            };
            match tags.len() {
                0 | 1 => usage.into(),
                n => format!("{usage} · {} more tags", n - 1),
            }
        }
        MigrationItem::ComposeProject {
            containers,
            files_exist,
            ..
        } => {
            let count = match containers.len() {
                1 => "1 container".to_string(),
                n => format!("{n} containers"),
            };
            if *files_exist {
                format!("{count} · recreated with docker compose up -d")
            } else {
                format!("{count} · files not on this computer, so recreated one by one")
            }
        }
        MigrationItem::Container { image, running, .. } => {
            let state = if *running { "running" } else { "stopped" };
            format!("{image} · {state}")
        }
    }
}

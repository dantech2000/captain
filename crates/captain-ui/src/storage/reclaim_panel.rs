use captain_core::format::bytes_label;
use captain_core::storage::{ReclaimGroup, ReclaimPlan};
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::*;

use super::{StorageModel, preview_dialog};
use crate::help::HelpExt;
use crate::icons::{CaptainIcon, cap_icon};
use crate::settings;
use crate::theme::Palette;
use crate::widgets::primary_button;

/// "Free up space": a checkbox per group with its size, the snapshot and weekly
/// options, and the button that opens the preview.
pub fn render(
    handle: &Entity<StorageModel>,
    model: &StorageModel,
    can_snapshot: bool,
    weekly: bool,
    palette: &Palette,
) -> Div {
    let plan = model.plan();
    let total = model.selected_bytes();
    let count = plan.selected(&model.checked).count();
    let groups = ReclaimGroup::ALL
        .into_iter()
        .map(|group| group_row(handle, plan, group, model.is_checked(group), palette));

    let snapshot = can_snapshot.then(|| {
        let handle = handle.clone();
        option(
            "storage-snapshot-first",
            "Take a snapshot first so you can undo. The engine restarts, about 10 seconds.",
            "Save a snapshot of Captain Engine before the cleanup. Restore it on the Snapshots page.",
            model.snapshot_first,
            move |checked, _, cx| {
                handle.update(cx, |model, cx| model.set_snapshot_first(*checked, cx));
            },
        )
    });
    let weekly = option(
        "storage-weekly",
        "Do this every week for build cache older than 14 days",
        "Once a week, while the engine runs, remove build cache that no build used in 14 days.",
        weekly,
        |checked, _, cx| {
            let on = *checked;
            settings::update(cx, |settings| settings.weekly_build_cache_cleanup = on);
        },
    );

    let action: AnyElement = match model.step() {
        Some(step) => div()
            .h(px(34.))
            .flex()
            .items_center()
            .justify_center()
            .gap(px(8.))
            .text_color(palette.text2)
            .child(Spinner::new().color(palette.text2))
            .child(step)
            .into_any_element(),
        None => {
            let handle = handle.clone();
            let label = format!(
                "Review {} {}, free {}",
                count,
                items(count),
                bytes_label(total)
            );
            primary_button(
                "storage-review",
                label,
                "List every item the cleanup removes, then confirm.",
                count > 0,
                palette,
                move |_, window, cx| preview_dialog::open(handle.clone(), window, cx),
            )
            .into_any_element()
        }
    };

    div()
        .w(px(420.))
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap(px(12.))
        .p(px(18.))
        .rounded(px(16.))
        .bg(palette.side)
        .border_1()
        .border_color(palette.sep)
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(10.))
                .child(cap_icon(CaptainIcon::Reclaim, px(20.), palette.orange))
                .child(
                    div()
                        .flex_1()
                        .text_size(px(15.))
                        .font_weight(FontWeight::BOLD)
                        .child("Free up space"),
                )
                .child(
                    div()
                        .text_size(px(20.))
                        .font_weight(FontWeight::BOLD)
                        .text_color(palette.warn_text)
                        .child(bytes_label(total)),
                ),
        )
        .children(groups)
        .child(div().h(px(4.)))
        .children(snapshot)
        .child(weekly)
        .child(action)
        .child(
            div()
                .text_size(px(11.))
                .text_color(palette.text3)
                .text_center()
                .child("Nothing a container uses is removed. Named volumes stay unless you check them."),
        )
}

fn group_row(
    handle: &Entity<StorageModel>,
    plan: &ReclaimPlan,
    group: ReclaimGroup,
    checked: bool,
    palette: &Palette,
) -> Stateful<Div> {
    let risky = group.has_data();
    let handle = handle.clone();
    let (border, bg, note_color) = if risky {
        (palette.red.alpha(0.3), palette.red.alpha(0.06), palette.red)
    } else {
        (palette.sep, palette.card, palette.text3)
    };
    div()
        .id(SharedString::from(format!("storage-group-{group:?}")))
        .flex()
        .items_start()
        .gap(px(12.))
        .px(px(12.))
        .py(px(10.))
        .rounded(px(10.))
        .border_1()
        .border_color(border)
        .bg(bg)
        .child(
            Checkbox::new(SharedString::from(format!("storage-check-{group:?}")))
                .checked(checked)
                .on_click(move |_, _, cx| {
                    handle.update(cx, |model, cx| model.toggle(group, cx));
                }),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(3.))
                .child(div().font_weight(FontWeight::SEMIBOLD).child(group.label()))
                .child(
                    div()
                        .text_size(px(11.5))
                        .text_color(note_color)
                        .child(note(plan, group)),
                ),
        )
        .child(
            div()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(palette.text2)
                .child(bytes_label(plan.group_bytes(group))),
        )
        .help(group_help(group))
}

fn option(
    id: &'static str,
    label: &'static str,
    help: &'static str,
    checked: bool,
    on_click: impl Fn(&bool, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    div()
        .id(SharedString::from(format!("{id}-row")))
        .text_size(px(12.))
        .child(
            Checkbox::new(id)
                .label(label)
                .checked(checked)
                .on_click(on_click),
        )
        .help(help)
}

/// The line under a group's name: what it holds, or that it is empty.
fn note(plan: &ReclaimPlan, group: ReclaimGroup) -> String {
    let names: Vec<&str> = plan.group(group).map(|item| item.name.as_str()).collect();
    let Some(first) = names.first() else {
        return "Nothing to remove".into();
    };
    let more = match names.len() {
        1 => String::new(),
        n => format!(" and {} more", n - 1),
    };
    match group {
        ReclaimGroup::OldBuildCache => "Made again when you next build".into(),
        ReclaimGroup::DanglingImages => {
            format!("{} with no tag and no container", images(names.len()))
        }
        ReclaimGroup::UnusedImages => format!("{first}{more} · pulled again when needed"),
        ReclaimGroup::OldStoppedContainers => format!("{first}{more}"),
        ReclaimGroup::UnusedVolumes if names.len() == 1 => {
            format!("{first} may hold data. Removing it cannot be undone without a snapshot.")
        }
        ReclaimGroup::UnusedVolumes => format!(
            "{first}{more} may hold data. Removing them cannot be undone without a snapshot."
        ),
    }
}

fn group_help(group: ReclaimGroup) -> &'static str {
    match group {
        ReclaimGroup::OldBuildCache => {
            "Remove build cache that no build used in 14 days. Builds make it again."
        }
        ReclaimGroup::DanglingImages => "Remove images with no tag that no container uses.",
        ReclaimGroup::UnusedImages => {
            "Remove tagged images that no container uses. A pull or a run gets them again."
        }
        ReclaimGroup::OldStoppedContainers => {
            "Remove stopped containers created more than 3 days ago, with their files."
        }
        ReclaimGroup::UnusedVolumes => {
            "Remove volumes that no container mounts. Their data is lost."
        }
    }
}

fn images(count: usize) -> String {
    if count == 1 {
        "1 image".into()
    } else {
        format!("{count} images")
    }
}

fn items(count: usize) -> &'static str {
    if count == 1 { "item" } else { "items" }
}

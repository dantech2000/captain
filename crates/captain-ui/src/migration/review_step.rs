use captain_core::format::bytes_label;
use captain_core::migration::{
    DEFAULT_THROUGHPUT, DiskCheck, DiskVerdict, Estimate, ImageChoice, MigrationPlan, Step,
    duration_label,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::assistant::MigrationAssistant;
use super::plan_row::plan_row;
use super::view::footer_row;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, Segment, section_note, segmented, text_button, titled_section};

/// Step 2: what the source holds, grouped by step, with the estimate and warnings.
pub fn body(
    view: &MigrationAssistant,
    palette: &Palette,
    cx: &mut Context<MigrationAssistant>,
) -> Div {
    let Some(plan) = &view.plan else {
        return div();
    };
    if plan.is_empty() {
        return section_note("The engine holds nothing to copy.", palette);
    }
    let mut body = div()
        .flex()
        .flex_col()
        .gap(px(16.))
        .child(estimate(plan, view.free, palette))
        .children(lost_changes(plan, palette))
        .children(live_volumes(plan, palette));
    let mut ix = 0;
    for step in Step::ALL {
        let entries: Vec<_> = plan.step(step).collect();
        if entries.is_empty() {
            continue;
        }
        let bytes: u64 = entries
            .iter()
            .filter(|e| e.selected)
            .map(|e| e.item.size())
            .sum();
        let selected = entries.iter().filter(|e| e.selected).count();
        let mut note = format!("{selected} of {}", entries.len());
        if bytes > 0 {
            note.push_str(&format!(" · {}", bytes_label(bytes)));
        }
        let sep = palette.sep;
        let rows = entries.into_iter().enumerate().map(|(n, entry)| {
            let row = plan_row(ix + n, entry, palette, cx);
            div()
                .when(n > 0, |d| d.border_t_1().border_color(sep))
                .child(row)
        });
        let rows: Vec<Div> = rows.collect();
        ix += rows.len();
        let mut list = div().flex().flex_col();
        if step == Step::Images {
            list = list.child(image_choice(plan.image_choice(), palette, cx));
        }
        let card = div()
            .rounded(px(10.))
            .bg(palette.group)
            .border_1()
            .border_color(palette.sep)
            .children(rows);
        body = body.child(titled_section(
            step.label(),
            Some(note),
            list.child(card),
            palette,
        ));
    }
    body
}

/// The size, the time, and the free-space check.
fn estimate(plan: &MigrationPlan, free: Option<u64>, palette: &Palette) -> Div {
    let bytes = plan.total_bytes();
    let estimate = Estimate::new(bytes, DEFAULT_THROUGHPUT);
    let summary = format!(
        "About {} to copy from {}, {}.",
        bytes_label(bytes),
        plan.source,
        duration_label(estimate.seconds)
    );
    let check = DiskCheck { bytes, free };
    let (text, color) = match check.verdict() {
        DiskVerdict::Unknown => (
            "Captain could not read the free space in the target.".to_string(),
            palette.text2,
        ),
        DiskVerdict::Enough => (
            format!(
                "{} free in the target: enough.",
                bytes_label(free.unwrap_or(0))
            ),
            palette.green,
        ),
        DiskVerdict::Tight => (
            format!(
                "{} free in the target. The copy fits, but an image load briefly needs about twice its size.",
                bytes_label(free.unwrap_or(0))
            ),
            palette.orange,
        ),
        DiskVerdict::NotEnough => (
            format!(
                "Only {} free in the target. Select less, or give the engine a larger disk.",
                bytes_label(free.unwrap_or(0))
            ),
            palette.red,
        ),
    };
    div()
        .flex()
        .flex_col()
        .gap(px(4.))
        .text_size(px(12.))
        .child(summary)
        .child(div().text_color(color).child(text))
        .child(
            div()
                .text_size(px(11.))
                .text_color(palette.text3)
                .child("Sizes are estimates. An image can export larger than the engine lists it."),
        )
}

/// The warning about changes made inside containers, when any selected item loses them.
fn lost_changes(plan: &MigrationPlan, palette: &Palette) -> Option<Div> {
    let names: Vec<String> = plan
        .lost_changes()
        .iter()
        .map(|item| item.label())
        .collect();
    if names.is_empty() {
        return None;
    }
    let text = format!(
        "Changes made inside these containers, outside volumes, are not copied: {}. \
         Turn on Snapshot for a container to keep them. A snapshot writes a temporary \
         captain-migrate image in the old engine, which Captain removes after the copy.",
        names.join(", ")
    );
    Some(
        div()
            .p(px(10.))
            .rounded(px(8.))
            .bg(palette.tint(palette.orange))
            .text_size(px(12.))
            .text_color(palette.text)
            .child(text),
    )
}

/// The warning about volumes that running containers are writing to.
fn live_volumes(plan: &MigrationPlan, palette: &Palette) -> Option<Div> {
    let volumes = plan.live_volumes();
    if volumes.is_empty() {
        return None;
    }
    let list: Vec<String> = volumes
        .iter()
        .map(|(name, users)| format!("{name} (used by {})", users.join(", ")))
        .collect();
    let text = format!(
        "These volumes belong to running containers: {}. A copy made while a program \
         writes (a database, for example) may not be clean. For a clean copy, turn on \
         Switch over for those containers, or stop them in the old engine first.",
        list.join("; ")
    );
    Some(
        div()
            .p(px(10.))
            .rounded(px(8.))
            .bg(palette.tint(palette.orange))
            .text_size(px(12.))
            .text_color(palette.text)
            .child(text),
    )
}

fn image_choice(
    choice: ImageChoice,
    palette: &Palette,
    cx: &mut Context<MigrationAssistant>,
) -> Div {
    let segments = [
        (
            ImageChoice::All,
            "All images",
            "Copy every image, also the ones no container uses.",
        ),
        (
            ImageChoice::InUse,
            "Only in use",
            "Copy only the images that a container uses.",
        ),
    ]
    .into_iter()
    .map(|(value, label, help)| {
        let this = cx.entity().downgrade();
        Segment {
            label: label.into(),
            selected: choice == value,
            help: help.into(),
            on_click: Box::new(move |_, cx| {
                this.update(cx, |view, cx| view.set_image_choice(value, cx))
                    .ok();
            }),
        }
    })
    .collect();
    div()
        .pb(px(8.))
        .flex()
        .child(segmented("migration-images", segments, palette))
}

pub fn footer(
    view: &MigrationAssistant,
    palette: &Palette,
    cx: &mut Context<MigrationAssistant>,
) -> Div {
    let plan = view.plan.as_ref();
    let selected = plan.map_or(0, |plan| plan.selected().count());
    let fits = plan.is_none_or(|plan| {
        let check = DiskCheck {
            bytes: plan.total_bytes(),
            free: view.free,
        };
        check.verdict() != DiskVerdict::NotEnough
    });
    let back = text_button(
        "migration-back",
        "Back",
        ButtonTone::Accent,
        true,
        palette,
        cx.listener(|view, _, _, cx| view.back_to_choose(cx)),
    );
    let start = text_button(
        "migration-start",
        format!("Copy {selected} items"),
        ButtonTone::Accent,
        selected > 0 && fits,
        palette,
        cx.listener(|view, _, window, cx| view.confirm_start(window, cx)),
    );
    let switching = plan.map_or(0, |plan| plan.switch_overs().len());
    let note = match switching {
        0 => "Your old engine is not changed.".to_string(),
        1 => "The switched item is stopped in your old engine. Nothing is deleted.".to_string(),
        n => format!("{n} switched items are stopped in your old engine. Nothing is deleted."),
    };
    footer_row(
        Some(note),
        vec![back.into_any_element(), start.into_any_element()],
        palette,
    )
}

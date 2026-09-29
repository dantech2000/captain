use captain_core::format::bytes_label;
use captain_core::migration::{RunEntry, StepStatus};
use gpui_kit::assets::IconName;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::{Icon, Sizable};
use gpui_kit::*;

use super::assistant::MigrationAssistant;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, progress_bar, text_button};

/// One item of the run: a status icon, the name and step, the status text, a bar
/// while it copies, and Retry when it failed.
pub fn run_row(
    ix: usize,
    entry: &RunEntry,
    palette: &Palette,
    cx: &mut Context<MigrationAssistant>,
) -> AnyElement {
    let (text, color) = status_text(entry, palette);
    let mut info = div()
        .flex_1()
        .min_w_0()
        .flex()
        .flex_col()
        .gap(px(3.))
        .child(
            div()
                .flex()
                .gap(px(6.))
                .text_size(px(12.))
                .child(
                    div()
                        .font_weight(FontWeight::MEDIUM)
                        .truncate()
                        .child(entry.item.label()),
                )
                .child(
                    div()
                        .text_color(palette.text3)
                        .child(entry.item.step().label()),
                ),
        )
        .child(div().text_size(px(11.)).text_color(color).child(text));
    if let StepStatus::Running { .. } = entry.status {
        info = info.child(progress_bar(
            entry.status.fraction(),
            palette.accent,
            palette,
        ));
    }
    let mut row = div()
        .px(px(12.))
        .py(px(8.))
        .flex()
        .items_center()
        .gap(px(10.))
        .child(status_icon(&entry.status, palette))
        .child(info);
    if let StepStatus::Failed(_) = entry.status {
        row = row.child(text_button(
            ("migration-retry", ix),
            "Retry",
            ButtonTone::Accent,
            true,
            palette,
            cx.listener(move |view, _, _, cx| view.retry(ix, cx)),
        ));
    }
    row.into_any_element()
}

pub fn status_icon(status: &StepStatus, palette: &Palette) -> AnyElement {
    let icon = |name: IconName, color: Hsla| {
        Icon::new(name)
            .size(px(16.))
            .text_color(color)
            .into_any_element()
    };
    let icon = match status {
        StepStatus::Pending => icon(IconName::Clock, palette.text3),
        StepStatus::Running { .. } => Spinner::new()
            .small()
            .color(palette.accent)
            .into_any_element(),
        StepStatus::Done => icon(IconName::CircleCheck, palette.green),
        StepStatus::Failed(_) => icon(IconName::CircleX, palette.red),
        StepStatus::Skipped(_) => icon(IconName::Ban, palette.gray),
    };
    div()
        .size(px(18.))
        .flex()
        .items_center()
        .justify_center()
        .child(icon)
        .into_any_element()
}

fn status_text(entry: &RunEntry, palette: &Palette) -> (String, Hsla) {
    match &entry.status {
        StepStatus::Pending => ("Waiting".into(), palette.text3),
        StepStatus::Running { done: 0, .. } => ("Starting…".into(), palette.text2),
        StepStatus::Running { done, total } if *total > 0 => (
            format!("{} of {}", bytes_label(*done), bytes_label(*total)),
            palette.text2,
        ),
        StepStatus::Running { done, .. } => {
            (format!("{} copied", bytes_label(*done)), palette.text2)
        }
        StepStatus::Done => (
            entry.note.clone().unwrap_or_else(|| "Copied".into()),
            palette.green,
        ),
        StepStatus::Skipped(reason) => (format!("Skipped. {reason}"), palette.text2),
        StepStatus::Failed(error) => (error.clone(), palette.red),
    }
}

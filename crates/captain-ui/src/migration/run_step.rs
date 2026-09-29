use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::assistant::MigrationAssistant;
use super::run_row::run_row;
use super::view::footer_row;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, progress_bar, text_button};

/// Step 3: the overall progress and one row per item.
pub fn body(
    view: &MigrationAssistant,
    palette: &Palette,
    cx: &mut Context<MigrationAssistant>,
) -> Div {
    let run = &view.run;
    let summary = run.summary();
    let finished = run.entries.len() - summary.pending;
    let sep = palette.sep;
    let rows: Vec<Div> = run
        .entries
        .iter()
        .enumerate()
        .map(|(ix, entry)| {
            div()
                .when(ix > 0, |d| d.border_t_1().border_color(sep))
                .child(run_row(ix, entry, palette, cx))
        })
        .collect();
    div()
        .flex()
        .flex_col()
        .gap(px(12.))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(6.))
                .text_size(px(12.))
                .child(format!(
                    "{finished} of {} items finished",
                    run.entries.len()
                ))
                .child(progress_bar(run.fraction(), palette.accent, palette)),
        )
        .child(
            div()
                .rounded(px(10.))
                .bg(palette.group)
                .border_1()
                .border_color(palette.sep)
                .children(rows),
        )
}

pub fn footer(
    view: &MigrationAssistant,
    palette: &Palette,
    cx: &mut Context<MigrationAssistant>,
) -> Div {
    let mut buttons = Vec::new();
    if view.is_copying() {
        buttons.push(
            text_button(
                "migration-stop",
                "Stop",
                ButtonTone::Danger,
                true,
                palette,
                cx.listener(|view, _, _, cx| view.stop(cx)),
            )
            .into_any_element(),
        );
    } else {
        buttons.push(
            text_button(
                "migration-plan",
                "Back to plan",
                ButtonTone::Accent,
                true,
                palette,
                cx.listener(|view, _, _, cx| view.back_to_review(cx)),
            )
            .into_any_element(),
        );
        if view.run.next_pending().is_some() {
            buttons.push(
                text_button(
                    "migration-resume",
                    "Resume",
                    ButtonTone::Accent,
                    true,
                    palette,
                    cx.listener(|view, _, _, cx| view.resume(cx)),
                )
                .into_any_element(),
            );
        }
    }
    let note = if view.is_copying() {
        "Stop ends the current item and removes its partial copy."
    } else {
        "Stopped. Resume copies the items that are not done."
    };
    footer_row(Some(note.into()), buttons, palette)
}

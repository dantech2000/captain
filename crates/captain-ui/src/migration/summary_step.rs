use captain_core::migration::StepStatus;
use gpui_kit::assets::IconName;
use gpui_kit::component::{Icon, WindowExt};
use gpui_kit::*;

use super::assistant::MigrationAssistant;
use super::run_row::run_row;
use super::view::footer_row;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, text_button, titled_section};

/// Step 4: the counts, the items that failed or were skipped, and the promise that
/// the old engine is unchanged.
pub fn body(
    view: &MigrationAssistant,
    palette: &Palette,
    cx: &mut Context<MigrationAssistant>,
) -> Div {
    let summary = view.run.summary();
    let snapshots = view
        .run
        .entries
        .iter()
        .filter(|e| e.snapshot && e.status == StepStatus::Done)
        .count();
    let mut lines = vec![format!("Copied {} items.", summary.copied)];
    if summary.skipped > 0 {
        lines.push(format!(
            "Skipped {}, because the target already had them or they hold no data here.",
            summary.skipped
        ));
    }
    if summary.failed > 0 {
        lines.push(format!(
            "{} failed. Retry them below, or copy them later.",
            summary.failed
        ));
    }
    if summary.pending > 0 {
        lines.push(format!(
            "{} were not copied, because the run stopped.",
            summary.pending
        ));
    }
    let mut unchanged = "Your old engine was not changed.".to_string();
    if snapshots > 0 {
        unchanged.push_str(&format!(
            " Captain made {snapshots} temporary snapshot images there and removed them again."
        ));
    }
    let headline = div()
        .flex()
        .items_center()
        .gap(px(8.))
        .text_size(px(14.))
        .font_weight(FontWeight::SEMIBOLD)
        .child(
            Icon::new(IconName::CircleCheck)
                .size(px(18.))
                .text_color(palette.green),
        )
        .child(unchanged);
    let mut body = div().flex().flex_col().gap(px(14.)).child(headline).child(
        div()
            .flex()
            .flex_col()
            .gap(px(4.))
            .text_size(px(12.))
            .text_color(palette.text2)
            .children(lines),
    );
    let notable: Vec<AnyElement> = view
        .run
        .entries
        .iter()
        .enumerate()
        .filter(|(_, e)| !matches!(e.status, StepStatus::Done))
        .map(|(ix, entry)| run_row(ix, entry, palette, cx))
        .collect();
    if !notable.is_empty() {
        let card = div()
            .rounded(px(10.))
            .bg(palette.group)
            .border_1()
            .border_color(palette.sep)
            .children(notable);
        body = body.child(titled_section("Not copied", None, card, palette));
    }
    body
}

pub fn footer(
    view: &MigrationAssistant,
    palette: &Palette,
    cx: &mut Context<MigrationAssistant>,
) -> Div {
    let mut buttons = Vec::new();
    if view.run.summary().failed > 0 {
        buttons.push(
            text_button(
                "migration-retry-failed",
                "Retry failed",
                ButtonTone::Accent,
                true,
                palette,
                cx.listener(|view, _, _, cx| view.retry_failed(cx)),
            )
            .into_any_element(),
        );
    }
    buttons.push(
        text_button(
            "migration-done",
            "Done",
            ButtonTone::Accent,
            true,
            palette,
            |_, window, cx| window.close_dialog(cx),
        )
        .into_any_element(),
    );
    let note = view.source().map(|source| format!("From {source}"));
    footer_row(note, buttons, palette)
}

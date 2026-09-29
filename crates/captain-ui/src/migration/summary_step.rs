use captain_core::migration::{RollbackStatus, StepStatus};
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
    if summary.switched > 0 {
        lines.push(match summary.switched {
            1 => "Switched over 1: it runs here now.".to_string(),
            n => format!("Switched over {n}: they run here now."),
        });
    }
    // Rolled-back items run in the old engine again, so they do not count.
    let stopped = view
        .run
        .switched()
        .filter(|(_, e)| {
            e.switch_over
                .as_ref()
                .is_some_and(|p| p.rollback != RollbackStatus::Done)
        })
        .count();
    let mut unchanged = match stopped {
        0 => "Your old engine was not changed.".to_string(),
        1 => "Captain stopped 1 item in your old engine. It deleted nothing there.".to_string(),
        n => format!("Captain stopped {n} items in your old engine. It deleted nothing there."),
    };
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
    if let Some(section) = switched(view, palette, cx) {
        body = body.child(section);
    }
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

/// The items that switched over, each with Roll back, and what that means for the
/// old engine.
fn switched(
    view: &MigrationAssistant,
    palette: &Palette,
    cx: &mut Context<MigrationAssistant>,
) -> Option<Div> {
    let rows: Vec<AnyElement> = view
        .run
        .switched()
        .filter(|(_, e)| e.status == StepStatus::Done)
        .map(|(ix, entry)| run_row(ix, entry, palette, cx))
        .collect();
    if rows.is_empty() {
        return None;
    }
    let note = "The old containers are stopped, not deleted. Roll back stops the copy here \
                and starts the original again.";
    let card = div()
        .rounded(px(10.))
        .bg(palette.group)
        .border_1()
        .border_color(palette.sep)
        .children(rows);
    let content = div()
        .flex()
        .flex_col()
        .gap(px(6.))
        .child(
            div()
                .text_size(px(12.))
                .text_color(palette.text2)
                .child(note),
        )
        .child(card);
    Some(titled_section("Switched over", None, content, palette))
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

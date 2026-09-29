use captain_core::migration::{
    RollbackStatus, SubStatus, SwitchOverProgress, SwitchOverStep, downtime_label,
};
use gpui_kit::assets::IconName;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::{Icon, Sizable};
use gpui_kit::*;

use crate::theme::Palette;

/// The steps of a switch-over under its run row, then the downtime once the check
/// passed, and how a roll back went.
pub fn switch_steps(progress: &SwitchOverProgress, palette: &Palette) -> Div {
    let steps = SwitchOverStep::SEQUENCE
        .into_iter()
        .filter(|step| *step != SwitchOverStep::Done)
        .map(|step| {
            let status = progress.status(step);
            let color = match status {
                SubStatus::Pending => palette.text3,
                SubStatus::Running => palette.text,
                SubStatus::Done => palette.green,
                SubStatus::Failed => palette.red,
            };
            div()
                .flex()
                .items_center()
                .gap(px(4.))
                .child(step_icon(status, color, palette))
                .child(div().text_color(color).child(step.label()))
        });
    let mut lines = Vec::new();
    if let Some(downtime) = progress.downtime {
        lines.push((
            format!("Downtime: {}", downtime_label(downtime)),
            palette.text2,
        ));
    }
    match &progress.rollback {
        RollbackStatus::NotRun => {}
        RollbackStatus::Running => lines.push(("Rolling back…".into(), palette.text2)),
        RollbackStatus::Done => lines.push((
            "Rolled back. It is stopped here, and the original runs in the old engine again."
                .into(),
            palette.green,
        )),
        RollbackStatus::Failed(error) => {
            lines.push((format!("Roll back failed: {error}"), palette.red));
        }
    }
    div()
        .flex()
        .flex_col()
        .gap(px(3.))
        .text_size(px(11.))
        .child(div().flex().flex_wrap().gap(px(10.)).children(steps))
        .children(
            lines
                .into_iter()
                .map(|(text, color)| div().text_color(color).child(text)),
        )
}

fn step_icon(status: SubStatus, color: Hsla, palette: &Palette) -> AnyElement {
    let name = match status {
        SubStatus::Running => {
            return Spinner::new()
                .xsmall()
                .color(palette.accent)
                .into_any_element();
        }
        SubStatus::Pending => IconName::Clock,
        SubStatus::Done => IconName::CircleCheck,
        SubStatus::Failed => IconName::CircleX,
    };
    Icon::new(name)
        .size(px(12.))
        .text_color(color)
        .into_any_element()
}

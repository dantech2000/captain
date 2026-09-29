use std::path::PathBuf;

use captain_core::diagnostics::{Check, CheckState};
use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::*;

use super::fix;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, pill, settings_row, text_button};

/// One check: an icon and title, the detail line, the fix, and the state.
pub fn render(check: &Check, engine_dir: Option<PathBuf>, palette: &Palette) -> AnyElement {
    let (icon, color) = match check.state {
        CheckState::Passed => (IconName::CircleCheck, palette.green),
        CheckState::Warning => (IconName::TriangleAlert, palette.orange),
        CheckState::Failed => (IconName::CircleX, palette.red),
        CheckState::NotApplicable => (IconName::CircleMinus, palette.gray),
    };
    let label = div()
        .flex()
        .items_center()
        .gap(px(8.))
        .child(Icon::new(icon).size(px(15.)).text_color(color))
        .child(check.id.title());
    let fix_button = check.fix.clone().map(|action| {
        text_button(
            SharedString::from(format!("diagnostics-fix-{:?}", check.id)),
            action.label(),
            ButtonTone::Accent,
            true,
            palette,
            move |_, window, cx| fix::run(&action, engine_dir.as_ref(), window, cx),
        )
    });
    let controls = div()
        .flex()
        .items_center()
        .gap(px(8.))
        .children(fix_button)
        .child(pill(check.state.label(), color, palette.tint(color)));
    settings_row(label, Some(check.detail.clone().into()), controls, palette).into_any_element()
}

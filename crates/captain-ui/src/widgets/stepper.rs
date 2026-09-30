use gpui_kit::assets::IconName;
use gpui_kit::*;

use super::icon_button;
use crate::theme::Palette;

/// Runs with the step, -1 or +1.
pub type StepHandler = Box<dyn Fn(i32, &mut App)>;

/// A value between minus and plus buttons, for settings such as the CPU count.
/// `label` names the setting in the buttons' help.
pub fn stepper(
    id: &'static str,
    label: &str,
    value: impl Into<SharedString>,
    palette: &Palette,
    on_step: StepHandler,
) -> Div {
    let on_step = std::rc::Rc::new(on_step);
    let down = on_step.clone();
    div()
        .flex()
        .items_center()
        .gap(px(8.))
        .child(icon_button(
            (id, 0usize),
            IconName::Minus,
            format!("Decrease {label} by one step."),
            palette,
            move |_, _, cx| down(-1, cx),
        ))
        .child(
            div()
                .min_w(px(64.))
                .flex()
                .justify_center()
                .text_color(palette.text)
                .child(value.into()),
        )
        .child(icon_button(
            (id, 1usize),
            IconName::Plus,
            format!("Increase {label} by one step."),
            palette,
            move |_, _, cx| on_step(1, cx),
        ))
}

//! Shown when Captain Engine is stopped, failed, or cannot run here.

use captain_core::HostStatus;
use gpui_kit::*;

use super::HostModel;
use super::host_screen::{frame, note, page, title};
use crate::theme::Palette;
use crate::widgets::{ButtonTone, primary_button, text_button};

pub fn render(model: &Entity<HostModel>, host: &HostModel, palette: &Palette) -> Stateful<Div> {
    let column = frame(palette);
    let column = match host.status() {
        HostStatus::NotInstalled(reason) => column
            .child(title("Captain Engine needs Lima"))
            .child(note(reason.clone(), palette))
            .child(code_box("brew install lima", palette))
            .child(note(
                "Captain checks again every few seconds. Until then, you can use \
                 another engine.",
                palette,
            ))
            .child(buttons(model, None, palette)),
        HostStatus::Failed(message) => column
            .child(title("Captain Engine did not start"))
            .child(code_box(message.clone(), palette))
            .child(note(
                "Try again: Captain keeps what the first start downloaded.",
                palette,
            ))
            .child(buttons(model, Some("Try again"), palette)),
        _ => column
            .child(title("Captain Engine is stopped"))
            .child(note("Start it to see your containers.", palette))
            .children(
                host.last_error()
                    .map(|error| code_box(format!("The last start failed: {error}"), palette)),
            )
            .child(buttons(model, Some("Start"), palette)),
    };
    page(column)
}

/// The start button (when `start` has a label) and a way to another engine.
fn buttons(model: &Entity<HostModel>, start: Option<&'static str>, palette: &Palette) -> Div {
    let starter = model.clone();
    let external = model.clone();
    div()
        .pt(px(6.))
        .flex()
        .items_center()
        .gap(px(12.))
        .children(start.map(|label| {
            primary_button("host-start", label, true, palette, move |_, _, cx| {
                starter.update(cx, |model, cx| model.start(cx));
            })
        }))
        .child(text_button(
            "host-use-other",
            "Use an existing engine",
            ButtonTone::Accent,
            true,
            palette,
            move |_, _, cx| external.update(cx, |model, cx| model.use_external(cx)),
        ))
}

fn code_box(text: impl Into<SharedString>, palette: &Palette) -> Div {
    div()
        .w_full()
        .p(px(12.))
        .rounded(px(10.))
        .bg(palette.card)
        .border_1()
        .border_color(palette.sep)
        .font_family(palette.mono())
        .text_size(px(11.))
        .text_color(palette.text2)
        .child(text.into())
}

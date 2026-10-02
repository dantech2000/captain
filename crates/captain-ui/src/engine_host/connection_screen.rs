use captain_core::EngineError;
use gpui_kit::*;

use super::host_model;
use crate::help::HelpExt;
use crate::icons::{CaptainIcon, cap_icon};
use crate::settings;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, text_button};
use crate::workspace::{Connection, Workspace};

/// What every engine page shows instead of its content while Captain has no working
/// connection: "Reconnecting" for a while after a drop, then the failure with Retry.
/// `None` while connected or connecting for the first time. See feature 0013.
pub fn render(workspace: &Workspace, palette: &Palette, cx: &App) -> Option<AnyElement> {
    let retrying = workspace.reconnecting().is_some();
    match workspace.connection() {
        Connection::Failed(error) => Some(failure(error, retrying, palette, cx).into_any_element()),
        Connection::Connecting if retrying => Some(reconnecting(palette)),
        _ => None,
    }
}

/// The calm note while Captain reconnects after a drop, for example after Docker
/// restarted inside the engine.
fn reconnecting(palette: &Palette) -> AnyElement {
    div()
        .size_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(6.))
        .p(px(40.))
        .child(div().text_color(palette.text).child("Reconnecting to the engine\u{2026}"))
        .child(
            div()
                .text_size(px(12.))
                .text_color(palette.text2)
                .child("Docker stopped answering, for example after a restart. Captain connects again when it answers."),
        )
        .into_any_element()
}

/// Shown when Captain cannot reach its engine. When Captain Engine runs but does not
/// answer, the Docker socket forward may have died after the Mac slept
/// (lima-vm/lima#5420), so it offers a restart. `retrying` says that Captain keeps
/// trying to reconnect by itself.
fn failure(error: &EngineError, retrying: bool, palette: &Palette, cx: &App) -> impl IntoElement {
    let restart = host_model(cx).filter(|host| {
        let host = host.read(cx);
        host.uses_captain(cx) && host.status().is_running() && host.can_control()
    });
    let (heading, hint) = if restart.is_some() {
        (
            "Captain Engine is not answering",
            "Its Docker socket can stop working after the Mac sleeps. Restart the engine.",
        )
    } else {
        (
            "Captain's engine isn't running",
            "Start the engine, then click Retry. Settings can switch engines.",
        )
    };
    let retry_note = retrying.then_some("Captain keeps trying to reconnect by itself.");
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .p(px(40.))
        .child(
            div()
                .w(px(520.))
                .flex()
                .flex_col()
                .items_center()
                .gap(px(16.))
                .child(
                    div()
                        .size(px(64.))
                        .rounded(px(18.))
                        .bg(palette.tint(palette.red))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(cap_icon(CaptainIcon::Engine, px(30.), palette.red)),
                )
                .child(
                    div()
                        .text_size(px(22.))
                        .font_weight(FontWeight::BOLD)
                        .child(heading),
                )
                .child(div().text_color(palette.text2).child(hint))
                .children(retry_note.map(|note| {
                    div()
                        .text_size(px(12.))
                        .text_color(palette.text2)
                        .child(note)
                }))
                .child(
                    div()
                        .flex()
                        .gap(px(8.))
                        .child(text_button(
                            "engine-retry",
                            "Retry",
                            ButtonTone::Accent,
                            true,
                            palette,
                            |_, _, cx| settings::retry(cx),
                        )
                        .help("Connect to the engine again now."))
                        .children(restart.map(|host| {
                            text_button(
                                "engine-restart",
                                "Restart Captain Engine",
                                ButtonTone::Accent,
                                true,
                                palette,
                                move |_, _, cx| host.update(cx, |host, cx| host.restart(cx)),
                            )
                            .help("Stop Captain Engine and start it again. Containers with a restart policy come back.")
                        })),
                )
                .child(
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
                        .child(error.to_string()),
                ),
        )
}

use captain_core::EngineError;
use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::*;

use crate::engine_host::HostModel;
use crate::settings;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, text_button};

/// Shown when Captain cannot reach its engine. When Captain Engine runs but does not
/// answer, the Docker socket forward may have died after the Mac slept
/// (lima-vm/lima#5420), so it offers a restart.
pub fn render(
    error: &EngineError,
    host: Option<&Entity<HostModel>>,
    palette: &Palette,
    cx: &App,
) -> impl IntoElement {
    let restart = host
        .filter(|host| {
            let host = host.read(cx);
            host.uses_captain(cx) && host.status().is_running() && host.can_control()
        })
        .cloned();
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
                        .child(
                            Icon::new(IconName::Container)
                                .size(px(30.))
                                .text_color(palette.red),
                        ),
                )
                .child(
                    div()
                        .text_size(px(22.))
                        .font_weight(FontWeight::BOLD)
                        .child(heading),
                )
                .child(div().text_color(palette.text2).child(hint))
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
                        ))
                        .children(restart.map(|host| {
                            text_button(
                                "engine-restart",
                                "Restart Captain Engine",
                                ButtonTone::Accent,
                                true,
                                palette,
                                move |_, _, cx| host.update(cx, |host, cx| host.restart(cx)),
                            )
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

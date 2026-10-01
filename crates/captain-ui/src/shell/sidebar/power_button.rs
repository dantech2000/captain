use captain_core::HostStatus;
use gpui_kit::assets::IconName;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::{Icon, Sizable};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::engine_host::HostSummary;
use crate::help::HelpExt;
use crate::theme::Palette;

/// Captain Engine's power button at the end of the sidebar status line: Stop while
/// it runs, Start or Set up while it does not, and a spinner while it starts or
/// stops.
pub fn render(host: &HostSummary, palette: &Palette) -> Stateful<Div> {
    let base = div()
        .flex_shrink_0()
        .size(px(22.))
        .rounded(px(6.))
        .flex()
        .items_center()
        .justify_center();
    if host.status.is_busy() {
        return base
            .id("sidebar-engine-busy")
            .child(Spinner::new().xsmall().color(palette.text2))
            .help(format!(
                "Captain Engine is {}.",
                host.status.label().to_lowercase()
            ));
    }
    let model = host.model.clone();
    let stop = host.status.can_stop();
    let enabled = host.can_control && (stop || host.status.can_start());
    let (id, help) = if stop {
        (
            "sidebar-engine-stop",
            "Stop Captain Engine. Running containers stop with it.",
        )
    } else if host.status == HostStatus::NotCreated {
        (
            "sidebar-engine-start",
            "Set up Captain Engine: download and create its virtual machine.",
        )
    } else {
        ("sidebar-engine-start", "Start Captain Engine.")
    };
    let hover = palette.hover;
    base.id(id)
        .text_color(if enabled {
            palette.text2
        } else {
            palette.text3
        })
        .when(enabled, |button| {
            button
                .cursor_pointer()
                .hover(move |style| style.bg(hover))
                .on_click(move |_, _, cx| {
                    if stop {
                        model.update(cx, |model, cx| model.stop(cx)).detach();
                    } else {
                        model.update(cx, |model, cx| model.start(cx));
                    }
                })
        })
        .child(Icon::new(IconName::Power).size(px(14.)))
        .help(help)
}

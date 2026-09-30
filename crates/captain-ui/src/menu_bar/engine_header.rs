//! The popover's first row: the engine, its use, and a switch to start or stop it.

use gpui_kit::*;

use super::controls::switch;
use super::popover_data::EngineLine;
use crate::engine_host::host_model;
use crate::help::HelpExt;
use crate::icons::{CaptainIcon, cap_icon};
use crate::theme::Palette;

pub fn render(engine: &EngineLine, palette: &Palette) -> Div {
    let color = match &engine.host {
        Some(status) => palette.host_status(status),
        None if engine.on => palette.green,
        None => palette.gray,
    };
    let help = match (&engine.host, engine.on) {
        (None, _) => "Captain does not start or stop this engine. Switch engines in Settings.",
        (Some(_), _) if !engine.can_switch => {
            "Wait until Captain Engine finishes starting or stopping."
        }
        (Some(_), true) => "Stop Captain Engine and every container in it.",
        (Some(_), false) => "Start Captain Engine.",
    };
    div()
        .flex()
        .items_center()
        .gap(px(12.))
        .px(px(16.))
        .py(px(14.))
        .child(
            div()
                .size(px(34.))
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(9.))
                .bg(palette.tint(palette.accent))
                .child(cap_icon(CaptainIcon::Engine, px(22.), palette.accent_fg)),
        )
        .child(
            div()
                .id("popover-engine")
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(div().font_weight(FontWeight::BOLD).child(engine.name))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .text_size(px(11.5))
                        .text_color(palette.text2)
                        .child(div().size(px(6.)).rounded_full().bg(color))
                        .child(div().truncate().child(engine.line.clone())),
                )
                .help("The engine that runs your containers, its CPUs, and the memory in use."),
        )
        .child(switch(
            "popover-engine-switch",
            engine.on,
            engine.can_switch,
            help,
            |on, _, cx| {
                let Some(host) = host_model(cx) else {
                    return;
                };
                if on {
                    host.update(cx, |host, cx| host.start(cx));
                } else {
                    host.update(cx, |host, cx| host.stop(cx)).detach();
                }
            },
        ))
}

//! The Terminal section: one line that says whether the terminal's docker,
//! Compose, and Buildx use Captain Engine, and the button to the setup sheet.
//! See features 0035 and 0037.

use captain_core::cli_tools;
use captain_core::settings::Settings;
use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::page_section::{fill_note, row, section};
use super::{SettingsView, terminal_sheet};
use crate::help::HelpExt;
use crate::theme::Palette;
use crate::widgets::primary_button;

pub fn render(
    view: &SettingsView,
    settings: &Settings,
    palette: &Palette,
    cx: &mut Context<SettingsView>,
) -> Div {
    let head = row("Terminal", palette);
    if !cli_tools::SUPPORTED {
        return section(palette).child(head.child(fill_note(cli_tools::UNSUPPORTED, palette)));
    }
    let this = cx.entity();
    let open = move |_: &ClickEvent, window: &mut Window, cx: &mut App| {
        terminal_sheet::open(this.clone(), window, cx)
    };
    let Some(facts) = view.terminal_facts(settings, cx) else {
        return section(palette)
            .child(head.child(fill_note("Checking your terminal\u{2026}", palette)));
    };
    let done = facts.steps.all();
    let line = div()
        .flex_1()
        .min_w_0()
        .flex()
        .items_center()
        .gap(px(8.))
        .when(done, |line| {
            line.child(facts.summary.clone()).child(
                Icon::new(IconName::Check)
                    .size(px(14.))
                    .text_color(palette.green),
            )
        })
        .when(!done, |line| {
            line.child(
                div()
                    .size(px(8.))
                    .flex_shrink_0()
                    .rounded_full()
                    .bg(palette.orange),
            )
            .child(div().min_w_0().child(facts.summary.clone()))
        });
    let action = if done {
        div()
            .id("settings-terminal-details")
            .text_size(px(12.))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(palette.link)
            .cursor_pointer()
            .on_click(open)
            .child("Details")
            .help("Show the terminal setup: the links, PATH, and the docker context.")
    } else {
        primary_button(
            "settings-terminal-setup",
            "Set up\u{2026}",
            "Make docker, Compose, and Buildx in your terminal use Captain Engine, step by step.",
            true,
            palette,
            open,
        )
    };
    section(palette)
        .when(!done, |card| card.border_color(palette.orange.alpha(0.4)))
        .child(
            head.id("settings-terminal")
                .child(line)
                .child(action)
                .help("Whether a new terminal uses Captain's docker, Compose, and Buildx with Captain Engine."),
        )
}

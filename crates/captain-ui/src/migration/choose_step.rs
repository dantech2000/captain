use captain_core::migration::{SourceOption, is_captain_engine};
use gpui_kit::component::Sizable;
use gpui_kit::component::WindowExt;
use gpui_kit::component::input::Input;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::*;

use super::assistant::{MigrationAssistant, Stage};
use super::view::footer_row;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, section_note, settings_card, settings_row, text_button};

/// Step 1: the engines to copy from, and a field for any other endpoint.
pub fn body(
    view: &MigrationAssistant,
    palette: &Palette,
    cx: &mut Context<MigrationAssistant>,
) -> Div {
    let Some(target) = view.target.clone() else {
        return section_note(
            "Connect to an engine first. It receives the copies.",
            palette,
        );
    };
    if view.backend.is_none() {
        return section_note(
            "The Migration Assistant is not available in this build.",
            palette,
        );
    }
    if view.stage == Stage::Loading {
        return div()
            .pt(px(120.))
            .flex()
            .flex_col()
            .items_center()
            .gap(px(10.))
            .text_color(palette.text2)
            .child(Spinner::new().color(palette.text2))
            .child("Reading what the engine holds…");
    }
    let mut rows: Vec<AnyElement> = view
        .sources
        .iter()
        .enumerate()
        .map(|(ix, source)| source_row(ix, source, palette, cx))
        .collect();
    if rows.is_empty() {
        let note = "Start the old engine, then open the assistant again.";
        rows.push(
            settings_row("No other engines found", Some(note.into()), div(), palette)
                .into_any_element(),
        );
    }
    rows.push(custom_row(view, palette, cx));
    div()
        .flex()
        .flex_col()
        .gap(px(14.))
        .child(intro(&target, palette))
        .child(settings_card("Copy from", rows, palette))
}

fn intro(target: &str, palette: &Palette) -> Div {
    let mut intro = div()
        .flex()
        .flex_col()
        .gap(px(6.))
        .text_size(px(12.))
        .text_color(palette.text2)
        .child(
            "Captain copies volumes, images, networks, Compose projects, and containers \
             into the engine it is connected to. It never changes or deletes anything \
             in the old engine.",
        )
        .child(
            div().flex().gap(px(4.)).child("Copy into:").child(
                div()
                    .font_family(palette.mono())
                    .text_color(palette.text)
                    .child(target.to_string()),
            ),
        );
    if !is_captain_engine(target) {
        intro = intro.child(div().text_color(palette.warn_text).child(
            "This engine is not Captain Engine. The copy works, but it goes into the engine above.",
        ));
    }
    intro
}

fn source_row(
    ix: usize,
    source: &SourceOption,
    palette: &Palette,
    cx: &mut Context<MigrationAssistant>,
) -> AnyElement {
    let host = source.host.clone();
    let button = text_button(
        ("migration-source", ix),
        "Choose",
        ButtonTone::Accent,
        true,
        palette,
        cx.listener(move |view, _, _, cx| view.choose(host.clone(), cx)),
    );
    let label = div()
        .font_family(palette.mono())
        .text_size(px(12.))
        .child(source.host.clone());
    settings_row(label, Some(source.label.clone().into()), button, palette).into_any_element()
}

fn custom_row(
    view: &MigrationAssistant,
    palette: &Palette,
    cx: &mut Context<MigrationAssistant>,
) -> AnyElement {
    let control = div()
        .flex()
        .items_center()
        .gap(px(8.))
        .child(div().w(px(260.)).child(Input::new(&view.custom).small()))
        .child(text_button(
            "migration-custom",
            "Choose",
            ButtonTone::Accent,
            true,
            palette,
            cx.listener(|view, _, _, cx| view.choose_custom(cx)),
        ));
    let note = "A unix://, npipe://, tcp://, or http:// URL.";
    settings_row("Other engine", Some(note.into()), control, palette).into_any_element()
}

pub fn footer(view: &MigrationAssistant, palette: &Palette) -> Div {
    let note = (view.stage == Stage::Loading).then(|| "Connecting…".to_string());
    let close = text_button(
        "migration-close",
        "Close",
        ButtonTone::Accent,
        true,
        palette,
        |_, window, cx| window.close_dialog(cx),
    );
    footer_row(note, vec![close.into_any_element()], palette)
}

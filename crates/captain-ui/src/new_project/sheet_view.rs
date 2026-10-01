use std::path::{Path, PathBuf};

use gpui_kit::component::Sizable;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::new_sheet::{NewSheet, SheetStatus};
use super::options::NewOption;
use crate::help::{HelpExt, cmd_key};
use crate::icons::glyph;
use crate::palette::key_hint;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, pill, text_button};

/// The sheet's title, the four option cards, the status of an open, and where new
/// projects go.
pub fn render(sheet: &NewSheet, cx: &mut Context<NewSheet>) -> Div {
    let palette = Palette::of(cx);
    let this = cx.entity().downgrade();
    let cards = NewOption::ALL
        .into_iter()
        .enumerate()
        .map(|(index, option)| {
            let this = this.clone();
            card(option, index, index == sheet.highlight, &palette).on_click(
                move |_, window, cx| {
                    this.update(cx, |sheet, cx| sheet.choose(option, window, cx))
                        .ok();
                },
            )
        });
    let home = std::env::home_dir();
    let dir =
        crate::settings::current(cx).projects_dir_in(home.as_deref().unwrap_or(Path::new("")));
    div()
        .flex()
        .flex_col()
        .gap(px(14.))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(4.))
                .pr(px(32.))
                .child(
                    div()
                        .text_size(px(18.))
                        .font_weight(FontWeight::EXTRA_BOLD)
                        .child("New project"),
                )
                .child(
                    div().text_color(palette.text2).child(
                        "Each way ends in the editor, with a preview before anything starts.",
                    ),
                ),
        )
        .child(div().flex().flex_col().gap(px(8.)).children(cards))
        .children(status(&sheet.status, &palette))
        .child(footer(dir, home.as_deref(), &palette))
}

fn card(option: NewOption, index: usize, highlighted: bool, palette: &Palette) -> Stateful<Div> {
    let ready = option.ready();
    let hover = palette.hover;
    div()
        .id(SharedString::from(format!("new-option-{index}")))
        .flex()
        .items_center()
        .gap(px(12.))
        .p(px(12.))
        .rounded(px(12.))
        .border_1()
        .when(highlighted, |this| {
            this.bg(palette.nav_selected)
                .border_color(palette.accent.alpha(0.45))
        })
        .when(!highlighted, |this| {
            this.border_color(palette.sep)
                .bg(palette.card)
                .hover(move |style| style.bg(hover))
        })
        .when(ready, |this| this.cursor_pointer())
        .child(
            div()
                .size(px(38.))
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(10.))
                .bg(palette.tint(palette.accent))
                .child(glyph(option.icon(), px(22.), palette.accent_fg)),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(2.))
                .when(!ready, |this| this.opacity(0.55))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(option.title())
                        .when(!ready, |this| {
                            this.child(pill("Coming next", palette.text2, palette.field))
                        }),
                )
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(palette.text2)
                        .child(option.sentence()),
                ),
        )
        .child(key_hint(cmd_key(&(index + 1).to_string()), palette))
        .help(option.help())
}

fn status(status: &SheetStatus, palette: &Palette) -> Option<Div> {
    let line = div().flex().items_center().gap(px(8.)).text_size(px(12.));
    match status {
        SheetStatus::Idle => None,
        SheetStatus::Checking(file) => Some(
            line.text_color(palette.text2)
                .child(Spinner::new().xsmall().color(palette.text2))
                .child(format!(
                    "Checking {file} with docker compose config\u{2026}"
                )),
        ),
        SheetStatus::Failed(error) => Some(line.text_color(palette.red).child(error.clone())),
    }
}

/// Where new projects go, with Show folder, which creates the folder first.
fn footer(dir: PathBuf, home: Option<&Path>, palette: &Palette) -> Div {
    let shown = match home.and_then(|home| dir.strip_prefix(home).ok()) {
        Some(rest) => format!("~/{}", rest.display()),
        None => dir.display().to_string(),
    };
    let help = format!(
        "Open {shown}, and create it if it is missing. Change it with projects_dir in the settings file."
    );
    div()
        .flex()
        .items_center()
        .justify_between()
        .gap(px(12.))
        .pt(px(12.))
        .border_t_1()
        .border_color(palette.sep)
        .text_size(px(12.))
        .text_color(palette.text2)
        .child(format!("New projects go in {shown}."))
        .child(
            text_button(
                "new-show-folder",
                "Show folder",
                ButtonTone::Accent,
                true,
                palette,
                move |_, _, cx| match std::fs::create_dir_all(&dir) {
                    Ok(()) => cx.open_with_system(&dir),
                    Err(error) => tracing::warn!(%error, "cannot create {}", dir.display()),
                },
            )
            .help(help),
        )
}

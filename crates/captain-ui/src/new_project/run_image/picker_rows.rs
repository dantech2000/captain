use captain_core::new_project::{HubRepo, count_label};
use gpui_kit::component::IndexPath;
use gpui_kit::component::Sizable;
use gpui_kit::component::tag::Tag;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::help::HelpExt;
use crate::theme::Palette;

/// A row of the image list: a bold title over a gray line.
pub fn two_lines(
    ix: IndexPath,
    title: SharedString,
    line: SharedString,
    palette: &Palette,
) -> Stateful<Div> {
    div()
        .id(("image-row", ix.section * 1000 + ix.row))
        .h(px(36.))
        .flex_1()
        .min_w_0()
        .flex()
        .flex_col()
        .justify_center()
        .child(
            div()
                .font_weight(FontWeight::SEMIBOLD)
                .truncate()
                .child(title),
        )
        .child(
            div()
                .text_size(px(12.))
                .text_color(palette.text2)
                .truncate()
                .child(line),
        )
}

/// A Docker Hub row: the name, an Official tag, stars and pulls, and the
/// description.
pub fn hub_row(ix: IndexPath, repo: &HubRepo, palette: &Palette) -> Stateful<Div> {
    let counts = format!(
        "\u{2605} {} \u{b7} {} pulls",
        count_label(repo.stars),
        count_label(repo.pulls)
    );
    let line = match repo.description.trim() {
        "" => counts,
        text => format!("{counts} \u{b7} {text}"),
    };
    let official = Tag::custom(
        palette.tint(palette.accent),
        palette.readable(palette.accent),
        palette.accent.alpha(0.3),
    )
    .xsmall()
    .child("Official");
    two_lines(ix, repo.name.clone().into(), line.into(), palette)
        .when(repo.official, |row| {
            row.child(div().absolute().top(px(2.)).right(px(4.)).child(official))
        })
        .relative()
        .help(format!(
            "Run {} from Docker Hub{}. Captain pulls it when it runs.",
            repo.name,
            if repo.official {
                ", an official image"
            } else {
                ""
            }
        ))
}

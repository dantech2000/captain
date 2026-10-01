use captain_core::new_project::{HubRepo, count_label};
use gpui_kit::component::IndexPath;
use gpui_kit::component::Sizable;
use gpui_kit::component::tag::Tag;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::help::HelpExt;
use crate::new_project::pick_row::lines;
use crate::theme::Palette;

/// A row of the image list: a bold title over a gray line.
pub fn two_lines(
    ix: IndexPath,
    title: impl IntoElement,
    line: SharedString,
    palette: &Palette,
) -> Stateful<Div> {
    div()
        .id(("image-row", ix.section * 1000 + ix.row))
        .flex_1()
        .min_w_0()
        .flex()
        .child(lines(title, line, palette))
}

/// A Docker Hub row: the name with an Official tag, then stars, pulls, and the
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
    let title = div()
        .flex()
        .items_center()
        .gap(px(6.))
        .child(div().min_w_0().truncate().child(repo.name.clone()))
        .when(repo.official, |title| {
            title.child(div().flex_shrink_0().child(official))
        });
    two_lines(ix, title, line.into(), palette).help(format!(
        "Run {} from Docker Hub{}. Captain pulls it when it runs.",
        repo.name,
        if repo.official {
            ", an official image"
        } else {
            ""
        }
    ))
}

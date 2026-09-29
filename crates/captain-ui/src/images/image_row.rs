use captain_core::format::{age_label, bytes_label};
use captain_core::model::Image;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::ImagesState;
use crate::theme::Palette;
use crate::widgets::pill;

pub const ID_WIDTH: f32 = 104.;
pub const SIZE_WIDTH: f32 = 72.;
pub const CREATED_WIDTH: f32 = 108.;

/// One image: repository, tag, usage, short ID, size, and age. A click selects it.
pub fn render(
    image: &Image,
    handle: &Entity<ImagesState>,
    state: &ImagesState,
    now: i64,
    palette: &Palette,
) -> impl IntoElement {
    let selected = state.selected().is_some_and(|s| s.id == image.id);
    let removing = state.is_removing(&image.id);
    let id = image.id.clone();
    let select = handle.clone();

    div()
        .id(SharedString::from(format!("image-{}", image.id)))
        .h(px(50.))
        .px(px(12.))
        .flex()
        .items_center()
        .gap(px(12.))
        .rounded(px(9.))
        .cursor_pointer()
        .when(removing, |row| row.opacity(0.5))
        .when(selected, |row| {
            row.bg(palette.accent.alpha(if palette.dark { 0.15 } else { 0.08 }))
                .border_1()
                .border_color(palette.accent.alpha(0.35))
        })
        .when(!selected, |row| row.hover(|style| style.bg(palette.group)))
        .on_click(move |_, _, cx| {
            select.update(cx, |state, cx| state.select(id.clone(), cx));
        })
        .child(name_cell(image, palette))
        .child(
            div()
                .w(px(ID_WIDTH))
                .flex_shrink_0()
                .font_family(palette.mono())
                .text_size(px(11.))
                .text_color(palette.text2)
                .child(image.short_id().to_string()),
        )
        .child(
            div()
                .w(px(SIZE_WIDTH))
                .flex_shrink_0()
                .text_right()
                .text_size(px(12.))
                .text_color(palette.text2)
                .child(bytes_label(image.size)),
        )
        .child(
            div()
                .w(px(CREATED_WIDTH))
                .flex_shrink_0()
                .text_right()
                .text_size(px(12.))
                .text_color(palette.text3)
                .truncate()
                .child(if removing {
                    "Removing...".to_string()
                } else {
                    age_label(image.created, now)
                }),
        )
}

fn name_cell(image: &Image, palette: &Palette) -> Div {
    let (repository, tag) = image.repository_and_tag();
    let extra = image.extra_tag_count();
    let usage = if image.in_use() {
        let label = match image.containers {
            1 => "in use".to_string(),
            n => format!("in use · {n}"),
        };
        Some(pill(label, palette.green, palette.tint(palette.green)))
    } else if image.dangling {
        Some(pill(
            "dangling",
            palette.orange,
            palette.tint(palette.orange),
        ))
    } else {
        None
    };

    div()
        .flex_1()
        .min_w_0()
        .flex()
        .flex_col()
        .gap(px(3.))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.))
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(if image.dangling {
                            palette.text2
                        } else {
                            palette.text
                        })
                        .truncate()
                        .child(repository.to_string()),
                )
                .children(usage),
        )
        .child(
            div()
                .flex()
                .gap(px(6.))
                .font_family(palette.mono())
                .text_size(px(11.))
                .text_color(palette.text3)
                .truncate()
                .child(if tag.is_empty() {
                    "no tag".to_string()
                } else {
                    tag.to_string()
                })
                .when(extra > 0, |this| this.child(format!("+{extra} more"))),
        )
}

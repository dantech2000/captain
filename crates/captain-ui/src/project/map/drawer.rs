use std::collections::BTreeSet;

use captain_core::model::count_label;
use captain_core::project_map::StagedChange;
use gpui_kit::assets::IconName;
use gpui_kit::*;

use crate::help::HelpExt;
use crate::icons::{CaptainIcon, cap_icon};
use crate::project::ProjectView;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, icon_button, primary_button, text_button};

/// The staged changes of the shown containers: one row per field, with a remove
/// button, then Discard and Apply. `ids` are the shown containers; `busy` is true
/// while an update runs.
pub fn render(
    changes: Vec<StagedChange>,
    ids: Vec<String>,
    busy: bool,
    view: &WeakEntity<ProjectView>,
    palette: &Palette,
) -> Div {
    let containers: BTreeSet<&str> = changes.iter().map(|c| c.container_id.as_str()).collect();
    let discard = {
        let (view, ids) = (view.clone(), ids.clone());
        text_button(
            "map-discard",
            "Discard",
            ButtonTone::Danger,
            !busy,
            palette,
            move |_, _, cx| {
                view.update(cx, |view, cx| view.discard(&ids, cx)).ok();
            },
        )
        .h(px(30.))
        .help("Drop the staged changes listed here. The containers stay as they are.")
    };
    let targets = count_label(containers.len(), "container");
    let apply = {
        let view = view.clone();
        primary_button(
            "map-apply",
            format!("Apply · update {targets}"),
            format!(
                "Send the staged changes to the engine, like docker update, for {targets}. They keep running."
            ),
            !busy,
            palette,
            move |_, _, cx| {
                view.update(cx, |view, cx| view.apply(&ids, cx)).ok();
            },
        )
        .h(px(30.))
        .text_size(px(12.))
    };
    div()
        .flex_shrink_0()
        .max_h(px(220.))
        .flex()
        .flex_col()
        .gap(px(10.))
        .px(px(24.))
        .py(px(16.))
        .bg(palette.panel)
        .border_t_1()
        .border_color(palette.orange.alpha(0.4))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(10.))
                .child(
                    div()
                        .text_size(px(14.))
                        .font_weight(FontWeight::BOLD)
                        .child("Staged changes"),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .text_size(px(12.))
                        .text_color(palette.text2)
                        .child("Nothing changes until you apply. Compose sets its own values again when it recreates a container."),
                )
                .child(discard)
                .child(apply),
        )
        .child(
            div()
                .id("map-staged-rows")
                .flex()
                .flex_col()
                .gap(px(6.))
                .overflow_y_scroll()
                .children(changes.iter().map(|change| row(change, view, palette))),
        )
}

fn row(change: &StagedChange, view: &WeakEntity<ProjectView>, palette: &Palette) -> Div {
    let (id, field) = (change.container_id.clone(), change.to);
    let view = view.clone();
    let remove = icon_button(
        SharedString::from(format!(
            "map-unstage-{}-{}",
            change.container_id,
            change.to.field()
        )),
        IconName::X,
        format!(
            "Remove the change to the {} of {} from the staged changes.",
            change.to.field().to_lowercase(),
            change.container
        ),
        palette,
        move |_, _, cx| {
            view.update(cx, |view, cx| view.unstage(&id, field, cx))
                .ok();
        },
    );
    div()
        .flex_shrink_0()
        .h(px(40.))
        .px(px(12.))
        .flex()
        .items_center()
        .gap(px(12.))
        .rounded(px(9.))
        .bg(palette.card)
        .border_1()
        .border_color(palette.sep)
        .text_size(px(12.))
        .child(
            div()
                .w(px(160.))
                .flex_shrink_0()
                .flex()
                .items_center()
                .gap(px(8.))
                .font_weight(FontWeight::SEMIBOLD)
                .child(cap_icon(CaptainIcon::Container, px(14.), palette.text2))
                .child(div().truncate().child(change.container.clone())),
        )
        .child(
            div()
                .w(px(140.))
                .flex_shrink_0()
                .text_color(palette.text2)
                .child(change.to.field()),
        )
        .child(
            div()
                .flex_1()
                .flex()
                .items_center()
                .gap(px(10.))
                .font_family(palette.mono())
                .child(
                    div()
                        .text_color(palette.red)
                        .line_through()
                        .child(change.from.value_label()),
                )
                .child(div().text_color(palette.text3).child("→"))
                .child(
                    div()
                        .text_color(palette.green)
                        .child(change.to.value_label()),
                ),
        )
        .child(
            div()
                .text_size(px(11.))
                .text_color(palette.text3)
                .child("Updates in place"),
        )
        .child(remove)
}

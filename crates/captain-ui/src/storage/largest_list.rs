use captain_core::format::bytes_label;
use captain_core::storage::{ItemKind, ItemUse, LargeItem};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::help::HelpExt;
use crate::icons::{CaptainIcon, cap_icon};
use crate::theme::Palette;

/// The most names a "Used by" line lists before "and N more".
const NAMES: usize = 2;

/// "Largest first": each item with its icon, who uses it, and its size. Items the
/// cleanup can remove are tinted with the warning color.
pub fn render(items: &[LargeItem], palette: &Palette) -> Div {
    div()
        .flex_1()
        .min_w_0()
        .flex()
        .flex_col()
        .gap(px(10.))
        .child(
            div()
                .flex()
                .items_baseline()
                .gap(px(10.))
                .child(
                    div()
                        .text_size(px(15.))
                        .font_weight(FontWeight::BOLD)
                        .child("Largest first"),
                )
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(palette.text3)
                        .child("What each item is and who uses it"),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .rounded(px(12.))
                .border_1()
                .border_color(palette.sep)
                .bg(palette.card)
                .overflow_hidden()
                .children(
                    items
                        .iter()
                        .enumerate()
                        .map(|(ix, item)| row(ix, item, palette)),
                ),
        )
}

fn row(ix: usize, item: &LargeItem, palette: &Palette) -> Stateful<Div> {
    let (icon_color, use_color) = match (&item.used, item.reclaimable) {
        (ItemUse::UnusedVolume, _) => (palette.orange, palette.red),
        (_, true) => (palette.orange, palette.warn_text),
        (_, false) => (palette.text3, palette.text2),
    };
    let used = use_label(&item.used);
    let size = bytes_label(item.size);
    let help = format!("{} takes {size}. {used}.", item.name);
    div()
        .id(("storage-item", ix))
        .h(px(44.))
        .px(px(14.))
        .flex()
        .items_center()
        .gap(px(12.))
        .when(ix > 0, |row| row.border_t_1().border_color(palette.sep))
        .when(item.reclaimable, |row| row.bg(palette.orange.alpha(0.05)))
        .child(cap_icon(icon(item.kind), px(16.), icon_color))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .truncate()
                .font_family(palette.mono())
                .text_size(px(12.))
                .font_weight(FontWeight::SEMIBOLD)
                .child(item.name.clone()),
        )
        .child(
            div()
                .w(px(240.))
                .flex_shrink_0()
                .truncate()
                .text_size(px(11.5))
                .text_color(use_color)
                .child(used),
        )
        .child(
            div()
                .w(px(80.))
                .flex_shrink_0()
                .flex()
                .justify_end()
                .font_weight(FontWeight::SEMIBOLD)
                .child(size),
        )
        .help(help)
}

fn icon(kind: ItemKind) -> CaptainIcon {
    match kind {
        ItemKind::Image => CaptainIcon::Image,
        ItemKind::Volume => CaptainIcon::Volume,
        ItemKind::BuildCache => CaptainIcon::Reclaim,
        ItemKind::Snapshot => CaptainIcon::Snapshot,
        ItemKind::Container => CaptainIcon::Container,
    }
}

/// The middle column, for example "Used by api, worker" or "Not used since Sep 12".
fn use_label(used: &ItemUse) -> String {
    match used {
        ItemUse::UsedBy(names) if names.is_empty() => "In use".into(),
        ItemUse::UsedBy(names) if names.len() <= NAMES => {
            format!("Used by {}", names.join(", "))
        }
        ItemUse::UsedBy(names) => format!(
            "Used by {} and {} more",
            names[..NAMES].join(", "),
            names.len() - NAMES
        ),
        ItemUse::Dangling => "No tag, not used".into(),
        ItemUse::Unused => "No container uses it".into(),
        ItemUse::NotUsedSince(time) => match date_label(*time) {
            Some(date) => format!("Not used since {date}"),
            None => "Not used".into(),
        },
        ItemUse::UnusedVolume => "Not used · may hold data".into(),
        ItemUse::Running(status) | ItemUse::Stopped(status) => status.clone(),
        ItemUse::RecentBuildCache => "Used by a build in the last 14 days".into(),
        ItemUse::Snapshot => "Snapshot".into(),
    }
}

/// A Unix time as a short local date, for example "Sep 12".
fn date_label(time: i64) -> Option<String> {
    let time = chrono::DateTime::from_timestamp(time, 0).filter(|_| time > 0)?;
    Some(
        time.with_timezone(&chrono::Local)
            .format("%b %-d")
            .to_string(),
    )
}

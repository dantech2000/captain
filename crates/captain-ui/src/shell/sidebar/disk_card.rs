use captain_core::format::bytes_label;
use captain_core::storage::DiskBreakdown;
use gpui_kit::*;

use crate::help::HelpExt;
use crate::storage::{category_color, storage_model};
use crate::theme::Palette;
use crate::workspace::{Page, Workspace};

/// What the Disk card shows: the storage model's disk bar, and the bytes that the
/// cleanup groups checked by default free.
pub struct DiskSummary {
    pub breakdown: DiskBreakdown,
    pub freeable: u64,
}

impl DiskSummary {
    /// The Disk card's data, once the storage model has read the disk use.
    pub fn read(cx: &App) -> Option<Self> {
        let model = storage_model(cx)?;
        let model = model.read(cx);
        Some(Self {
            breakdown: model.breakdown(cx)?,
            freeable: model.default_bytes(),
        })
    }
}

/// "Disk 18.2 of 64 GB", a bar with a part per category, and what a cleanup can
/// free. A click opens Storage. See feature 0031.
pub fn render(handle: &Entity<Workspace>, disk: &DiskSummary, palette: &Palette) -> Stateful<Div> {
    let handle = handle.clone();
    let hover = palette.hover;
    let breakdown = &disk.breakdown;
    let (line, color) = if disk.freeable > 0 {
        (
            format!("{} can be freed · Review", bytes_label(disk.freeable)),
            palette.warn_text,
        )
    } else {
        ("Nothing to free · Review".to_string(), palette.text3)
    };
    let help = if disk.freeable > 0 {
        format!(
            "Show what uses the engine disk. {} can be freed.",
            bytes_label(disk.freeable)
        )
    } else {
        "Show what uses the engine disk.".to_string()
    };
    div()
        .id("sidebar-disk")
        .flex_shrink_0()
        .mt(px(8.))
        .flex()
        .flex_col()
        .gap(px(8.))
        .p(px(12.))
        .rounded(px(12.))
        .bg(palette.card)
        .border_1()
        .border_color(palette.sep)
        .cursor_pointer()
        .hover(move |style| style.bg(hover))
        .on_click(move |_, _, cx| {
            handle.update(cx, |workspace, cx| workspace.set_page(Page::Storage, cx))
        })
        .child(
            div()
                .flex()
                .justify_between()
                .text_size(px(12.))
                .child(div().font_weight(FontWeight::SEMIBOLD).child("Disk"))
                .child(
                    div()
                        .text_color(palette.text2)
                        .child(used_of(breakdown.used, breakdown.capacity)),
                ),
        )
        .child(
            div()
                .h(px(6.))
                .flex()
                .rounded(px(3.))
                .overflow_hidden()
                .bg(palette.track)
                .children(
                    breakdown
                        .categories
                        .iter()
                        .filter(|share| share.bytes > 0)
                        .map(|share| {
                            div()
                                .h_full()
                                .w(relative(share.fraction))
                                .bg(category_color(share.category, palette))
                        }),
                ),
        )
        .child(div().text_size(px(11.)).text_color(color).child(line))
        .help(help)
}

/// "18.2 of 64 GB": the unit shows once when both amounts share it.
fn used_of(used: u64, capacity: Option<u64>) -> String {
    let used = bytes_label(used);
    let Some(capacity) = capacity.map(bytes_label) else {
        return used;
    };
    match (used.split_once(' '), capacity.split_once(' ')) {
        (Some((amount, unit)), Some((_, whole))) if unit == whole => {
            format!("{amount} of {capacity}")
        }
        _ => format!("{used} of {capacity}"),
    }
}

#[cfg(test)]
mod tests;

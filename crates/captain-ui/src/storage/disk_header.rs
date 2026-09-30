use captain_core::format::bytes_label;
use captain_core::storage::{Category, DiskBreakdown};
use gpui_kit::*;

use crate::help::HelpExt;
use crate::theme::Palette;
use crate::widgets::drag_region;
use crate::workspace::{Page, Workspace};

/// The top of the page: the bytes in use out of the disk size, this computer's free
/// space, a bar with a part per category, and its legend. `captain` adds the link to
/// the disk size setting.
pub fn render(
    breakdown: &DiskBreakdown,
    captain: bool,
    host_free: Option<u64>,
    workspace: &Entity<Workspace>,
    palette: &Palette,
) -> Stateful<Div> {
    let title = if captain {
        "Captain Engine disk"
    } else {
        "Engine disk"
    };
    let of = breakdown
        .capacity
        .map(|capacity| format!("of {}", bytes_label(capacity)));
    let workspace = workspace.clone();
    let right = div()
        .flex()
        .flex_col()
        .items_end()
        .gap(px(4.))
        .text_size(px(12.))
        .children(host_free.map(|free| {
            div().text_color(palette.text2).child(format!(
                "{} has {} free",
                this_computer(),
                bytes_label(free)
            ))
        }))
        .children(captain.then(|| {
            div()
                .id("storage-disk-settings")
                .text_color(palette.link)
                .font_weight(FontWeight::SEMIBOLD)
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .on_click(move |_, _, cx| {
                    workspace.update(cx, |w, cx| w.set_page(Page::Settings, cx));
                })
                .child("Change the disk size in Settings")
                .help("Open Settings, where Captain Engine's CPUs, memory, and disk size are.")
        }));

    drag_region("storage-header")
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap(px(16.))
        .px(px(24.))
        .pt(px(18.))
        .pb(px(20.))
        .bg(palette.side)
        .border_b_1()
        .border_color(palette.sep)
        .child(
            div()
                .flex()
                .items_end()
                .gap(px(24.))
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .gap(px(4.))
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(palette.text2)
                                .child(title),
                        )
                        .child(
                            div()
                                .flex()
                                .items_baseline()
                                .gap(px(8.))
                                .child(
                                    div()
                                        .text_size(px(36.))
                                        .font_weight(FontWeight::BOLD)
                                        .child(bytes_label(breakdown.used)),
                                )
                                .children(of.map(|of| {
                                    div().text_size(px(17.)).text_color(palette.text3).child(of)
                                })),
                        ),
                )
                .child(right),
        )
        .child(
            div()
                .h(px(14.))
                .flex()
                .gap(px(2.))
                .rounded(px(7.))
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
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap(px(22.))
                .text_size(px(12.))
                .children(breakdown.categories.iter().map(|share| {
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(
                            div()
                                .size(px(9.))
                                .rounded(px(3.))
                                .bg(category_color(share.category, palette)),
                        )
                        .child(
                            div()
                                .text_color(palette.text2)
                                .child(share.category.label()),
                        )
                        .child(
                            div()
                                .text_color(palette.text3)
                                .child(bytes_label(share.bytes)),
                        )
                })),
        )
}

/// Neutral colors: the bar shows amounts, not states. The sidebar's Disk card uses
/// them too.
pub fn category_color(category: Category, palette: &Palette) -> Hsla {
    match category {
        Category::Images => palette.info,
        Category::Volumes => palette.info.opacity(0.6),
        Category::BuildCache => palette.indigo,
        Category::Snapshots => palette.gray,
        Category::Containers => palette.gray.opacity(0.55),
    }
}

fn this_computer() -> &'static str {
    if cfg!(target_os = "macos") {
        "Your Mac"
    } else {
        "This computer"
    }
}

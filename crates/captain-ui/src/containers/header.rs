use captain_core::store::ContainerFilter;
use gpui_kit::*;

use crate::theme::Palette;
use crate::widgets::{Segment, drag_region, segmented};
use crate::workspace::Workspace;

/// The page title, counts, and the filter. The empty parts drag the window.
pub fn render(
    handle: &Entity<Workspace>,
    workspace: &Workspace,
    palette: &Palette,
) -> impl IntoElement {
    let store = workspace.store();
    let running = store.active_count();
    let stopped = store.len() - running;
    let projects = store.projects().len();
    let summary = match projects {
        0 => format!("{running} running · {stopped} stopped"),
        1 => format!("{running} running · {stopped} stopped · 1 project"),
        n => format!("{running} running · {stopped} stopped · {n} projects"),
    };

    let segments = ContainerFilter::ALL
        .into_iter()
        .map(|filter| {
            let handle = handle.clone();
            Segment {
                label: filter.label().into(),
                selected: workspace.filter() == filter,
                on_click: Box::new(move |_, cx| {
                    handle.update(cx, |workspace, cx| workspace.set_filter(filter, cx));
                }),
            }
        })
        .collect();

    drag_region("containers-header")
        .flex_shrink_0()
        .flex()
        .items_center()
        .gap(px(12.))
        .px(px(24.))
        .pt(px(18.))
        .pb(px(14.))
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(
                    div()
                        .text_size(px(22.))
                        .font_weight(FontWeight::BOLD)
                        .child("Containers"),
                )
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(palette.text2)
                        .child(summary),
                ),
        )
        .child(
            segmented("filter", segments, palette)
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation()),
        )
}

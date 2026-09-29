use captain_core::store::UsageFilter;
use gpui_kit::*;

use super::VolumesView;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, Segment, segmented, text_button};

/// The right side of the page header: the prune buttons and the usage filter.
/// "Prune unused" removes anonymous volumes. "Prune all" asks first, then removes
/// named volumes too.
pub fn render(view: &VolumesView, cx: &mut Context<VolumesView>, palette: &Palette) -> Div {
    let enabled = view.engine.is_some() && !view.pruning;
    let prune = text_button(
        "prune-volumes",
        if view.pruning {
            "Pruning..."
        } else {
            "Prune unused"
        },
        ButtonTone::Accent,
        enabled,
        palette,
        cx.listener(|this, _, _, cx| this.prune(false, cx)),
    );
    let prune_all = text_button(
        "prune-all-volumes",
        "Prune all",
        ButtonTone::Danger,
        enabled,
        palette,
        cx.listener(|this, _, window, cx| this.confirm_prune_all(window, cx)),
    );

    div()
        .flex()
        .items_center()
        .gap(px(8.))
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .child(prune)
        .child(prune_all)
        .child(filter(view, cx, palette))
}

fn filter(view: &VolumesView, cx: &mut Context<VolumesView>, palette: &Palette) -> Div {
    let segments = UsageFilter::ALL
        .into_iter()
        .map(|filter| {
            let this = cx.entity().downgrade();
            Segment {
                label: filter.label().into(),
                selected: view.filter == filter,
                on_click: Box::new(move |_, cx| {
                    this.update(cx, |this, cx| {
                        this.filter = filter;
                        cx.notify();
                    })
                    .ok();
                }),
            }
        })
        .collect();
    segmented("volume-filter", segments, palette)
}

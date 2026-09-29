use captain_core::store::UsageFilter;
use gpui_kit::*;

use super::NetworksView;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, Segment, segmented, text_button};

/// The right side of the page header: the prune button and the usage filter.
pub fn render(view: &NetworksView, cx: &mut Context<NetworksView>, palette: &Palette) -> Div {
    let prune = text_button(
        "prune-networks",
        if view.pruning {
            "Pruning..."
        } else {
            "Prune unused"
        },
        ButtonTone::Accent,
        view.engine.is_some() && !view.pruning,
        palette,
        cx.listener(|this, _, _, cx| this.prune(cx)),
    );

    div()
        .flex()
        .items_center()
        .gap(px(8.))
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .child(prune)
        .child(filter(view, cx, palette))
}

fn filter(view: &NetworksView, cx: &mut Context<NetworksView>, palette: &Palette) -> Div {
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
    segmented("network-filter", segments, palette)
}

use captain_core::store::UsageFilter;
use gpui_kit::*;

use super::NetworksView;
use crate::help::HelpExt;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, Segment, segmented, text_button};

/// The right side of the page header: the prune button and the usage filter.
pub fn render(view: &NetworksView, cx: &mut Context<NetworksView>, palette: &Palette) -> Div {
    let generation = view.generation;
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
        cx.listener(move |this, _, _, cx| this.prune(generation, cx)),
    )
    .help("Remove the networks that no container uses. Built-in networks stay.");

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
                help: match filter {
                    UsageFilter::All => "Show all networks.",
                    UsageFilter::InUse => "Show only the networks that a container uses.",
                    UsageFilter::Unused => "Show only the networks that no container uses.",
                }
                .into(),
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

use captain_core::model::Network;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::NetworksView;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, list_row, pill, status_dot, text_button};

pub const SUBNET_WIDTH: f32 = 150.;
pub const CONTAINERS_WIDTH: f32 = 100.;
pub const SCOPE_WIDTH: f32 = 92.;

/// One network: usage dot, name and badges, driver and ID, subnet, containers, and the
/// scope, or Remove when selected.
pub fn render(
    network: &Network,
    selected: bool,
    removing: bool,
    generation: u64,
    handle: &Entity<NetworksView>,
    palette: &Palette,
) -> impl IntoElement {
    let dot = if network.is_in_use() {
        palette.green
    } else {
        palette.gray
    };
    let select = handle.clone();
    let id = network.id.clone();

    list_row(
        SharedString::from(format!("network-{}", network.id)),
        selected,
        palette,
    )
    .on_click(move |_, _, cx| {
        select.update(cx, |view, cx| view.select(id.clone(), cx));
    })
    .child(
        div()
            .flex_1()
            .min_w_0()
            .flex()
            .items_center()
            .gap(px(12.))
            .child(status_dot(dot, network.is_in_use(), palette))
            .child(name_cell(network, palette)),
    )
    .child(
        div()
            .w(px(SUBNET_WIDTH))
            .flex_shrink_0()
            .flex()
            .flex_col()
            .gap(px(2.))
            .font_family(palette.mono())
            .text_size(px(11.))
            .text_color(palette.text2)
            .child(network.subnet.clone().unwrap_or_else(|| "—".into()))
            .children(network.gateway.clone().map(|gateway| {
                div()
                    .text_color(palette.text3)
                    .child(format!("gw {gateway}"))
            })),
    )
    .child(
        div()
            .w(px(CONTAINERS_WIDTH))
            .flex_shrink_0()
            .text_size(px(12.))
            .text_color(if network.is_in_use() {
                palette.text
            } else {
                palette.text3
            })
            .child(network.usage_label()),
    )
    .child(trailing_cell(
        network, selected, removing, generation, handle, palette,
    ))
}

fn name_cell(network: &Network, palette: &Palette) -> Div {
    div()
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
                        .truncate()
                        .child(network.name.clone()),
                )
                .when(network.is_built_in(), |this| {
                    this.child(pill("built-in", palette.text2, palette.field))
                })
                .when(network.internal, |this| {
                    this.child(pill(
                        "internal",
                        palette.orange,
                        palette.tint(palette.orange),
                    ))
                }),
        )
        .child(
            div()
                .font_family(palette.mono())
                .text_size(px(11.))
                .text_color(palette.text2)
                .truncate()
                .child(format!("{} · {}", network.driver, network.short_id())),
        )
}

fn trailing_cell(
    network: &Network,
    selected: bool,
    removing: bool,
    generation: u64,
    handle: &Entity<NetworksView>,
    palette: &Palette,
) -> Div {
    let cell = div()
        .w(px(SCOPE_WIDTH))
        .flex_shrink_0()
        .flex()
        .justify_end()
        .items_center()
        .text_size(px(12.))
        .text_color(palette.text3);
    if removing {
        return cell.child("Removing...");
    }
    if !selected {
        return cell.child(network.scope.clone());
    }
    let handle = handle.clone();
    let id = network.id.clone();
    cell.child(text_button(
        SharedString::from(format!("remove-network-{}", network.id)),
        "Remove",
        ButtonTone::Danger,
        network.can_remove(),
        palette,
        move |_, _, cx| {
            cx.stop_propagation();
            handle.update(cx, |view, cx| view.remove(id.clone(), generation, cx));
        },
    ))
}

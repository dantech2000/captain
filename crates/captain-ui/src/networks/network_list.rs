use captain_core::model::Network;
use captain_core::store::{ResourceGroup, UsageFilter};
use gpui_kit::assets::IconName;
use gpui_kit::*;

use super::NetworksView;
use super::network_row::{self, CONTAINERS_WIDTH, SCOPE_WIDTH, SUBNET_WIDTH};
use crate::theme::Palette;
use crate::widgets::{Column, column_header, empty_note, group_card};

/// The column header and the networks in project cards, or a note when there are none.
pub fn render(
    view: &NetworksView,
    cx: &mut Context<NetworksView>,
    palette: &Palette,
) -> AnyElement {
    if !view.loaded {
        return div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .text_color(palette.text2)
            .child("Loading networks...")
            .into_any_element();
    }
    let groups = view.store.groups(view.filter);
    if groups.is_empty() {
        let (title, hint) = if view.filter == UsageFilter::All {
            (
                "No networks",
                "Create one above, or run `docker network create`.",
            )
        } else {
            (
                "No networks match this filter",
                "Choose All to see every network.",
            )
        };
        return empty_note(IconName::Network, title, hint, palette).into_any_element();
    }

    let handle = cx.entity();
    let cards = groups
        .into_iter()
        .map(|group| card(group, view, &handle, palette));
    let columns = [
        Column {
            label: "Subnet",
            width: SUBNET_WIDTH,
            right: false,
        },
        Column {
            label: "Containers",
            width: CONTAINERS_WIDTH,
            right: false,
        },
        Column {
            label: "Scope",
            width: SCOPE_WIDTH,
            right: true,
        },
    ];
    div()
        .size_full()
        .flex()
        .flex_col()
        .child(column_header(&columns, palette))
        .child(
            div()
                .id("network-list")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .px(px(12.))
                .pt(px(10.))
                .pb(px(16.))
                .child(div().flex().flex_col().gap(px(10.)).children(cards)),
        )
        .into_any_element()
}

fn card(
    group: ResourceGroup<Network>,
    view: &NetworksView,
    handle: &Entity<NetworksView>,
    palette: &Palette,
) -> Div {
    let in_use = group.items.iter().filter(|n| n.is_in_use()).count();
    let summary = match group.items.len() {
        1 => format!("1 network · {in_use} in use"),
        n => format!("{n} networks · {in_use} in use"),
    };
    let rows = group.items.iter().map(|network| {
        let selected = view.selected.as_deref() == Some(network.id.as_str());
        let removing = view.removing.contains(&network.id);
        network_row::render(
            network,
            selected,
            removing,
            view.generation,
            handle,
            palette,
        )
        .into_any_element()
    });
    group_card(group.project.as_deref(), summary, rows, palette)
}

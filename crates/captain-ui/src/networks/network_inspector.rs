use captain_core::model::{Network, NetworkDetail, NetworkEndpoint};
use gpui_kit::*;

use crate::icons::CaptainIcon;
use crate::theme::Palette;
use crate::widgets::{
    DetailHeader, container_link, detail_note, detail_panel, detail_section, key_values,
    map_or_note,
};
use crate::workspace::{Page, Workspace};

/// The right-hand panel for the selected network: its details, labels, and attached
/// containers with their addresses. `detail` is `None` while the inspect loads; the
/// list fields show until then.
pub fn render(
    network: &Network,
    detail: Option<&Result<NetworkDetail, String>>,
    handle: &Entity<Workspace>,
    workspace: &Workspace,
    palette: &Palette,
) -> Div {
    let color = network
        .compose_project
        .as_deref()
        .map_or(palette.teal, |p| palette.project_color(p));
    let header = DetailHeader {
        icon: CaptainIcon::Network,
        color,
        title: network.name.clone().into(),
        badge: network
            .is_built_in()
            .then(|| ("built-in".into(), palette.text2)),
        subtitle: format!("{} · {}", network.driver, network.usage_label()).into(),
    };
    let fallback = NetworkDetail::from_network(network.clone());
    let shown = match detail {
        Some(Ok(detail)) => detail,
        _ => &fallback,
    };
    let containers = match detail {
        None => detail_note("Loading containers...", palette),
        Some(Err(error)) => detail_note(format!("Could not inspect: {error}"), palette),
        Some(Ok(detail)) => endpoints(&detail.endpoints, handle, workspace, palette),
    };
    let body = div()
        .flex()
        .flex_col()
        .gap(px(18.))
        .child(detail_section("Details", details(shown, palette), palette))
        .child(detail_section("Containers", containers, palette))
        .child(detail_section(
            "Labels",
            map_or_note(&shown.network.labels, "No labels", palette),
            palette,
        ));
    detail_panel("network-inspector", header, body, palette)
}

fn details(detail: &NetworkDetail, palette: &Palette) -> Div {
    let network = &detail.network;
    let yes_no = |flag: bool| if flag { "yes" } else { "no" }.to_string();
    let rows = vec![
        ("ID".into(), network.short_id().to_string()),
        ("Driver".into(), network.driver.clone()),
        ("Scope".into(), network.scope.clone()),
        ("Subnets".into(), detail.subnets_label()),
        ("Gateway".into(), detail.gateways_label()),
        ("Internal".into(), yes_no(network.internal)),
        ("Attachable".into(), yes_no(detail.attachable)),
        ("IPv6".into(), yes_no(detail.ipv6)),
        ("Created".into(), detail.created_date().to_string()),
    ];
    key_values(rows, palette)
}

fn endpoints(
    endpoints: &[NetworkEndpoint],
    handle: &Entity<Workspace>,
    workspace: &Workspace,
    palette: &Palette,
) -> Div {
    if endpoints.is_empty() {
        return detail_note("No containers", palette);
    }
    div()
        .flex()
        .flex_col()
        .gap(px(6.))
        .children(endpoints.iter().map(|endpoint| {
            let dot = workspace
                .store()
                .find(&endpoint.container_id)
                .map_or(palette.gray, |c| palette.container_state(c.state));
            let address = endpoint.address().unwrap_or("no address");
            let line = match &endpoint.mac {
                Some(mac) => format!("{address} · {mac}"),
                None => address.to_string(),
            };
            let handle = handle.clone();
            let id = endpoint.container_id.clone();
            container_link(
                SharedString::from(format!("network-endpoint-{}", endpoint.container_id)),
                endpoint.name.clone(),
                line,
                dot,
                palette,
                move |_, _, cx| {
                    handle.update(cx, |workspace, cx| {
                        workspace.select(id.clone(), cx);
                        workspace.set_page(Page::Containers, cx);
                    });
                },
            )
        }))
}

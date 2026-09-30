//! One card per namespace, with a row per Service port: Forward, or the local
//! address and Stop.

use captain_core::kubernetes::{ForwardKey, KubeService};
use gpui_kit::*;

use super::{ForwardingModel, forward_dialog};
use crate::help::HelpExt;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, settings_card, settings_row, text_button};

/// The cards for `services`, which come sorted by namespace.
pub fn render(
    handle: &Entity<ForwardingModel>,
    model: &ForwardingModel,
    services: &[KubeService],
    palette: &Palette,
) -> Vec<AnyElement> {
    let mut cards = Vec::new();
    let mut start = 0;
    while start < services.len() {
        let namespace = &services[start].namespace;
        let end = services[start..]
            .iter()
            .position(|service| service.namespace != *namespace)
            .map_or(services.len(), |offset| start + offset);
        let rows = services[start..end]
            .iter()
            .flat_map(|service| rows(handle, model, service, palette));
        cards.push(settings_card(namespace.clone(), rows, palette).into_any_element());
        start = end;
    }
    cards
}

fn rows(
    handle: &Entity<ForwardingModel>,
    model: &ForwardingModel,
    service: &KubeService,
    palette: &Palette,
) -> Vec<AnyElement> {
    service
        .ports
        .iter()
        .map(|port| {
            let key = ForwardKey {
                namespace: service.namespace.clone(),
                service: service.name.clone(),
                port: port.port,
            };
            let label = format!("{}:{}", service.name, port.port);
            let note = port.name.clone().map(SharedString::from);
            let id = SharedString::from(format!(
                "forward-{}-{}-{}",
                key.namespace, key.service, key.port
            ));
            let handle = handle.clone();
            let control = match model.local_port(&key) {
                Some(local) => div()
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .child(
                        div()
                            .text_color(palette.text2)
                            .child(format!("127.0.0.1:{local}")),
                    )
                    .child(
                        text_button(
                            id,
                            "Stop",
                            ButtonTone::Danger,
                            true,
                            palette,
                            move |_, _, cx| handle.update(cx, |model, cx| model.stop(&key, cx)),
                        )
                        .help(format!("Stop forwarding 127.0.0.1:{local} to {label}.")),
                    ),
                None => div().child(
                    text_button(
                        id,
                        "Forward",
                        ButtonTone::Accent,
                        true,
                        palette,
                        move |_, window, cx| {
                            forward_dialog::open(handle.clone(), key.clone(), window, cx)
                        },
                    )
                    .help(format!("Forward a port on this computer to {label}.")),
                ),
            };
            settings_row(label, note, control, palette).into_any_element()
        })
        .collect()
}

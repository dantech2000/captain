use captain_core::model::{Container, PortLink};
use gpui_kit::*;

use super::group_info::service_name;
use super::{ProjectNotice, ProjectView};
use crate::help::HelpExt;
use crate::theme::Palette;

/// A published port of one service.
pub struct Door {
    pub service: String,
    pub public: u16,
    pub link: PortLink,
}

/// The published ports of `containers`, one per host port, in order.
pub fn doors(containers: &[&Container]) -> Vec<Door> {
    let mut doors: Vec<Door> = Vec::new();
    for container in containers {
        for port in &container.ports {
            let Some(public) = port.public_port else {
                continue;
            };
            if doors.iter().any(|door| door.public == public) {
                continue;
            }
            doors.push(Door {
                service: service_name(container),
                public,
                link: PortLink::of(public, port.private_port),
            });
        }
    }
    doors
}

/// Opens the page of a web port, or copies the address of any other port.
pub fn follow(link: &PortLink, view: &WeakEntity<ProjectView>, cx: &mut App) {
    match link {
        PortLink::Open(url) => cx.open_url(url),
        PortLink::Copy { address, .. } => {
            cx.write_to_clipboard(ClipboardItem::new_string(address.clone()));
            let address = address.clone();
            view.update(cx, |_, cx| cx.emit(ProjectNotice::Copied { address }))
                .ok();
        }
    }
}

/// The status bar sentence for a port of `service`.
pub fn door_help(link: &PortLink, service: &str) -> String {
    match link {
        PortLink::Open(url) => format!("Open {url} in your browser ({service})."),
        PortLink::Copy {
            address,
            service: kind,
        } => {
            format!("Copy {address}. {kind} does not serve web pages.")
        }
    }
}

/// "Open", then one pill per published port: the service and `localhost:PORT`.
pub fn render(doors: Vec<Door>, view: &WeakEntity<ProjectView>, palette: &Palette) -> Div {
    let hover = palette.nav_selected;
    div()
        .flex_shrink_0()
        .flex()
        .flex_wrap()
        .items_center()
        .gap(px(8.))
        .px(px(28.))
        .pb(px(16.))
        .child(
            div()
                .mr(px(4.))
                .text_size(px(11.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(palette.text3)
                .child("Open"),
        )
        .children(doors.into_iter().map(|door| {
            let view = view.clone();
            let help = door_help(&door.link, &door.service);
            let link = door.link.clone();
            div()
                .id(SharedString::from(format!("door-{}", door.public)))
                .h(px(28.))
                .px(px(12.))
                .flex()
                .items_center()
                .gap(px(8.))
                .rounded(px(14.))
                .border_1()
                .border_color(palette.sep)
                .bg(palette.field)
                .text_size(px(12.))
                .cursor_pointer()
                .hover(move |style| style.bg(hover))
                .on_click(move |_, _, cx| follow(&link, &view, cx))
                .child(div().font_weight(FontWeight::SEMIBOLD).child(door.service))
                .child(
                    div()
                        .font_family(palette.mono())
                        .text_color(palette.link)
                        .child(format!("localhost:{}", door.public)),
                )
                .help(help)
        }))
}

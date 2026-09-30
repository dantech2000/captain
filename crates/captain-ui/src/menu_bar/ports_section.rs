//! Open ports: each published port with its service. A click opens it in the
//! browser, or copies the address of a database port.

use gpui_kit::*;

use super::popover_data::PortRow;
use super::projects_section::heading;
use crate::help::HelpExt;
use crate::theme::Palette;

pub fn render(ports: &[PortRow], palette: &Palette) -> Div {
    div()
        .flex()
        .flex_col()
        .pt(px(4.))
        .child(heading("Open ports", palette))
        .children(ports.iter().map(|port| port_row(port, palette)))
}

fn port_row(port: &PortRow, palette: &Palette) -> Stateful<Div> {
    let address = format!("localhost:{}", port.port);
    let help = if port.copies {
        format!("Copy {address}, the address of {}.", port.service)
    } else {
        format!("Open http://{address} ({}) in your browser.", port.service)
    };
    let copies = port.copies;
    let target = address.clone();
    let hover = palette.hover;
    div()
        .id(SharedString::from(format!("popover-port-{}", port.port)))
        .h(px(30.))
        .px(px(16.))
        .flex()
        .items_center()
        .gap(px(10.))
        .cursor_pointer()
        .hover(move |style| style.bg(hover))
        .on_click(move |_, _, cx| {
            if copies {
                cx.write_to_clipboard(ClipboardItem::new_string(target.clone()));
            } else {
                cx.open_url(&format!("http://{target}"));
            }
        })
        .child(
            div()
                .flex_1()
                .min_w_0()
                .truncate()
                .child(port.service.clone()),
        )
        .child(
            div()
                .font_family(palette.mono())
                .text_size(px(12.))
                .text_color(palette.link)
                .child(address),
        )
        .child(
            div()
                .w(px(30.))
                .text_size(px(11.))
                .text_color(palette.text3)
                .child(if copies { "Copy" } else { "" }),
        )
        .help(help)
}

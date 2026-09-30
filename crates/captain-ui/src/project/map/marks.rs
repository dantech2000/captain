use captain_core::format::bytes_label;
use captain_core::model::PortLink;
use captain_core::project_map::{Lane, Placed, Rect};
use gpui_kit::*;

use super::scale::Scale;
use crate::help::HelpExt;
use crate::icons::{CaptainIcon, cap_icon};
use crate::project::ProjectView;
use crate::project::open_row::{door_help, follow};
use crate::theme::Palette;

/// A host port pin: `:8080 ↗`. A click opens or copies the address, as the Open row
/// does.
pub fn pin(
    port: u16,
    private: u16,
    service: &str,
    rect: Rect,
    z: Scale,
    view: &WeakEntity<ProjectView>,
    palette: &Palette,
) -> Div {
    let link = PortLink::of(port, private);
    let arrow = match link {
        PortLink::Open(_) => "↗",
        PortLink::Copy { .. } => "⧉",
    };
    let help = door_help(&link, service);
    let view = view.clone();
    z.place(rect).child(
        div()
            .id(SharedString::from(format!("map-pin-{port}")))
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .rounded_full()
            .bg(palette.tint(palette.accent))
            .border_1()
            .border_color(palette.accent.alpha(0.5))
            .text_color(palette.link)
            .font_family(palette.mono())
            .text_size(z.px(11.5))
            .cursor_pointer()
            .on_click(move |_, _, cx| follow(&link, &view, cx))
            .child(format!(":{port} {arrow}"))
            .help(help),
    )
}

/// A volume node: its glyph, name, and size.
pub fn volume(placed: &Placed, size: Option<u64>, z: Scale, palette: &Palette) -> Div {
    let name = &placed.key;
    let size_label = size.map_or("Size unknown".to_string(), bytes_label);
    z.place(placed.rect).child(
        div()
            .id(SharedString::from(format!("map-volume-{name}")))
            .size_full()
            .flex()
            .items_center()
            .gap(z.px(10.))
            .px(z.px(12.))
            .rounded(z.px(12.))
            .bg(palette.card)
            .border_1()
            .border_color(palette.info.alpha(0.4))
            .child(cap_icon(CaptainIcon::Volume, z.px(18.), palette.info))
            .child(
                div()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(z.px(2.))
                    .child(
                        div()
                            .truncate()
                            .text_size(z.px(12.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .font_family(palette.mono())
                            .child(name.clone()),
                    )
                    .child(
                        div()
                            .text_size(z.px(11.))
                            .text_color(palette.text3)
                            .child(size_label.clone()),
                    ),
            )
            .help(format!(
                "The volume {name} ({size_label}). The dashed lines show the services that mount it."
            )),
    )
}

/// A network's dashed lane, with its name on top.
pub fn lane(lane: &Lane, z: Scale, palette: &Palette) -> Div {
    let (label, help) = if lane.network.is_empty() {
        (
            "No network".to_string(),
            "Services with no network, or not inspected yet.".to_string(),
        )
    } else {
        (
            lane.network.clone(),
            format!(
                "The network {}. Its services reach each other by service name.",
                lane.network
            ),
        )
    };
    z.place(lane.rect).child(
        div()
            .size_full()
            .rounded(z.px(18.))
            .border_1()
            .border_dashed()
            .border_color(palette.info.alpha(0.35))
            .bg(palette.info.alpha(0.025))
            .px(z.px(16.))
            .pt(z.px(8.))
            .child(
                div()
                    .id(SharedString::from(format!("map-lane-{}", lane.network)))
                    .flex()
                    .items_center()
                    .gap(z.px(6.))
                    .text_size(z.px(11.))
                    .text_color(palette.info)
                    .child(cap_icon(CaptainIcon::Network, z.px(13.), palette.info))
                    .child(label)
                    .help(help),
            ),
    )
}

/// A column label: "This Mac" or "Volumes".
pub fn column_label(text: &'static str, at: (f32, f32), z: Scale, palette: &Palette) -> Div {
    div()
        .absolute()
        .left(z.px(at.0))
        .top(z.px(at.1))
        .text_size(z.px(11.))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(palette.text3)
        .child(text)
}

//! The first-launch screen: set up Captain Engine, or use an engine that is
//! already on this computer. See ADR 0008 and ADR 0009.

use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::*;

use super::HostModel;
use super::host_screen::{frame, note, page, title};
use crate::theme::Palette;
use crate::widgets::{ButtonTone, primary_button, text_button};

pub fn render(model: &Entity<HostModel>, host: &HostModel, palette: &Palette) -> Stateful<Div> {
    let start = model.clone();
    let external = model.clone();
    let column = frame(palette)
        .child(title("Set up Captain Engine"))
        .child(note(
            "Captain runs Docker in its own small Linux VM. The first start downloads \
             about 600 MB and takes a few minutes.",
            palette,
        ))
        .child(resources(host, palette))
        .children((!host.detected().is_empty()).then(|| other_engines(model, host, palette)))
        .child(
            div()
                .pt(px(6.))
                .flex()
                .items_center()
                .gap(px(12.))
                .child(primary_button(
                    "host-setup",
                    "Set up Captain Engine",
                    "Download and create the Captain Engine virtual machine.",
                    true,
                    palette,
                    move |_, _, cx| start.update(cx, |model, cx| model.start(cx)),
                ))
                .child(text_button(
                    "host-use-existing",
                    "Use an existing engine",
                    ButtonTone::Accent,
                    true,
                    palette,
                    move |_, _, cx| external.update(cx, |model, cx| model.use_external(cx)),
                )),
        );
    page(column)
}

fn resources(host: &HostModel, palette: &Palette) -> Div {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(2.))
        .px(px(14.))
        .py(px(8.))
        .rounded(px(10.))
        .bg(palette.card)
        .border_1()
        .border_color(palette.sep)
        .child(
            div()
                .font_weight(FontWeight::MEDIUM)
                .child(host.resources().summary()),
        )
        .child(
            div()
                .text_size(px(11.))
                .text_color(palette.text2)
                .child("You can change these in Settings later."),
        )
}

/// The engines Captain found, and the offer to copy their data after setup.
fn other_engines(model: &Entity<HostModel>, host: &HostModel, palette: &Palette) -> Div {
    let toggle = model.clone();
    let rows = host.detected().iter().map(|engine| {
        div()
            .flex()
            .justify_between()
            .gap(px(12.))
            .text_size(px(12.))
            .child(div().text_color(palette.text2).child(engine.source.clone()))
            .child(
                div()
                    .font_family(palette.mono())
                    .truncate()
                    .child(engine.host.clone()),
            )
    });
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(8.))
        .p(px(14.))
        .rounded(px(12.))
        .bg(palette.group)
        .border_1()
        .border_color(palette.sep)
        .child(
            div()
                .text_size(px(12.))
                .font_weight(FontWeight::SEMIBOLD)
                .child("Other engines on this computer"),
        )
        .children(rows)
        .child(
            Checkbox::new("host-setup-migrate")
                .label("Bring your data along")
                .checked(host.migrate_after_setup())
                .on_click(move |checked, _, cx| {
                    toggle.update(cx, |model, cx| model.set_migrate_after_setup(*checked, cx));
                }),
        )
        .child(div().text_size(px(11.)).text_color(palette.text2).child(
            "After setup, the Migration Assistant copies volumes, images, and \
                     projects. The other engine stays as it is.",
        ))
}

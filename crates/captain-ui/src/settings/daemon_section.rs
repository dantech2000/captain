//! The Docker daemon card: registry mirrors, insecure registries, custom
//! `daemon.json` keys, and the TCP socket of Captain Engine. See feature 0020.

use captain_core::daemon::DaemonSettings;
use captain_core::settings::EngineChoice;
use gpui_kit::component::Sizable;
use gpui_kit::component::input::{Input, Textarea};
use gpui_kit::component::switch::Switch;
use gpui_kit::*;

use super::SettingsView;
use super::daemon_form::DaemonForm;
use crate::engine_host::HostModel;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, inline_error, settings_card, settings_row, text_button};

/// The card, or `None` unless Captain controls the chosen Captain Engine.
pub fn render(
    model: &Entity<HostModel>,
    form: &DaemonForm,
    palette: &Palette,
    cx: &mut Context<SettingsView>,
) -> Option<Div> {
    let host = model.read(cx);
    if host.choice(cx) != EngineChoice::Captain || !host.can_control() {
        return None;
    }
    let daemon = host.daemon(cx);
    let mut rows = Vec::new();
    if host.daemon_needs_restart(cx) {
        rows.push(restart_row(model, palette));
    }
    rows.push(area_row(
        "Registry mirrors",
        "One URL per line, with https:// or http://.",
        Textarea::new(&form.mirrors),
        palette,
    ));
    rows.push(area_row(
        "Insecure registries",
        "One host:port or CIDR per line. Docker uses plain HTTP for them.",
        Textarea::new(&form.insecure),
        palette,
    ));
    rows.push(area_row(
        "Custom daemon.json",
        "A JSON object with other dockerd keys. Captain manages hosts, containerd, and the two features Captain Engine needs.",
        Textarea::new(&form.custom),
        palette,
    ));
    rows.push(tcp_row(model, &daemon, form, palette));
    rows.push(save_row(form, palette, cx));
    Some(settings_card("Docker daemon", rows, palette))
}

fn restart_row(model: &Entity<HostModel>, palette: &Palette) -> AnyElement {
    let model = model.clone();
    settings_row(
        div()
            .text_color(palette.warn_text)
            .child("Restart to apply"),
        Some("Captain Engine runs with the previous daemon settings.".into()),
        text_button(
            "daemon-restart",
            "Restart",
            ButtonTone::Accent,
            true,
            palette,
            move |_, _, cx| model.update(cx, |model, cx| model.restart(cx)),
        ),
        palette,
    )
    .into_any_element()
}

fn area_row(
    label: &'static str,
    note: &'static str,
    area: Textarea,
    palette: &Palette,
) -> AnyElement {
    settings_row(
        label,
        Some(note.into()),
        div().w(px(320.)).child(area.small()),
        palette,
    )
    .into_any_element()
}

fn tcp_row(
    model: &Entity<HostModel>,
    daemon: &DaemonSettings,
    form: &DaemonForm,
    palette: &Palette,
) -> AnyElement {
    let note = format!(
        "Listens on tcp://127.0.0.1:{} on this Mac. There is no TLS: any program on this Mac can then control Docker, with root in the VM.",
        daemon.tcp_port
    );
    let current = daemon.clone();
    let model = model.clone();
    let control =
        div()
            .flex()
            .items_center()
            .gap(px(8.))
            .child(div().w(px(80.)).child(Input::new(&form.port).small()))
            .child(Switch::new("daemon-tcp").checked(daemon.tcp).on_click(
                move |checked, _, cx| {
                    let daemon = DaemonSettings {
                        tcp: *checked,
                        ..current.clone()
                    };
                    model.update(cx, |model, cx| model.set_daemon(daemon, cx));
                },
            ));
    settings_row(
        "Expose the Docker API on TCP",
        Some(note.into()),
        control,
        palette,
    )
    .into_any_element()
}

fn save_row(form: &DaemonForm, palette: &Palette, cx: &mut Context<SettingsView>) -> AnyElement {
    let note: SharedString = "Changes apply the next time Captain Engine starts.".into();
    let control = div()
        .flex()
        .flex_col()
        .items_end()
        .gap(px(4.))
        .child(text_button(
            "daemon-save",
            "Save",
            ButtonTone::Accent,
            true,
            palette,
            cx.listener(|view, _, window, cx| view.save_daemon(window, cx)),
        ))
        .children(
            form.error
                .clone()
                .map(|error| inline_error(error, palette).max_w(px(320.))),
        );
    settings_row("Save", Some(note), control, palette).into_any_element()
}

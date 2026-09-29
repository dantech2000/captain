//! The fields of the Docker daemon card. They need a window, so the Settings page
//! creates them on its first render. See feature 0020.

use captain_core::daemon::{DaemonFields, DaemonSettings, parse_fields};
use gpui_kit::component::input::{InputState, TextareaState};
use gpui_kit::*;

use super::SettingsView;

/// The text fields, and the result of the last Save.
pub struct DaemonForm {
    pub mirrors: Entity<TextareaState>,
    pub insecure: Entity<TextareaState>,
    pub custom: Entity<TextareaState>,
    pub port: Entity<InputState>,
    /// Why the last Save was refused.
    pub error: Option<SharedString>,
}

impl DaemonForm {
    /// Fields that show `daemon`.
    pub fn new(daemon: &DaemonSettings, window: &mut Window, cx: &mut App) -> Self {
        let area = |placeholder: &'static str, rows: usize, window: &mut Window, cx: &mut App| {
            cx.new(|cx| {
                TextareaState::new(window, cx)
                    .auto_grow(rows, 12)
                    .placeholder(placeholder)
            })
        };
        let form = Self {
            mirrors: area("https://mirror.gcr.io", 2, window, cx),
            insecure: area("registry.local:5000", 2, window, cx),
            custom: area("{\"log-level\": \"warn\"}", 4, window, cx),
            port: cx.new(|cx| InputState::new(window, cx)),
            error: None,
        };
        form.show(daemon, window, cx);
        form
    }

    /// Puts `daemon` in the fields, one list entry per line and the JSON indented.
    pub fn show(&self, daemon: &DaemonSettings, window: &mut Window, cx: &mut App) {
        let texts = [
            (&self.mirrors, daemon.registry_mirrors.join("\n")),
            (&self.insecure, daemon.insecure_registries.join("\n")),
            (&self.custom, daemon.custom_text()),
        ];
        for (area, text) in texts {
            area.update(cx, |area, cx| area.set_value(text, window, cx));
        }
        let port = daemon.tcp_port.to_string();
        self.port
            .update(cx, |input, cx| input.set_value(port, window, cx));
    }

    /// The settings in the fields, with the TCP switch from `current`, or the first
    /// problem.
    pub fn read(&self, current: &DaemonSettings, cx: &App) -> Result<DaemonSettings, String> {
        let mirrors = self.mirrors.read(cx).value();
        let insecure = self.insecure.read(cx).value();
        let custom = self.custom.read(cx).value();
        let port = self.port.read(cx).value();
        parse_fields(DaemonFields {
            registry_mirrors: &mirrors,
            insecure_registries: &insecure,
            custom: &custom,
            tcp: current.tcp,
            tcp_port: &port,
        })
    }
}

impl SettingsView {
    /// Checks the Docker daemon fields and saves them, or shows the first problem.
    pub(super) fn save_daemon(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (Some(host), Some(form)) = (self.host.clone(), self.daemon_form.as_mut()) else {
            return;
        };
        let current = host.read(cx).daemon(cx);
        match form.read(&current, cx) {
            Ok(daemon) => {
                form.error = None;
                form.show(&daemon, window, cx);
                host.update(cx, |model, cx| model.set_daemon(daemon, cx));
            }
            Err(error) => form.error = Some(error.into()),
        }
        cx.notify();
    }
}

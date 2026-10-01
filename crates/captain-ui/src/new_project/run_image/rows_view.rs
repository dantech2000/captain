use gpui_kit::assets::IconName;
use gpui_kit::*;

use super::form_state::RunImage;
use crate::help::HelpExt;
use crate::new_project::form::{field, text_input};
use crate::theme::Palette;
use crate::widgets::{ButtonTone, icon_button, section_note, text_button};

/// Which row list a button acts on, as [`RunImage::remove_row`] numbers them.
const PORTS: u8 = 0;
const ENV: u8 = 1;
const VOLUMES: u8 = 2;

impl RunImage {
    pub(super) fn ports_field(&self, palette: &Palette, cx: &mut Context<Self>) -> Div {
        let this = cx.entity().downgrade();
        let rows = self
            .ports
            .iter()
            .enumerate()
            .map(|(ix, (host, container))| {
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .child(div().w(px(100.)).child(text_input(
                        ("run-port-host", ix),
                        host,
                        "The port on this computer. Leave it empty to keep the port inside.",
                    )))
                    .child(div().text_color(palette.text2).child("\u{2192}"))
                    .child(div().w(px(100.)).child(text_input(
                        ("run-port-container", ix),
                        container,
                        "The port in the container, such as 80 or 53/udp.",
                    )))
                    .child(self.remove_button(PORTS, ix, "Remove this port.", palette, &this))
            });
        let note = self.ports.is_empty().then(|| {
            section_note(
                "No ports. Add one to reach the container from this computer.",
                palette,
            )
        });
        let add = self.add_button(
            "run-add-port",
            "Add port",
            "Publish another container port.",
            palette,
            &this,
            |this, window, cx| this.add_port(String::new(), String::new(), window, cx),
        );
        field("Ports", rows_body(rows, note, add), None, palette)
    }

    pub(super) fn env_field(&self, palette: &Palette, cx: &mut Context<Self>) -> Div {
        let this = cx.entity().downgrade();
        let rows = self.env.iter().enumerate().map(|(ix, input)| {
            div()
                .flex()
                .items_center()
                .gap(px(6.))
                .child(text_input(
                    ("run-env", ix),
                    input,
                    "One environment variable as KEY=value.",
                ))
                .child(self.remove_button(
                    ENV,
                    ix,
                    "Remove this environment variable.",
                    palette,
                    &this,
                ))
        });
        let add = self.add_button(
            "run-add-env",
            "Add variable",
            "Set another environment variable.",
            palette,
            &this,
            |this, window, cx| this.add_env(window, cx),
        );
        field("Environment", rows_body(rows, None, add), None, palette)
    }

    pub(super) fn volumes_field(&self, palette: &Palette, cx: &mut Context<Self>) -> Div {
        let this = cx.entity().downgrade();
        let rows = self.volumes.iter().enumerate().map(|(ix, (source, target))| {
            div()
                .flex()
                .items_center()
                .gap(px(6.))
                .child(text_input(
                    ("run-volume-source", ix),
                    source,
                    "A volume name such as data, or a folder. A folder like ./data is inside the project folder.",
                ))
                .child(div().text_color(palette.text2).child("\u{2192}"))
                .child(text_input(
                    ("run-volume-target", ix),
                    target,
                    "The absolute path in the container, such as /var/lib/data.",
                ))
                .child(self.remove_button(VOLUMES, ix, "Remove this volume.", palette, &this))
        });
        let add = self.add_button(
            "run-add-volume",
            "Add volume",
            "Keep a container path in a named volume or a folder.",
            palette,
            &this,
            |this, window, cx| this.add_volume(window, cx),
        );
        field("Volumes", rows_body(rows, None, add), None, palette)
    }

    fn remove_button(
        &self,
        list: u8,
        ix: usize,
        help: &'static str,
        palette: &Palette,
        this: &WeakEntity<Self>,
    ) -> Stateful<Div> {
        let this = this.clone();
        icon_button(
            ("run-remove", usize::from(list) * 1000 + ix),
            IconName::Minus,
            help,
            palette,
            move |_, _, cx| {
                this.update(cx, |this, cx| this.remove_row(list, ix, cx))
                    .ok();
            },
        )
    }

    fn add_button(
        &self,
        id: &'static str,
        label: &'static str,
        help: &'static str,
        palette: &Palette,
        this: &WeakEntity<Self>,
        add: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
    ) -> Stateful<Div> {
        let this = this.clone();
        text_button(
            id,
            label,
            ButtonTone::Accent,
            true,
            palette,
            move |_, window, cx| {
                this.update(cx, |this, cx| add(this, window, cx)).ok();
            },
        )
        .help(help)
    }
}

fn rows_body(rows: impl Iterator<Item = Div>, note: Option<Div>, add: Stateful<Div>) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(px(6.))
        .children(rows)
        .children(note)
        .child(div().flex().child(add))
}

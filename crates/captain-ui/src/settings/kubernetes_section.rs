//! The Kubernetes section: one switch, and when it is on, the version, the status
//! line, Apply, and Reset. The port and Traefik live in the settings file. See
//! features 0024 and 0037.

use captain_core::HostResources;
use captain_core::format::bytes_label;
use captain_core::kubernetes::{KubernetesStatus, RECOMMENDED_MEMORY};
use captain_core::settings::EngineChoice;
use gpui_kit::component::Sizable;
use gpui_kit::component::select::Select;
use gpui_kit::component::switch::Switch;
use gpui_kit::*;

use super::kube_dialogs;
use super::kube_form::KubeForm;
use super::page_section::{fill_note, row, section, sub_row, under_note};
use crate::engine_host::HostModel;
use crate::help::HelpExt;
use crate::kubernetes::KubernetesModel;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, text_button};

/// The section, or `None` unless Captain controls a Captain Engine with a cluster.
pub fn render(
    model: &Entity<KubernetesModel>,
    host: &Entity<HostModel>,
    form: &KubeForm,
    palette: &Palette,
    cx: &App,
) -> Option<Div> {
    let engine = host.read(cx);
    if engine.choice(cx) != EngineChoice::Captain || !engine.can_control() {
        return None;
    }
    let kube = model.read(cx);
    let settings = kube.settings(cx);
    let switch_model = model.clone();
    let head = row("Kubernetes", palette)
        .id("settings-kubernetes")
        .child(fill_note(
            "A k3s cluster inside Captain Engine. Off uses no memory.",
            palette,
        ))
        .child(
            Switch::new("kubernetes-enabled")
                .checked(settings.enabled)
                .on_click(move |checked, _, cx| {
                    let on = *checked;
                    switch_model.update(cx, |model, cx| model.turn_on(on, cx));
                }),
        )
        .help("Turn the k3s cluster in Captain Engine on or off.");
    let section = section(palette).child(head);
    if !settings.enabled {
        return Some(section);
    }
    let enabled = engine.status().is_running() && !kube.is_busy();
    let (apply_model, reset_model) = (model.clone(), model.clone());
    let version = sub_row("Version", palette)
        .child(
            div()
                .id("settings-kubernetes-version")
                .w(px(260.))
                .child(
                    Select::new(&form.version)
                        .small()
                        .placeholder("Loading versions\u{2026}"),
                )
                .help("Choose the k3s version. An upgrade keeps the workloads; a downgrade resets the cluster."),
        )
        .child(div().flex_1())
        .child(
            text_button(
                "kubernetes-apply",
                "Apply now",
                ButtonTone::Accent,
                enabled,
                palette,
                move |_, _, cx| apply_model.update(cx, |model, cx| model.apply(cx)),
            )
            .help("Apply the Kubernetes settings to the running engine now. The cluster restarts."),
        )
        .child(
            text_button(
                "kubernetes-reset",
                "Reset\u{2026}",
                ButtonTone::Danger,
                enabled,
                palette,
                move |_, window, cx| kube_dialogs::reset(reset_model.clone(), window, cx),
            )
            .help("Delete the Kubernetes workloads and the cluster state. Images stay. Captain asks first."),
        );
    let status = status_line(kube, palette);
    let memory = (engine.resources().memory_bytes < RECOMMENDED_MEMORY)
        .then(|| memory_row(host, engine.resources(), engine.machine(), palette));
    Some(section.child(version).child(status).children(memory))
}

/// What the cluster does now, or the step of an apply or reset.
fn status_line(kube: &KubernetesModel, palette: &Palette) -> Div {
    let status = kube.status();
    let color = match status {
        KubernetesStatus::Running { .. } => palette.green,
        KubernetesStatus::Starting => palette.orange,
        KubernetesStatus::Off => palette.gray,
        KubernetesStatus::Failed(_) => palette.red,
    };
    let text: SharedString = kube.step().unwrap_or_else(|| match status {
        KubernetesStatus::Running { version } => {
            format!("k3s {version} runs. kubectl uses the captain context.").into()
        }
        KubernetesStatus::Failed(why) => why.clone().into(),
        other => format!(
            "{}. Changes apply the next time the engine starts.",
            other.label()
        )
        .into(),
    });
    under_note("", palette).child(
        div()
            .flex()
            .items_center()
            .gap(px(6.))
            .child(div().size(px(6.)).rounded_full().bg(color))
            .child(text),
    )
}

fn memory_row(
    host: &Entity<HostModel>,
    current: HostResources,
    machine: HostResources,
    palette: &Palette,
) -> Stateful<Div> {
    let target = RECOMMENDED_MEMORY.min(machine.memory_bytes);
    let host = host.clone();
    sub_row("", palette)
        .id("settings-kubernetes-memory")
        .child(
            div()
                .flex_1()
                .text_size(px(12.))
                .text_color(palette.warn_text)
                .child(format!(
                    "Kubernetes needs about 2 GB of its own. Captain recommends {} for the engine.",
                    bytes_label(RECOMMENDED_MEMORY)
                )),
        )
        .child(text_button(
            "kubernetes-memory",
            format!("Use {}", bytes_label(target)),
            ButtonTone::Accent,
            target > current.memory_bytes,
            palette,
            move |_, _, cx| {
                host.update(cx, |host, cx| {
                    let resources = HostResources {
                        memory_bytes: target,
                        ..host.resources()
                    };
                    host.set_resources(resources, cx);
                });
            },
        ))
        .help("Give Captain Engine the memory that Kubernetes needs. It applies on the next start.")
}

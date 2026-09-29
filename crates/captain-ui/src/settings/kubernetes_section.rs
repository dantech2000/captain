//! The Kubernetes card: the switch, version, port, Traefik, Apply, and Reset. See
//! feature 0024 and Rancher Desktop's Kubernetes preferences.

use captain_core::HostResources;
use captain_core::format::bytes_label;
use captain_core::kubernetes::{KubernetesSettings, KubernetesStatus, RECOMMENDED_MEMORY};
use captain_core::settings::EngineChoice;
use gpui_kit::component::Sizable;
use gpui_kit::component::input::Input;
use gpui_kit::component::select::Select;
use gpui_kit::component::switch::Switch;
use gpui_kit::*;

use super::SettingsView;
use super::kube_dialogs;
use super::kube_form::KubeForm;
use crate::engine_host::HostModel;
use crate::kubernetes::KubernetesModel;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, inline_error, pill, settings_card, settings_row, text_button};

/// The card, or `None` unless Captain controls a Captain Engine with a cluster.
pub fn render(
    model: &Entity<KubernetesModel>,
    host: &Entity<HostModel>,
    form: &KubeForm,
    palette: &Palette,
    cx: &mut Context<SettingsView>,
) -> Option<Div> {
    let engine = host.read(cx);
    if engine.choice(cx) != EngineChoice::Captain || !engine.can_control() {
        return None;
    }
    let kube = model.read(cx);
    let settings = kube.settings(cx);
    let mut rows = vec![switch_row(model, kube, &settings, palette)];
    if settings.enabled {
        rows.push(version_row(form, palette));
        rows.push(port_row(form, palette));
        rows.push(traefik_row(model, &settings, palette));
        if engine.resources().memory_bytes < RECOMMENDED_MEMORY {
            rows.push(memory_row(
                host,
                engine.resources(),
                engine.machine(),
                palette,
            ));
        }
    }
    let running = engine.status().is_running();
    let (busy, step) = (kube.is_busy(), kube.step());
    rows.push(apply_row(step, form, running && !busy, palette, cx));
    rows.push(reset_row(model, running && !busy, palette));
    Some(settings_card("Kubernetes", rows, palette))
}

fn switch_row(
    model: &Entity<KubernetesModel>,
    kube: &KubernetesModel,
    settings: &KubernetesSettings,
    palette: &Palette,
) -> AnyElement {
    let status = kube.status();
    let color = match status {
        KubernetesStatus::Running { .. } => palette.green,
        KubernetesStatus::Starting => palette.orange,
        KubernetesStatus::Off => palette.gray,
        KubernetesStatus::Failed(_) => palette.red,
    };
    let note: SharedString = match status {
        KubernetesStatus::Running { version } => {
            format!("k3s {version} runs in Captain Engine. kubectl uses the captain context.").into()
        }
        KubernetesStatus::Failed(why) => why.clone().into(),
        _ => "A k3s cluster in Captain Engine. Images you build with Docker run in pods without a push.".into(),
    };
    let model = model.clone();
    let control = div()
        .flex()
        .items_center()
        .gap(px(8.))
        .child(pill(status.label(), color, palette.tint(color)))
        .child(
            Switch::new("kubernetes-enabled")
                .checked(settings.enabled)
                .on_click(move |checked, _, cx| {
                    let on = *checked;
                    model.update(cx, |model, cx| model.turn_on(on, cx));
                }),
        );
    settings_row("Enable Kubernetes", Some(note), control, palette).into_any_element()
}

fn version_row(form: &KubeForm, palette: &Palette) -> AnyElement {
    settings_row(
        "Kubernetes version",
        Some("An upgrade keeps the workloads. A downgrade resets the cluster. Images stay.".into()),
        div().w(px(260.)).child(
            Select::new(&form.version)
                .small()
                .placeholder("Loading versions…"),
        ),
        palette,
    )
    .into_any_element()
}

fn port_row(form: &KubeForm, palette: &Palette) -> AnyElement {
    settings_row(
        "Kubernetes port",
        Some("The API listens on https://127.0.0.1 at this port. Apply saves it.".into()),
        div().w(px(90.)).child(Input::new(&form.port).small()),
        palette,
    )
    .into_any_element()
}

fn traefik_row(
    model: &Entity<KubernetesModel>,
    settings: &KubernetesSettings,
    palette: &Palette,
) -> AnyElement {
    let model = model.clone();
    let current = settings.clone();
    settings_row(
        "Enable Traefik",
        Some("The ingress controller on ports 80 and 443. Turn it off to free those ports.".into()),
        Switch::new("kubernetes-traefik")
            .checked(settings.traefik)
            .on_click(move |checked, _, cx| {
                let wanted = KubernetesSettings {
                    traefik: *checked,
                    ..current.clone()
                };
                model.update(cx, |model, cx| model.save(wanted, cx));
            }),
        palette,
    )
    .into_any_element()
}

fn memory_row(
    host: &Entity<HostModel>,
    current: HostResources,
    machine: HostResources,
    palette: &Palette,
) -> AnyElement {
    let target = RECOMMENDED_MEMORY.min(machine.memory_bytes);
    let host = host.clone();
    settings_row(
        div()
            .text_color(palette.orange)
            .child("More memory recommended"),
        Some(
            format!(
                "Kubernetes needs about 2 GB of its own. Captain recommends {} for the engine.",
                bytes_label(RECOMMENDED_MEMORY)
            )
            .into(),
        ),
        text_button(
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
        ),
        palette,
    )
    .into_any_element()
}

/// Apply, enabled while the engine runs and nothing else runs. `step` is the
/// progress line of an apply or reset.
fn apply_row(
    step: Option<SharedString>,
    form: &KubeForm,
    enabled: bool,
    palette: &Palette,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    let note: SharedString = step.unwrap_or_else(|| {
        if enabled {
            "Changes apply the next time Captain Engine starts, or now with Apply.".into()
        } else {
            "Changes apply when Captain Engine starts.".into()
        }
    });
    let control = div()
        .flex()
        .flex_col()
        .items_end()
        .gap(px(4.))
        .child(text_button(
            "kubernetes-apply",
            "Apply",
            ButtonTone::Accent,
            enabled,
            palette,
            cx.listener(|view, _, _, cx| view.apply_kubernetes(cx)),
        ))
        .children(
            form.error
                .clone()
                .map(|error| inline_error(error, palette).max_w(px(320.))),
        );
    settings_row("Apply", Some(note), control, palette).into_any_element()
}

fn reset_row(model: &Entity<KubernetesModel>, enabled: bool, palette: &Palette) -> AnyElement {
    let model = model.clone();
    settings_row(
        "Reset",
        Some("Deletes the workloads and the cluster state. Images stay.".into()),
        text_button(
            "kubernetes-reset",
            "Reset Kubernetes\u{2026}",
            ButtonTone::Danger,
            enabled,
            palette,
            move |_, window, cx| kube_dialogs::reset(model.clone(), window, cx),
        ),
        palette,
    )
    .into_any_element()
}

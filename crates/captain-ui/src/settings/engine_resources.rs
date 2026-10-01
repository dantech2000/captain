//! The Resources row of the Engine section: CPU, memory, and disk steppers, and a
//! note with this computer's memory and free disk. CPUs and memory save on each
//! click. The disk stepper holds a larger size until Grow confirms it, because a
//! disk cannot shrink. See feature 0037.

use captain_core::HostResources;
use captain_core::format::bytes_label;
use gpui_kit::*;

use super::page_section::{sub_row, under_note};
use super::{SettingsView, disk_dialog};
use crate::engine_host::HostModel;
use crate::help::HelpExt;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, stepper, text_button};

/// The disk stepper's step in GiB.
const DISK_STEP: i64 = 16;

/// The stepper row and its note.
pub fn rows(
    view: &SettingsView,
    model: &Entity<HostModel>,
    host: &HostModel,
    palette: &Palette,
    cx: &Context<SettingsView>,
) -> [Div; 2] {
    let resources = host.resources();
    let step = |label: &'static str,
                id: &'static str,
                help: &'static str,
                value: String,
                step: fn(HostResources, i32, HostResources) -> HostResources| {
        let model = model.clone();
        labeled(label, id, palette)
            .child(stepper(
                id,
                label,
                value,
                palette,
                Box::new(move |delta, cx| {
                    model.update(cx, |model, cx| {
                        let changed = step(model.resources(), delta, model.machine());
                        if changed != model.resources() {
                            model.set_resources(changed, cx);
                        }
                    })
                }),
            ))
            .help(help)
    };
    let steppers = sub_row("Resources", palette)
        .child(step(
            "CPUs",
            "engine-cpus",
            "The CPUs Captain Engine may use. A running engine needs a restart to use a new value.",
            resources.cpus.to_string(),
            |r, delta, machine| r.step_cpus(delta, machine.cpus),
        ))
        .child(step(
            "Memory",
            "engine-memory",
            "The memory Captain Engine may use. A running engine needs a restart to use a new value.",
            bytes_label(resources.memory_bytes),
            |r, delta, machine| r.step_memory(i64::from(delta), machine.memory_bytes),
        ))
        .child(disk(view.disk_pending, model, resources, palette, cx))
        .children(view.disk_pending.map(|disk| grow_button(model, disk, palette, cx)));
    [
        steppers,
        under_note(note(host.machine(), view.free_disk), palette),
    ]
}

fn labeled(label: &'static str, id: &'static str, palette: &Palette) -> Stateful<Div> {
    div().id(id).flex().items_center().gap(px(8.)).child(
        div()
            .text_size(px(12.))
            .text_color(palette.text2)
            .child(label),
    )
}

/// The disk stepper. It shows `pending`, a larger size that waits for Grow, or the
/// saved size. It never goes below the engine's disk now. A smaller size than the
/// saved one saves at once, since the disk has not grown to it yet.
fn disk(
    pending: Option<u64>,
    model: &Entity<HostModel>,
    resources: HostResources,
    palette: &Palette,
    cx: &Context<SettingsView>,
) -> Stateful<Div> {
    let (model, view) = (model.clone(), cx.weak_entity());
    labeled("Disk", "engine-disk", palette)
        .child(stepper(
            "engine-disk",
            "Disk",
            bytes_label(pending.unwrap_or(resources.disk_bytes)),
            palette,
            Box::new(move |delta, cx| {
                let (saved, floor) = {
                    let host = model.read(cx);
                    (host.resources(), host.current_disk())
                };
                let Ok(next) = view.update(cx, |view, cx| {
                    let shown = HostResources {
                        disk_bytes: view.disk_pending.unwrap_or(saved.disk_bytes),
                        ..saved
                    };
                    let next = shown.step_disk(i64::from(delta) * DISK_STEP, floor);
                    view.disk_pending = (next.disk_bytes > saved.disk_bytes).then_some(next.disk_bytes);
                    cx.notify();
                    next
                }) else {
                    return;
                };
                if next.disk_bytes < saved.disk_bytes {
                    model.update(cx, |model, cx| model.set_resources(next, cx));
                }
            }),
        ))
        .help(
            "The largest size of Captain Engine's disk. It can grow but not shrink, so Grow asks first.",
        )
}

/// Grow, while the disk stepper shows a larger size than the saved one.
fn grow_button(
    model: &Entity<HostModel>,
    disk: u64,
    palette: &Palette,
    cx: &Context<SettingsView>,
) -> Stateful<Div> {
    let (model, view) = (model.clone(), cx.weak_entity());
    text_button(
        "engine-disk-grow",
        "Grow\u{2026}",
        ButtonTone::Accent,
        true,
        palette,
        move |_, window, cx| disk_dialog::open(view.clone(), model.clone(), disk, window, cx),
    )
    .help("Grow Captain Engine's disk to the size the stepper shows. It asks first.")
}

fn note(machine: HostResources, free_disk: Option<u64>) -> String {
    let computer = if cfg!(target_os = "macos") {
        "Your Mac"
    } else {
        "This computer"
    };
    let memory = bytes_label(machine.memory_bytes);
    let has = match free_disk {
        Some(free) => format!(
            "{computer} has {memory} of memory and {} free.",
            bytes_label(free)
        ),
        None => format!("{computer} has {memory} of memory."),
    };
    format!("A running engine uses new values after a restart. {has}")
}

//! The Resources row of the Engine section: CPU, memory, and disk steppers, and a
//! note with this computer's memory and free disk.

use captain_core::HostResources;
use captain_core::format::bytes_label;
use gpui_kit::*;

use super::page_section::{sub_row, under_note};
use crate::engine_host::HostModel;
use crate::help::HelpExt;
use crate::theme::Palette;
use crate::widgets::stepper;

/// The stepper row and its note. `free_disk` is the free space on the engine's
/// disk, once Captain has read it.
pub fn rows(
    model: &Entity<HostModel>,
    host: &HostModel,
    free_disk: Option<u64>,
    palette: &Palette,
) -> [Div; 2] {
    let resources = host.resources();
    let step = |label: &'static str,
                id: &'static str,
                help: &'static str,
                value: String,
                step: fn(HostResources, i32, HostResources) -> HostResources| {
        let model = model.clone();
        div()
            .id(id)
            .flex()
            .items_center()
            .gap(px(8.))
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(palette.text2)
                    .child(label),
            )
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
            "The CPUs Captain Engine may use. Changes apply the next time it starts.",
            resources.cpus.to_string(),
            |r, delta, machine| r.step_cpus(delta, machine.cpus),
        ))
        .child(step(
            "Memory",
            "engine-memory",
            "The memory Captain Engine may use. Changes apply the next time it starts.",
            bytes_label(resources.memory_bytes),
            |r, delta, machine| r.step_memory(i64::from(delta), machine.memory_bytes),
        ))
        .child(step(
            "Disk",
            "engine-disk",
            "The largest size of Captain Engine's disk. It grows as it fills and cannot shrink.",
            bytes_label(resources.disk_bytes),
            |r, delta, _| r.step_disk(i64::from(delta) * 16),
        ));
    [
        steppers,
        under_note(note(host.machine(), free_disk), palette),
    ]
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
    format!("Changes apply the next time the engine starts. {has}")
}

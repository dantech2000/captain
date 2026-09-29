//! The CPU, memory, and disk rows of the Captain Engine card.

use captain_core::HostResources;
use captain_core::format::bytes_label;
use gpui_kit::*;

use crate::engine_host::HostModel;
use crate::theme::Palette;
use crate::widgets::{settings_row, stepper};

/// One row per resource, and a note that changes apply on the next start.
pub fn rows(model: &Entity<HostModel>, host: &HostModel, palette: &Palette) -> Vec<AnyElement> {
    let resources = host.resources();
    let machine = host.machine();
    let running = host.status().is_running() || host.status().is_busy();
    let hint = if running {
        "Changes apply the next time Captain Engine starts."
    } else {
        "Changes apply when Captain Engine starts."
    };
    let row = |label: &'static str,
               note: Option<SharedString>,
               id: &'static str,
               value: String,
               step: fn(HostResources, i32, HostResources) -> HostResources| {
        let model = model.clone();
        settings_row(
            label,
            note,
            stepper(
                id,
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
            ),
            palette,
        )
        .into_any_element()
    };
    vec![
        row(
            "CPUs",
            Some(hint.into()),
            "engine-cpus",
            resources.cpus.to_string(),
            |r, delta, machine| r.step_cpus(delta, machine.cpus),
        ),
        row(
            "Memory",
            Some(format!("This computer has {}.", bytes_label(machine.memory_bytes)).into()),
            "engine-memory",
            bytes_label(resources.memory_bytes),
            |r, delta, machine| r.step_memory(i64::from(delta), machine.memory_bytes),
        ),
        row(
            "Disk",
            Some("The disk grows as it fills, up to this size. It cannot shrink.".into()),
            "engine-disk",
            bytes_label(resources.disk_bytes),
            |r, delta, _| r.step_disk(i64::from(delta) * 16),
        ),
    ]
}

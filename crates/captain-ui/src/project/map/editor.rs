use captain_core::model::RestartPolicy;
use captain_core::project_map::{Rect, Setting, memory_step};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::map_state::Draft;
use super::scale::Scale;
use crate::help::HelpExt;
use crate::project::ProjectView;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, stepper, text_button};

/// One CPU step: half a CPU, in billionths.
const HALF_CPU: u64 = 500_000_000;
/// About the editor's height, to open it above a node near the bottom of the map.
const HEIGHT: f32 = 260.;

/// The editor under a node: memory, CPUs, and restart policy, then Cancel and Stage.
/// It opens above the node when the map ends below it. `cpus` is the engine's CPU
/// count, the most a container can get.
pub fn render(
    draft: &Draft,
    (node, map_height): (Rect, f32),
    cpus: u32,
    z: Scale,
    view: &WeakEntity<ProjectView>,
    palette: &Palette,
) -> Stateful<Div> {
    let name = &draft.name;
    let memory = {
        let view = view.clone();
        stepper(
            "map-edit-memory",
            "the memory limit",
            Setting::Memory(draft.memory).value_label(),
            palette,
            Box::new(move |step, cx| {
                view.update(cx, |view, cx| {
                    if let Some(draft) = view.map.draft.as_mut() {
                        draft.memory = memory_step(draft.memory, step > 0);
                    }
                    cx.notify();
                })
                .ok();
            }),
        )
    };
    let most = u64::from(cpus.max(1)) * 2 * HALF_CPU;
    let cpu = {
        let view = view.clone();
        stepper(
            "map-edit-cpus",
            "the CPU limit",
            Setting::Cpus(draft.nano_cpus).value_label(),
            palette,
            Box::new(move |step, cx| {
                view.update(cx, |view, cx| {
                    if let Some(draft) = view.map.draft.as_mut() {
                        let next = if step > 0 {
                            (draft.nano_cpus / HALF_CPU + 1) * HALF_CPU
                        } else {
                            draft.nano_cpus.div_ceil(HALF_CPU).saturating_sub(1) * HALF_CPU
                        };
                        draft.nano_cpus = next.clamp(HALF_CPU, most);
                    }
                    cx.notify();
                })
                .ok();
            }),
        )
    };
    let policies =
        div()
            .flex()
            .flex_wrap()
            .gap(px(4.))
            .children(RestartPolicy::ALL.into_iter().map(|policy| {
                let selected = draft.restart == policy;
                let view = view.clone();
                div()
                    .id(SharedString::from(format!(
                        "map-edit-restart-{}",
                        policy.name()
                    )))
                    .h(px(24.))
                    .px(px(8.))
                    .flex()
                    .items_center()
                    .rounded(px(6.))
                    .border_1()
                    .border_color(if selected {
                        palette.accent
                    } else {
                        palette.sep
                    })
                    .when(selected, |this| this.bg(palette.tint(palette.accent)))
                    .text_size(px(11.))
                    .font_family(palette.mono())
                    .cursor_pointer()
                    .on_click(move |_, _, cx| {
                        view.update(cx, |view, cx| {
                            if let Some(draft) = view.map.draft.as_mut() {
                                draft.restart = policy;
                            }
                            cx.notify();
                        })
                        .ok();
                    })
                    .child(policy.name())
                    .help(format!(
                        "When {name} exits, the engine {}.",
                        meaning(policy)
                    ))
            }));
    let cancel = {
        let view = view.clone();
        text_button(
            "map-edit-cancel",
            "Cancel",
            ButtonTone::Accent,
            true,
            palette,
            move |_, _, cx| {
                view.update(cx, |view, cx| {
                    view.map.draft = None;
                    cx.notify();
                })
                .ok();
            },
        )
        .help("Close the editor. Nothing is staged.")
    };
    let stage = {
        let view = view.clone();
        text_button("map-edit-stage", "Stage", ButtonTone::Accent, true, palette, move |_, _, cx| {
            view.update(cx, |view, cx| view.stage_draft(cx)).ok();
        })
        .help(format!(
            "Add the changed values of {name} to the staged changes. Nothing changes until you apply."
        ))
    };
    let field = |label: &'static str, control: Div| {
        div()
            .flex()
            .flex_col()
            .gap(px(4.))
            .child(
                div()
                    .text_size(px(11.))
                    .text_color(palette.text3)
                    .child(label),
            )
            .child(control)
    };
    let below = z.px(node.y + node.h) + px(8.);
    let above = z.px(node.y) - px(HEIGHT + 8.);
    let top = if below + px(HEIGHT) > z.px(map_height) && above > px(0.) {
        above
    } else {
        below
    };
    div()
        .id("map-editor")
        .absolute()
        .left(z.px(node.x))
        .top(top)
        .w(px(280.))
        .flex()
        .flex_col()
        .gap(px(10.))
        .p(px(12.))
        .rounded(px(12.))
        .bg(palette.card)
        .border_1()
        .border_color(palette.border_strong)
        .shadow_lg()
        .text_size(px(12.))
        .on_click(|_, _, cx| cx.stop_propagation())
        .child(
            div()
                .font_weight(FontWeight::BOLD)
                .child(format!("Edit {name}")),
        )
        .child(field("Memory limit", memory))
        .child(field("CPUs", cpu))
        .child(field("Restart policy", policies))
        .child(
            div()
                .flex()
                .justify_end()
                .gap(px(6.))
                .child(cancel)
                .child(stage),
        )
        .help(format!(
            "Staged values for {name}. Docker changes them in place when you apply."
        ))
}

/// What the engine does under `policy`, for the help sentences.
fn meaning(policy: RestartPolicy) -> &'static str {
    match policy {
        RestartPolicy::No => "leaves it stopped",
        RestartPolicy::Always => "always starts it again, also after the engine starts",
        RestartPolicy::UnlessStopped => "starts it again, unless you stopped it",
        RestartPolicy::OnFailure => "starts it again only when it exits with an error",
    }
}

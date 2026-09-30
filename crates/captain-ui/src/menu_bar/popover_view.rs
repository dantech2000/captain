//! The popover under the menu bar icon: the engine, the worst problem, projects,
//! open ports, and a help line. See docs/features/0032-menu-bar-popover.md.

use std::collections::{HashMap, HashSet};

use captain_core::model::ContainerState;
use captain_core::problems::ExitFacts;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::popover::closed_itself;
use super::popover_data::PopoverData;
use super::{engine_header, footer, ports_section, projects_section, warning_card};
use crate::diagnostics::diagnostics_model;
use crate::engine_host::host_model;
use crate::help::{self, hover_help};
use crate::kubernetes::kubernetes_model;
use crate::theme::Palette;
use crate::workspace::Workspace;

pub struct PopoverView {
    workspace: Entity<Workspace>,
    open_captain: fn(&mut App),
    focus: FocusHandle,
    /// What `inspect` said about each restarting container.
    facts: HashMap<String, ExitFacts>,
    /// The containers that were restarting at the last look.
    restarting: HashSet<String>,
    /// The panel became key once. It starts inactive, so only a later loss closes it.
    was_active: bool,
    _subscriptions: Vec<Subscription>,
}

impl PopoverView {
    pub fn new(
        workspace: Entity<Workspace>,
        open_captain: fn(&mut App),
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let help = help::ensure(cx);
        let mut subscriptions = vec![
            cx.observe(&help, |_, _, cx| cx.notify()),
            cx.observe(&workspace, |this: &mut Self, workspace, cx| {
                this.follow_restarts(&workspace, cx);
                cx.notify();
            }),
            cx.observe_window_activation(window, |this: &mut Self, window, cx| {
                if window.is_window_active() {
                    this.was_active = true;
                } else if this.was_active {
                    closed_itself(window, cx);
                }
            }),
        ];
        subscriptions.extend(host_model(cx).map(|m| cx.observe(&m, |_, _, cx| cx.notify())));
        subscriptions.extend(kubernetes_model(cx).map(|m| cx.observe(&m, |_, _, cx| cx.notify())));
        subscriptions.extend(diagnostics_model(cx).map(|m| cx.observe(&m, |_, _, cx| cx.notify())));
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        let mut view = Self {
            workspace: workspace.clone(),
            open_captain,
            focus,
            facts: HashMap::new(),
            restarting: HashSet::new(),
            was_active: false,
            _subscriptions: subscriptions,
        };
        view.follow_restarts(&workspace, cx);
        view
    }

    /// Inspects each container that just started restarting, to learn if it ran out
    /// of memory. Each new restart asks again, so a raised limit shows.
    fn follow_restarts(&mut self, workspace: &Entity<Workspace>, cx: &mut Context<Self>) {
        let workspace = workspace.read(cx);
        let engine = workspace.engine();
        let now: HashSet<String> = workspace
            .store()
            .containers()
            .iter()
            .filter(|c| c.state == ContainerState::Restarting)
            .map(|c| c.id.clone())
            .collect();
        self.facts.retain(|id, _| now.contains(id));
        let fresh: Vec<String> = now.difference(&self.restarting).cloned().collect();
        self.restarting = now;
        let Some(engine) = engine else {
            return;
        };
        for id in fresh {
            let inspect = engine.inspect_container(&id);
            cx.spawn(async move |this, cx| {
                let Ok(detail) = inspect.await else {
                    return;
                };
                let facts = ExitFacts {
                    oom_killed: detail.oom_killed,
                    memory_limit: i64::try_from(detail.memory_limit).unwrap_or(i64::MAX),
                    restart_count: detail.restart_count,
                };
                this.update(cx, |this, cx| {
                    if this.restarting.contains(&id) {
                        this.facts.insert(id, facts);
                        cx.notify();
                    }
                })
                .ok();
            })
            .detach();
        }
    }
}

impl Render for PopoverView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let data = PopoverData::gather(self.workspace.read(cx), &self.facts, cx);
        let hint = hover_help(cx)
            .and_then(|help| help.read(cx).hint().map(|hint| hint.text.clone()))
            .unwrap_or_else(|| "Hover anything for help.".into());
        let has_rows = !data.projects.is_empty() || data.kubernetes.is_some();
        div()
            .track_focus(&self.focus)
            .on_key_down(|event, window, cx| {
                if event.keystroke.key == "escape" {
                    closed_itself(window, cx);
                }
            })
            .size_full()
            .flex()
            .flex_col()
            .bg(palette.card)
            .text_color(palette.text)
            .text_size(px(13.))
            .child(engine_header::render(&data.engine, &palette))
            .child(
                div()
                    .id("popover-body")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .children(
                        data.problem.as_ref().map(|problem| {
                            warning_card::render(problem, &self.workspace, &palette)
                        }),
                    )
                    .when(has_rows, |body| {
                        body.child(projects_section::render(
                            &data.projects,
                            data.kubernetes.as_ref(),
                            &self.workspace,
                            &palette,
                        ))
                    })
                    .when(!data.ports.is_empty(), |body| {
                        body.child(ports_section::render(&data.ports, &palette))
                    }),
            )
            .child(footer::render(
                data.float,
                data.active,
                &self.workspace,
                self.open_captain,
                &palette,
            ))
            .child(
                div()
                    .h(px(28.))
                    .flex_shrink_0()
                    .px(px(16.))
                    .flex()
                    .items_center()
                    .bg(palette.side)
                    .border_t_1()
                    .border_color(palette.sep)
                    .text_size(px(11.))
                    .text_color(palette.text3)
                    .child(div().truncate().child(hint)),
            )
    }
}

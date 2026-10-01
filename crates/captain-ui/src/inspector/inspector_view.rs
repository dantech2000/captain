use captain_core::model::{ContainerDetail, ContainerState};
use gpui_kit::*;

use super::files::FilesPane;
use super::logs::LogsPane;
use super::processes::ProcessList;
use super::resize_handle::{self, DraggedEdge};
use super::tabs::{self, Tab};
use super::terminal::{TerminalPane, TerminalTarget};
use super::{actions, header, overview, stats_tab};
use crate::theme::Palette;
use crate::widgets::drag_region;
use crate::workspace::Workspace;

/// The right-hand panel for the selected container.
pub struct InspectorView {
    workspace: Entity<Workspace>,
    tab: Tab,
    /// The container and state the detail and logs belong to.
    shown: Option<(String, ContainerState)>,
    detail: Option<ContainerDetail>,
    logs: Entity<LogsPane>,
    terminal: Entity<TerminalPane>,
    files: Entity<FilesPane>,
    processes: Entity<ProcessList>,
    detail_task: Option<Task<()>>,
    /// The panel's width, set by dragging its left edge.
    width: f32,
    _observe: Subscription,
}

impl InspectorView {
    pub fn new(workspace: Entity<Workspace>, cx: &mut Context<Self>) -> Self {
        let observe = cx.observe(&workspace, |this, workspace, cx| {
            this.follow_selection(cx);
            // A card's Logs or Shell button asks for a tab.
            if let Some(tab) = workspace.update(cx, |workspace, _| workspace.take_inspector_tab()) {
                this.set_tab(tab.into(), cx);
            }
            // A `logs` command in the ⌘K palette asks for filters.
            if let Some(filter) = workspace.update(cx, |workspace, _| workspace.take_log_filter()) {
                this.logs
                    .update(cx, |logs, cx| logs.filter(filter.since, filter.errors, cx));
            }
            cx.notify();
        });
        let mut view = Self {
            workspace,
            tab: Tab::default(),
            shown: None,
            detail: None,
            logs: cx.new(|_| LogsPane::default()),
            terminal: cx.new(TerminalPane::new),
            files: cx.new(FilesPane::new),
            processes: cx.new(|_| ProcessList::new()),
            detail_task: None,
            width: resize_handle::DEFAULT_WIDTH,
            _observe: observe,
        };
        view.follow_selection(cx);
        view
    }

    /// Reloads detail and logs when the selection, or the selected container's state, changes.
    fn follow_selection(&mut self, cx: &mut Context<Self>) {
        let workspace = self.workspace.read(cx);
        let current = workspace.selected().map(|c| (c.id.clone(), c.state));
        if current == self.shown {
            return;
        }
        let target = workspace.selected().map(|c| TerminalTarget {
            id: c.id.clone(),
            name: c.name.clone(),
            running: c.state == ContainerState::Running,
        });
        let engine = workspace.engine();
        let show_terminal = self.tab == Tab::Terminal;
        self.terminal.update(cx, |terminal, cx| {
            terminal.set_target(target, engine.clone(), cx);
            if show_terminal {
                terminal.show(cx);
            }
        });
        let target = current
            .clone()
            .map(|(id, state)| (id, state == ContainerState::Running));
        let show_files = self.tab == Tab::Files;
        self.files.update(cx, |files, cx| {
            files.set_target(target.clone(), engine.clone(), cx);
            if show_files {
                files.show(cx);
            }
        });
        self.processes
            .update(cx, |processes, cx| processes.set_target(target, engine, cx));
        let workspace = self.workspace.read(cx);
        let same_container = current.as_ref().map(|c| &c.0) == self.shown.as_ref().map(|s| &s.0);
        self.shown = current.clone();
        let (Some((id, _)), Some(engine)) = (current, workspace.engine()) else {
            return;
        };
        if !same_container {
            self.detail = None;
        }

        let inspect = engine.inspect_container(&id);
        self.detail_task = Some(cx.spawn(async move |this, cx| {
            if let Ok(detail) = inspect.await {
                this.update(cx, |this, cx| {
                    this.detail = Some(detail);
                    cx.notify();
                })
                .ok();
            }
        }));

        self.logs.update(cx, |logs, cx| logs.load(engine, id, cx));
    }
}

impl InspectorView {
    fn set_tab(&mut self, tab: Tab, cx: &mut Context<Self>) {
        self.tab = tab;
        if tab == Tab::Terminal {
            self.terminal.update(cx, |terminal, cx| terminal.show(cx));
        }
        if tab == Tab::Files {
            self.files.update(cx, |files, cx| files.show(cx));
        }
        self.processes.update(cx, |processes, cx| {
            processes.set_active(tab == Tab::Stats, cx)
        });
        cx.notify();
    }
}

impl Render for InspectorView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let Some(container) = self.workspace.read(cx).selected().cloned() else {
            return div();
        };
        let entity = cx.entity().downgrade();
        let on_tab = move |tab: Tab, _: &mut Window, cx: &mut App| {
            entity.update(cx, |this, cx| this.set_tab(tab, cx)).ok();
        };

        let body = match self.tab {
            Tab::Logs => self.logs.clone().into_any_element(),
            Tab::Overview => {
                let workspace = self.workspace.read(cx);
                overview::render(&container, self.detail.as_ref(), workspace, &palette)
                    .into_any_element()
            }
            Tab::Stats => {
                let history = self.workspace.read(cx).stats().get(&container.id);
                div()
                    .id("stats-tab")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .pb(px(20.))
                    .child(stats_tab::render(history, &palette))
                    .child(self.processes.clone())
                    .into_any_element()
            }
            Tab::Terminal => self.terminal.clone().into_any_element(),
            Tab::Files => self.files.clone().into_any_element(),
        };
        let workspace = self.workspace.read(cx);

        let reset = cx.entity().downgrade();
        div()
            .w(px(self.width))
            .relative()
            .on_drag_move(
                cx.listener(|this, event: &DragMoveEvent<DraggedEdge>, window, cx| {
                    let beside = crate::shell::left_width(this.workspace.read(cx).sidebar_hidden());
                    let room = f32::from(window.viewport_size().width) - beside;
                    this.width = resize_handle::width_for(
                        event.bounds.right(),
                        event.event.position.x,
                        room,
                    );
                    cx.notify();
                }),
            )
            .child(resize_handle::render(&palette, move |_, cx| {
                reset
                    .update(cx, |this, cx| {
                        this.width = resize_handle::DEFAULT_WIDTH;
                        cx.notify();
                    })
                    .ok();
            }))
            .h_full()
            .flex_shrink_0()
            .flex()
            .flex_col()
            .bg(palette.panel)
            .border_l_1()
            .border_color(palette.sep)
            .child(
                drag_region("inspector-header")
                    .flex()
                    .flex_col()
                    .gap(px(16.))
                    .px(px(20.))
                    .pt(px(20.))
                    .child(header::render(&container, &self.workspace, &palette))
                    .child(actions::render(
                        &container,
                        &self.workspace,
                        workspace,
                        &palette,
                    ))
                    .child(tabs::render(self.tab, &palette, on_tab)),
            )
            .child(div().flex_1().min_h_0().flex().flex_col().child(body))
    }
}

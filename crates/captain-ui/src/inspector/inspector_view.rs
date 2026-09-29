use captain_core::model::{ContainerDetail, ContainerState};
use captain_core::store::{LevelFilter, LogBuffer};
use futures::StreamExt;
use gpui_kit::assets::IconName;
use gpui_kit::*;

use super::tabs::{self, Tab};
use super::{actions, header, logs_tab, overview, placeholder, stats_tab};
use crate::theme::Palette;
use crate::widgets::drag_region;
use crate::workspace::Workspace;

/// How many past log lines to load when a container is selected.
const LOG_TAIL: usize = 500;

/// The right-hand panel for the selected container.
pub struct InspectorView {
    workspace: Entity<Workspace>,
    tab: Tab,
    level: LevelFilter,
    /// The container and state the detail and logs belong to.
    shown: Option<(String, ContainerState)>,
    detail: Option<ContainerDetail>,
    logs: LogBuffer,
    log_scroll: UniformListScrollHandle,
    detail_task: Option<Task<()>>,
    logs_task: Option<Task<()>>,
    _observe: Subscription,
}

impl InspectorView {
    pub fn new(workspace: Entity<Workspace>, cx: &mut Context<Self>) -> Self {
        let observe = cx.observe(&workspace, |this, _, cx| {
            this.follow_selection(cx);
            cx.notify();
        });
        let mut view = Self {
            workspace,
            tab: Tab::default(),
            level: LevelFilter::default(),
            shown: None,
            detail: None,
            logs: LogBuffer::default(),
            log_scroll: UniformListScrollHandle::new(),
            detail_task: None,
            logs_task: None,
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
        let same_container = current.as_ref().map(|c| &c.0) == self.shown.as_ref().map(|s| &s.0);
        self.shown = current.clone();
        let (Some((id, _)), Some(engine)) = (current, workspace.engine()) else {
            return;
        };
        if !same_container {
            self.detail = None;
        }
        self.logs.clear();

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

        let mut lines = engine.logs(&id, LOG_TAIL);
        self.logs_task = Some(cx.spawn(async move |this, cx| {
            while let Some(Ok(line)) = lines.next().await {
                let pushed = this.update(cx, |this, cx| {
                    this.logs.push(line);
                    let last = this.logs.filtered(this.level).len().saturating_sub(1);
                    this.log_scroll.scroll_to_item(last, ScrollStrategy::Bottom);
                    cx.notify();
                });
                if pushed.is_err() {
                    break;
                }
            }
        }));
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
            entity
                .update(cx, |this, cx| {
                    this.tab = tab;
                    cx.notify();
                })
                .ok();
        };

        let body = match self.tab {
            Tab::Logs => logs_tab::render(self, &palette, cx).into_any_element(),
            Tab::Overview => {
                let workspace = self.workspace.read(cx);
                overview::render(&container, self.detail.as_ref(), workspace, &palette)
                    .into_any_element()
            }
            Tab::Stats => {
                let history = self.workspace.read(cx).stats().get(&container.id);
                stats_tab::render(history, &palette).into_any_element()
            }
            Tab::Terminal => placeholder::render(
                IconName::SquareTerminal,
                "Terminal is coming in M8",
                "An interactive shell in the container, built on libghostty-vt.",
                &palette,
            )
            .into_any_element(),
            Tab::Files => placeholder::render(
                IconName::Folder,
                "Files are coming later",
                "Browse and copy files inside the container.",
                &palette,
            )
            .into_any_element(),
        };
        let workspace = self.workspace.read(cx);

        div()
            .w(px(400.))
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
                    .child(header::render(&container, &palette))
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

impl InspectorView {
    pub(super) fn logs(&self) -> &LogBuffer {
        &self.logs
    }

    pub(super) fn level(&self) -> LevelFilter {
        self.level
    }

    pub(super) fn log_scroll(&self) -> &UniformListScrollHandle {
        &self.log_scroll
    }

    pub(super) fn set_level(&mut self, level: LevelFilter, cx: &mut Context<Self>) {
        self.level = level;
        let last = self.logs.filtered(level).len().saturating_sub(1);
        self.log_scroll.scroll_to_item(last, ScrollStrategy::Bottom);
        cx.notify();
    }
}

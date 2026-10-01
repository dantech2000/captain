use std::path::PathBuf;
use std::rc::Rc;

use gpui_kit::*;

use super::keys::{CloseTerminalTab, NewTerminalTab, PANEL_CONTEXT};
use super::new_tab::{default_dir, local_source};
use super::resize_edge::{self, DEFAULT_HEIGHT, DraggedPanelEdge};
use super::tab_strip;
use crate::project::ProjectView;
use crate::terminal::{CloseRequested, EndedBar, TerminalView};
use crate::theme::Palette;
use crate::workspace::Workspace;

/// One tab: a shell and the folder it started in.
pub(super) struct TerminalTab {
    pub view: Entity<TerminalView>,
    pub cwd: PathBuf,
    _subscriptions: [Subscription; 2],
}

/// The panel under the page: tabs of shells on this computer. Hiding it keeps the
/// tabs and their shells. See docs/features/0041-integrated-terminal.md.
pub struct TerminalPanel {
    workspace: Entity<Workspace>,
    project: Entity<ProjectView>,
    pub(super) tabs: Vec<TerminalTab>,
    pub(super) active: usize,
    /// The workspace's open state at the last look, to focus the panel when it opens.
    was_open: bool,
    _subscriptions: Vec<Subscription>,
}

impl TerminalPanel {
    pub fn new(
        workspace: Entity<Workspace>,
        project: Entity<ProjectView>,
        cx: &mut Context<Self>,
    ) -> Self {
        let observe = cx.observe(&workspace, |this, workspace, cx| {
            if let Some(dir) =
                workspace.update(cx, |workspace, _| workspace.take_terminal_request())
            {
                this.new_tab(dir, cx);
                this.focus_active(cx);
            }
            let open = workspace.read(cx).terminal_open();
            if open && !this.was_open {
                if this.tabs.is_empty() {
                    this.new_tab(this.default_dir(cx), cx);
                }
                this.focus_active(cx);
            }
            this.was_open = open;
        });
        // Quit hangs up every shell, so none outlives Captain.
        let quit = cx.on_app_quit(|this, cx| {
            for tab in &this.tabs {
                tab.view.update(cx, |view, _| view.end());
            }
            async {}
        });
        Self {
            workspace,
            project,
            tabs: Vec::new(),
            active: 0,
            was_open: false,
            _subscriptions: vec![observe, quit],
        }
    }

    /// The folder for a new tab: the shown project's, else home.
    pub(super) fn default_dir(&self, cx: &App) -> PathBuf {
        default_dir(self.workspace.read(cx), &self.project, cx)
    }

    /// Opens a tab in `dir` and makes it the active one.
    pub(super) fn new_tab(&mut self, dir: PathBuf, cx: &mut Context<Self>) {
        let source = Rc::new(local_source(&dir, self.workspace.read(cx), cx));
        let view = cx.new(|cx| {
            let mut view = TerminalView::new(EndedBar::Buttons, cx);
            view.set_source(Some(source), cx);
            view.show(cx);
            view
        });
        let subscriptions = [
            cx.observe(&view, |_, _, cx| cx.notify()),
            cx.subscribe(&view, |this, view, _: &CloseRequested, cx| {
                if let Some(index) = this.tabs.iter().position(|tab| tab.view == view) {
                    this.close_tab(index, cx);
                }
            }),
        ];
        self.tabs.push(TerminalTab {
            view,
            cwd: dir,
            _subscriptions: subscriptions,
        });
        self.active = self.tabs.len() - 1;
        cx.notify();
    }

    /// Closes the tab at `index` and hangs up its shell. The last tab hides the panel.
    pub(super) fn close_tab(&mut self, index: usize, cx: &mut Context<Self>) {
        if index >= self.tabs.len() {
            return;
        }
        let tab = self.tabs.remove(index);
        tab.view.update(cx, |view, _| view.end());
        if self.tabs.is_empty() {
            self.active = 0;
            self.workspace
                .update(cx, |workspace, cx| workspace.set_terminal_open(false, cx));
        } else {
            if self.active >= index && self.active > 0 {
                self.active -= 1;
            }
            self.focus_active(cx);
        }
        cx.notify();
    }

    pub(super) fn select(&mut self, index: usize, cx: &mut Context<Self>) {
        self.active = index.min(self.tabs.len().saturating_sub(1));
        self.focus_active(cx);
        cx.notify();
    }

    /// Hides the panel. The tabs keep running.
    pub(super) fn hide(&mut self, cx: &mut Context<Self>) {
        self.workspace
            .update(cx, |workspace, cx| workspace.set_terminal_open(false, cx));
    }

    /// Moves focus to the active tab's grid.
    fn focus_active(&self, cx: &mut Context<Self>) {
        if let Some(tab) = self.tabs.get(self.active) {
            tab.view.update(cx, |view, cx| view.focus_next_render(cx));
        }
    }
}

impl Render for TerminalPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        let room = f32::from(window.viewport_size().height);
        let height = self
            .workspace
            .read(cx)
            .terminal_height()
            .min(resize_edge::max_height(room));
        let reset = self.workspace.downgrade();
        let body = self.tabs.get(self.active).map(|tab| tab.view.clone());

        div()
            .id("terminal-panel")
            .key_context(PANEL_CONTEXT)
            .relative()
            .flex_shrink_0()
            .h(px(height))
            .flex()
            .flex_col()
            .bg(palette.panel)
            .border_t_1()
            .border_color(palette.sep)
            .on_action(cx.listener(|this, _: &NewTerminalTab, _, cx| {
                let dir = this.default_dir(cx);
                this.new_tab(dir, cx);
                this.focus_active(cx);
            }))
            .on_action(cx.listener(|this, _: &CloseTerminalTab, _, cx| {
                this.close_tab(this.active, cx);
            }))
            .on_drag_move(cx.listener(
                move |this, event: &DragMoveEvent<DraggedPanelEdge>, window, cx| {
                    let room = f32::from(window.viewport_size().height);
                    let height = resize_edge::height_for(
                        event.bounds.bottom(),
                        event.event.position.y,
                        room,
                    );
                    this.workspace.update(cx, |workspace, cx| {
                        workspace.set_terminal_height(height, cx)
                    });
                },
            ))
            .child(resize_edge::render(&palette, move |_, cx| {
                reset
                    .update(cx, |workspace, cx| {
                        workspace.set_terminal_height(DEFAULT_HEIGHT, cx)
                    })
                    .ok();
            }))
            .child(tab_strip::render(self, &palette, cx))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .px(px(12.))
                    .pb(px(10.))
                    .children(body),
            )
    }
}

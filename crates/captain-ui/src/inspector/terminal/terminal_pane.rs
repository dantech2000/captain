use std::sync::Arc;

use captain_core::Engine;
use captain_core::model::{ExecInput, ExecResizer};
use captain_terminal::{Emulator, Screen, default_emulator};
use gpui_kit::*;

use super::colors::TerminalColors;
use super::grid::TerminalGrid;
use super::metrics::GridMetrics;
use super::{header, keys, status};
use crate::theme::Palette;

/// The grid size before the first layout.
pub(super) const DEFAULT_SIZE: (u16, u16) = (80, 24);

/// The container a terminal belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalTarget {
    pub id: String,
    pub name: String,
    pub running: bool,
}

/// Where the shell session is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Phase {
    /// No session yet. The next show of the tab starts one.
    Idle,
    Connecting,
    Running,
    /// The shell exited, with its code if the engine reported one.
    Exited(Option<i64>),
    /// The engine refused to start the shell.
    Failed(String),
}

/// The live half of a session: where input goes and how to resize it.
pub(super) struct Live {
    pub input: ExecInput,
    pub resizer: ExecResizer,
}

/// The Terminal tab: one shell in the selected container. It owns the emulator and the
/// exec session, and starts the session only when the tab is shown.
pub struct TerminalPane {
    pub(super) target: Option<TerminalTarget>,
    pub(super) engine: Option<Arc<dyn Engine>>,
    pub(super) phase: Phase,
    pub(super) emulator: Box<dyn Emulator>,
    pub(super) live: Option<Live>,
    /// The command the session runs, for example `/bin/bash`.
    pub(super) command: Option<String>,
    pub(super) focus: FocusHandle,
    /// The last layout of the grid, for mouse hit tests.
    pub(super) metrics: Option<GridMetrics>,
    /// Scroll wheel pixels not yet turned into whole lines.
    pub(super) scroll_rest: Pixels,
    /// Move focus to the grid on the next render.
    pub(super) wants_focus: bool,
    pub(super) tasks: Vec<Task<()>>,
    pub(super) resize_task: Option<Task<()>>,
}

impl TerminalPane {
    pub fn new(cx: &mut Context<Self>) -> Self {
        keys::bind(cx);
        let (cols, rows) = DEFAULT_SIZE;
        Self {
            target: None,
            engine: None,
            phase: Phase::Idle,
            emulator: default_emulator(cols, rows),
            live: None,
            command: None,
            focus: cx.focus_handle(),
            metrics: None,
            scroll_rest: px(0.),
            wants_focus: false,
            tasks: Vec::new(),
            resize_task: None,
        }
    }

    /// Follows the inspector's container. A different container, or a stop, ends the
    /// session; the tab starts a new one when it is shown.
    pub fn set_target(
        &mut self,
        target: Option<TerminalTarget>,
        engine: Option<Arc<dyn Engine>>,
        cx: &mut Context<Self>,
    ) {
        let same = target.as_ref().map(|t| &t.id) == self.target.as_ref().map(|t| &t.id);
        let stopped = target.as_ref().is_some_and(|t| !t.running);
        if !same || stopped {
            self.end();
        }
        self.target = target;
        self.engine = engine;
        cx.notify();
    }

    /// The tab is showing: start a session if there is none.
    pub fn show(&mut self, cx: &mut Context<Self>) {
        if self.phase == Phase::Idle {
            self.start(cx);
        }
    }

    /// Ends the old session and starts a new one.
    pub(super) fn reconnect(&mut self, cx: &mut Context<Self>) {
        self.end();
        self.start(cx);
    }

    /// Drops the session, which closes the connection and ends the shell's input.
    pub(super) fn end(&mut self) {
        self.tasks.clear();
        self.resize_task = None;
        self.live = None;
        self.command = None;
        self.phase = Phase::Idle;
    }

    pub(super) fn snapshot(&self) -> Screen {
        self.emulator.snapshot()
    }

    pub(super) fn running(&self) -> bool {
        self.target.as_ref().is_some_and(|t| t.running)
    }
}

impl Render for TerminalPane {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        if self.wants_focus {
            self.wants_focus = false;
            window.focus(&self.focus, cx);
        }
        let body = if self.running() {
            let focused = self.focus.is_focused(window);
            let grid = TerminalGrid::new(
                cx.entity(),
                TerminalColors::new(&palette),
                palette.mono(),
                focused,
            );
            self.grid_box(grid, &palette, cx).into_any_element()
        } else {
            status::not_running(&palette).into_any_element()
        };

        div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .gap(px(10.))
            .px(px(20.))
            .pt(px(14.))
            .pb(px(20.))
            .child(header::render(self, &palette, cx))
            .child(body)
    }
}

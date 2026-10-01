use std::rc::Rc;

use captain_core::model::{ExecInput, ExecResizer};
use captain_terminal::{Emulator, Screen, default_emulator};
use gpui_kit::*;

use super::colors::TerminalColors;
use super::grid::TerminalGrid;
use super::metrics::GridMetrics;
use super::{TerminalSource, keys};
use crate::theme::Palette;

/// The grid size before the first layout.
pub(super) const DEFAULT_SIZE: (u16, u16) = (80, 24);

/// Where the session is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase {
    /// No session yet. The next show starts one.
    Idle,
    Connecting,
    Running,
    /// The shell exited, with its code if the source reported one.
    Exited(Option<i64>),
    /// The source refused to start the shell.
    Failed(String),
}

/// What the view shows over the grid once the shell has ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndedBar {
    /// A note that points at the Reconnect button of the inspector's header.
    Note,
    /// Restart and Close tab buttons, for the terminal panel.
    Buttons,
}

/// The view asks its owner to close it: the Close tab button of [`EndedBar::Buttons`].
pub struct CloseRequested;

/// The live half of a session: where input goes and how to resize it.
pub(super) struct Live {
    pub input: ExecInput,
    pub resizer: ExecResizer,
}

/// One terminal: the emulator, the grid, and a session from a [`TerminalSource`].
/// The container exec in the inspector and the tabs of the terminal panel share it.
pub struct TerminalView {
    pub(super) source: Option<Rc<dyn TerminalSource>>,
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
    pub(super) ended_bar: EndedBar,
    pub(super) tasks: Vec<Task<()>>,
    pub(super) resize_task: Option<Task<()>>,
}

impl EventEmitter<CloseRequested> for TerminalView {}

impl TerminalView {
    pub fn new(ended_bar: EndedBar, cx: &mut Context<Self>) -> Self {
        keys::bind(cx);
        let (cols, rows) = DEFAULT_SIZE;
        Self {
            source: None,
            phase: Phase::Idle,
            emulator: default_emulator(cols, rows),
            live: None,
            command: None,
            focus: cx.focus_handle(),
            metrics: None,
            scroll_rest: px(0.),
            wants_focus: false,
            ended_bar,
            tasks: Vec::new(),
            resize_task: None,
        }
    }

    /// Ends the session and uses `source` for the next one.
    pub fn set_source(&mut self, source: Option<Rc<dyn TerminalSource>>, cx: &mut Context<Self>) {
        self.end();
        self.source = source;
        cx.notify();
    }

    /// The view is showing: start a session if there is none.
    pub fn show(&mut self, cx: &mut Context<Self>) {
        if self.phase == Phase::Idle {
            self.start(cx);
        }
    }

    /// Ends the old session and starts a new one.
    pub fn restart(&mut self, cx: &mut Context<Self>) {
        self.end();
        self.start(cx);
    }

    /// Drops the session. A local shell gets a hangup at once; an exec's
    /// connection closes, which ends the shell's input.
    pub fn end(&mut self) {
        if self.live.is_some()
            && let Some(source) = &self.source
        {
            source.close();
        }
        self.tasks.clear();
        self.resize_task = None;
        self.live = None;
        self.command = None;
        self.phase = Phase::Idle;
    }

    /// Moves focus to the grid on the next render.
    pub fn focus_next_render(&mut self, cx: &mut Context<Self>) {
        self.wants_focus = true;
        cx.notify();
    }

    pub fn phase(&self) -> &Phase {
        &self.phase
    }

    /// The title the program set, if any.
    pub fn title(&self) -> Option<String> {
        self.emulator.title().filter(|title| !title.is_empty())
    }

    pub fn command(&self) -> Option<&str> {
        self.command.as_deref()
    }

    /// The grid size in cells.
    pub fn size(&self) -> (u16, u16) {
        self.emulator.size()
    }

    pub(super) fn snapshot(&self) -> Screen {
        self.emulator.snapshot()
    }
}

impl Render for TerminalView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::of(cx);
        if self.wants_focus {
            self.wants_focus = false;
            window.focus(&self.focus, cx);
        }
        let focused = self.focus.is_focused(window);
        let grid = TerminalGrid::new(
            cx.entity(),
            TerminalColors::new(&palette),
            palette.mono(),
            focused,
        );
        self.grid_box(grid, &palette, cx)
    }
}

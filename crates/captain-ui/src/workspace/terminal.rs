//! The terminal panel's state: open or hidden, its height, and a folder that a
//! button asked a new tab for. See docs/features/0041-integrated-terminal.md.

use std::path::PathBuf;

use gpui_kit::*;

use super::Workspace;
use crate::terminal_panel::DEFAULT_HEIGHT;

/// Where the terminal panel stands. Its tabs live in the panel and stay while it
/// is hidden.
#[derive(Debug, Clone)]
pub struct TerminalPanelState {
    open: bool,
    height: f32,
    /// A folder for a new tab, until the panel takes it.
    request: Option<PathBuf>,
}

impl Default for TerminalPanelState {
    fn default() -> Self {
        Self {
            open: false,
            height: DEFAULT_HEIGHT,
            request: None,
        }
    }
}

impl Workspace {
    /// True while the terminal panel shows.
    pub fn terminal_open(&self) -> bool {
        self.terminal.open
    }

    /// Shows the terminal panel, or hides it, as ⌃` does.
    pub fn toggle_terminal(&mut self, cx: &mut Context<Self>) {
        self.terminal.open = !self.terminal.open;
        cx.notify();
    }

    pub fn set_terminal_open(&mut self, open: bool, cx: &mut Context<Self>) {
        self.terminal.open = open;
        cx.notify();
    }

    /// The panel's height, set by dragging its top edge.
    pub fn terminal_height(&self) -> f32 {
        self.terminal.height
    }

    pub fn set_terminal_height(&mut self, height: f32, cx: &mut Context<Self>) {
        self.terminal.height = height;
        cx.notify();
    }

    /// Shows the panel with a new tab in `dir`, as the Project page's Terminal
    /// button does.
    pub fn open_terminal_in(&mut self, dir: PathBuf, cx: &mut Context<Self>) {
        self.terminal.request = Some(dir);
        self.terminal.open = true;
        cx.notify();
    }

    /// The folder a button asked a new tab for, once. It does not notify.
    pub fn take_terminal_request(&mut self) -> Option<PathBuf> {
        self.terminal.request.take()
    }
}

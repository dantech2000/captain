//! The shell's handlers for the File menu: New Project…, Open Folder…, and New
//! Terminal Tab. See docs/features/0042-app-menus.md.

use gpui_kit::*;

use super::AppShell;
use crate::new_project::{self, NewProject, OpenProjectFolder};
use crate::terminal_panel::{NewTerminalTab, default_dir};

impl AppShell {
    /// Adds the File menu's handlers to `root`. The terminal panel handles New
    /// Terminal Tab first while it has focus; this one serves the rest of the window.
    pub(super) fn on_menu_actions(&self, root: Div, cx: &Context<Self>) -> Div {
        root.on_action(cx.listener(|this, _: &NewProject, window, cx| {
            new_project::open(this.workspace.clone(), this.project.clone(), window, cx);
        }))
        .on_action(cx.listener(|this, _: &OpenProjectFolder, window, cx| {
            new_project::open_folder(this.workspace.clone(), this.project.clone(), window, cx);
        }))
        .on_action(cx.listener(|this, _: &NewTerminalTab, _, cx| {
            let dir = default_dir(this.workspace.read(cx), &this.project, cx);
            this.workspace
                .update(cx, |workspace, cx| workspace.open_terminal_in(dir, cx));
        }))
    }
}

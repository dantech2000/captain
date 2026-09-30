use std::cell::Cell;
use std::rc::Rc;

use captain_core::problems;
use gpui_kit::*;

use crate::diagnostics::{diagnostics_model, failures};
use crate::workspace::{Connection, Workspace};

/// Calls `show` with the number of problems now and whenever it changes: failed
/// diagnostics checks, and containers that are restarting or unhealthy. The Dock
/// badge shows it.
pub fn observe_problem_count(
    workspace: &Entity<Workspace>,
    show: impl Fn(usize, &mut App) + 'static,
    cx: &mut App,
) {
    let shown = Rc::new(Cell::new(None));
    let show = Rc::new(show);
    let update: Rc<dyn Fn(&mut App)> = {
        let workspace = workspace.downgrade();
        Rc::new(move |cx: &mut App| {
            let Some(workspace) = workspace.upgrade() else {
                return;
            };
            let count = count(workspace.read(cx), cx);
            if shown.replace(Some(count)) != Some(count) {
                show(count, cx);
            }
        })
    };
    let on_workspace = update.clone();
    cx.observe(workspace, move |_, cx| on_workspace(cx))
        .detach();
    if let Some(model) = diagnostics_model(cx) {
        let on_checks = update.clone();
        cx.observe(&model, move |_, cx| on_checks(cx)).detach();
    }
    update(cx);
}

fn count(workspace: &Workspace, cx: &App) -> usize {
    let containers: &[_] = match workspace.connection() {
        Connection::Connected(_) => workspace.store().containers(),
        _ => &[],
    };
    problems::problem_count(failures(cx), containers)
}

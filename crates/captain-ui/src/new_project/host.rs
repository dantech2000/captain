use gpui_kit::*;

use crate::project::ProjectView;
use crate::workspace::Workspace;

/// The window's workspace and Project page. The New sheet records a new project
/// in the one and shows its files in the other.
#[derive(Clone)]
pub struct SheetHost {
    pub workspace: Entity<Workspace>,
    pub project: Entity<ProjectView>,
}

struct HostGlobal {
    workspace: WeakEntity<Workspace>,
    project: WeakEntity<ProjectView>,
}

impl Global for HostGlobal {}

/// Remembers the window's workspace and Project page, for views that open the
/// New sheet without them, such as the Images page's Run.
pub fn set_host(workspace: &Entity<Workspace>, project: &Entity<ProjectView>, cx: &mut App) {
    cx.set_global(HostGlobal {
        workspace: workspace.downgrade(),
        project: project.downgrade(),
    });
}

/// The host set by [`set_host`], while its window is open.
pub fn host(cx: &App) -> Option<SheetHost> {
    let global = cx.try_global::<HostGlobal>()?;
    Some(SheetHost {
        workspace: global.workspace.upgrade()?,
        project: global.project.upgrade()?,
    })
}

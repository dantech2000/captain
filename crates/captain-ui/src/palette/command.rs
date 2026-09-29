use captain_core::model::{ContainerAction, ProjectAction};
use captain_core::store::ContainerFilter;
use gpui_kit::assets::IconName;
use gpui_kit::*;

use crate::containers::down_dialog;
use crate::migration::OpenMigrationAssistant;
use crate::workspace::{Page, Workspace};

/// The group a command is listed under.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Navigate,
    Actions,
    Containers,
}

impl Section {
    pub fn label(self) -> &'static str {
        match self {
            Section::Navigate => "Navigate",
            Section::Actions => "Actions",
            Section::Containers => "Containers",
        }
    }
}

/// What a command does when it runs.
#[derive(Debug, Clone, PartialEq)]
pub enum CommandKind {
    GoTo(Page),
    SetFilter(ContainerFilter),
    Run {
        id: String,
        action: ContainerAction,
    },
    /// Runs a `docker compose` command on a project. Down asks first.
    RunProject {
        project: String,
        action: ProjectAction,
    },
    /// Selects a container and shows it on the Containers page.
    Show(String),
    /// Opens a published port on localhost in the browser.
    OpenPort(u16),
    /// Opens the Migration Assistant.
    BringData,
}

impl CommandKind {
    pub fn run(&self, workspace: &Entity<Workspace>, cx: &mut App) {
        match self.clone() {
            CommandKind::GoTo(page) => workspace.update(cx, |w, cx| w.set_page(page, cx)),
            CommandKind::SetFilter(filter) => workspace.update(cx, |w, cx| {
                w.set_filter(filter, cx);
                w.set_page(Page::Containers, cx);
            }),
            CommandKind::Run { id, action } => {
                workspace.update(cx, |w, cx| w.run_action(id, action, cx))
            }
            CommandKind::RunProject { project, action } => {
                run_project(project, action, workspace, cx)
            }
            CommandKind::Show(id) => workspace.update(cx, |w, cx| {
                // Clear a filter that would hide the container.
                let container = w.store().find(&id);
                let hidden = container.is_some_and(|c| !w.filter().matches(c));
                let other_project = w.project_filter().is_some()
                    && container
                        .is_some_and(|c| c.compose_project.as_deref() != w.project_filter());
                if hidden {
                    w.set_filter(ContainerFilter::All, cx);
                }
                if other_project {
                    w.clear_project_filter(cx);
                }
                w.select(id, cx);
                w.set_page(Page::Containers, cx);
            }),
            CommandKind::OpenPort(port) => cx.open_url(&format!("http://localhost:{port}")),
            CommandKind::BringData => open_migration(cx),
        }
    }
}

/// Runs a project action. Down opens the confirmation in the active window first.
fn run_project(
    project: String,
    action: ProjectAction,
    workspace: &Entity<Workspace>,
    cx: &mut App,
) {
    if action != ProjectAction::Down {
        workspace.update(cx, |w, cx| w.run_project_action(project, action, cx));
        return;
    }
    let Some(window) = cx.active_window() else {
        return;
    };
    let workspace = workspace.clone();
    // The palette runs commands while it handles a key or click in this window, and
    // the window cannot be updated again until that ends.
    cx.defer(move |cx| {
        window
            .update(cx, |_, window, cx| {
                down_dialog::open(project, workspace, window, cx);
            })
            .ok();
    });
}

/// Dispatches [`OpenMigrationAssistant`] in the active window, once the palette's
/// own event has finished.
fn open_migration(cx: &mut App) {
    let Some(window) = cx.active_window() else {
        return;
    };
    cx.defer(move |cx| {
        window
            .update(cx, |_, window, cx| {
                window.dispatch_action(Box::new(OpenMigrationAssistant), cx);
            })
            .ok();
    });
}

/// One row in the palette.
#[derive(Debug, Clone)]
pub struct Command {
    pub section: Section,
    /// The text the query matches.
    pub title: String,
    /// A secondary line, such as the project and state of a container.
    pub meta: String,
    pub icon: IconName,
    /// The color of the icon and its tile.
    pub color: Hsla,
    /// True if the palette lists this command before the user types.
    pub suggested: bool,
    pub kind: CommandKind,
}

use std::path::PathBuf;

use captain_core::model::{ComposeProject, ProjectAction};
use captain_core::store::GroupKey;
use gpui_kit::component::Sizable;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::action_help::{down_help, restart_help, up_help};
use crate::containers::down_dialog;
use crate::help::HelpExt;
use crate::theme::Palette;
use crate::widgets::{drag_region, primary_button};
use crate::workspace::Workspace;

/// The folder and Compose files in mono, the name in large type, and for a Compose
/// project: Open folder, Terminal, Down, and Restart project (Up while nothing runs).
/// `active` and `count` are the containers that run and all of them. `tabs` goes
/// between the name and the buttons.
pub fn render(
    (key, project): (&GroupKey, Option<&ComposeProject>),
    (active, count): (usize, usize),
    handle: &Entity<Workspace>,
    workspace: &Workspace,
    tabs: Div,
    palette: &Palette,
) -> Stateful<Div> {
    let (path, name) = match (key, project) {
        (GroupKey::Project(name), Some(project)) => (location(project), name.clone()),
        (GroupKey::Project(name), None) => ("Compose project".to_string(), name.clone()),
        (GroupKey::Namespace(namespace), _) => ("Kubernetes namespace".into(), namespace.clone()),
        (GroupKey::Standalone, _) => ("Not in a Compose project".into(), "Loose containers".into()),
    };
    let buttons = project.map(|project| match workspace.project_pending(&project.name) {
        Some(action) => div()
            .flex()
            .items_center()
            .gap(px(6.))
            .text_size(px(12.))
            .text_color(palette.text2)
            .child(Spinner::new().xsmall().color(palette.text2))
            .child(action.progress_label()),
        None => buttons(project, (active, count), handle, workspace, palette),
    });
    drag_region("project-header")
        .flex_shrink_0()
        .flex()
        .items_end()
        .gap(px(12.))
        .px(px(28.))
        .pt(px(22.))
        .pb(px(16.))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(4.))
                .child(
                    div()
                        .text_size(px(11.))
                        .font_family(palette.mono())
                        .text_color(palette.text3)
                        .truncate()
                        .child(path),
                )
                .child(
                    div()
                        .text_size(px(28.))
                        .font_weight(FontWeight::EXTRA_BOLD)
                        .truncate()
                        .child(name),
                ),
        )
        .child(tabs)
        .children(buttons)
}

/// `~/code/shop · compose.yaml`.
fn location(project: &ComposeProject) -> String {
    let home = std::env::home_dir();
    let dir = project.short_working_dir(home.as_deref());
    let files: Vec<&str> = project
        .config_files
        .iter()
        .map(|file| file.rsplit(['/', '\\']).next().unwrap_or(file))
        .collect();
    match (dir, files.is_empty()) {
        (Some(dir), false) => format!("{dir} · {}", files.join(", ")),
        (Some(dir), true) => dir,
        (None, _) => "Compose project".into(),
    }
}

fn buttons(
    project: &ComposeProject,
    (active, count): (usize, usize),
    handle: &Entity<Workspace>,
    workspace: &Workspace,
    palette: &Palette,
) -> Div {
    let name = &project.name;
    let shown = project
        .short_working_dir(std::env::home_dir().as_deref())
        .unwrap_or_default();
    let dir = project
        .working_dir
        .clone()
        .map(PathBuf::from)
        .filter(|dir| dir.is_dir());
    let compose = workspace.has_project_runner();

    let folder = {
        let help = match &dir {
            Some(_) => format!("Open {shown} with the default app for folders."),
            None => format!("The folder of {name} is not on this computer."),
        };
        let dir = dir.clone();
        secondary(
            "project-open-folder",
            "Open folder",
            dir.is_some(),
            palette,
            move |_, cx| {
                if let Some(dir) = &dir {
                    cx.open_with_system(dir);
                }
            },
        )
        .help(help)
    };
    let terminal = {
        let (dir, handle) = (dir.clone(), handle.clone());
        secondary(
            "project-terminal",
            "Terminal",
            dir.is_some(),
            palette,
            move |_, cx| {
                let Some(dir) = dir.clone() else { return };
                handle.update(cx, |workspace, cx| workspace.open_terminal_in(dir, cx));
            },
        )
        .help(format!(
            "Open a terminal tab in {shown}, in Captain's terminal panel. Its docker uses the engine Captain shows."
        ))
    };
    let down = {
        let (handle, project) = (handle.clone(), name.clone());
        secondary(
            "project-down",
            "Down",
            compose && count > 0,
            palette,
            move |window, cx| down_dialog::open(project.clone(), handle.clone(), window, cx),
        )
        .help(down_help(name, count))
    };
    let (action, label, help) = if active == 0 {
        (ProjectAction::Up, "Up", up_help(name))
    } else {
        (
            ProjectAction::Restart,
            "Restart project",
            restart_help(&project.services_label(), name),
        )
    };
    let primary = {
        let (handle, project) = (handle.clone(), project.clone());
        primary_button(
            "project-primary",
            label,
            help,
            compose,
            palette,
            move |_, _, cx| {
                handle.update(cx, |workspace, cx| {
                    workspace.run_project_action_on(project.clone(), action, cx)
                })
            },
        )
        .h(px(32.))
        .px(px(14.))
        .text_size(px(12.))
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
    };
    div()
        .flex()
        .items_center()
        .gap(px(8.))
        .child(folder)
        .child(terminal)
        .when(count > 0, |this| this.child(down))
        .child(primary)
}

fn secondary(
    id: &'static str,
    label: &'static str,
    enabled: bool,
    palette: &Palette,
    on_click: impl Fn(&mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let hover = palette.nav_selected;
    div()
        .id(id)
        .h(px(32.))
        .px(px(12.))
        .flex()
        .items_center()
        .rounded(px(8.))
        .border_1()
        .border_color(palette.border_strong)
        .bg(palette.field)
        .text_size(px(12.))
        .font_weight(FontWeight::SEMIBOLD)
        .when(!enabled, |this| this.opacity(0.45))
        .when(enabled, |this| {
            this.cursor_pointer()
                .hover(move |style| style.bg(hover))
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .on_click(move |_, window, cx| on_click(window, cx))
        })
        .child(label)
}

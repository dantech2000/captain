//! The Projects list: a switch per Compose project, and Kubernetes when it is on.

use captain_core::kubernetes::KubernetesStatus;
use captain_core::model::{ContainerAction, ProjectAction};
use gpui_kit::*;

use super::controls::switch;
use super::popover_data::ProjectRow;
use crate::help::HelpExt;
use crate::icons::{CaptainIcon, cap_icon};
use crate::kubernetes::kubernetes_model;
use crate::theme::Palette;
use crate::workspace::Workspace;

pub fn render(
    projects: &[ProjectRow],
    kubernetes: Option<&(KubernetesStatus, bool)>,
    workspace: &Entity<Workspace>,
    palette: &Palette,
) -> Div {
    div()
        .flex()
        .flex_col()
        .child(heading("Projects", palette))
        .children(
            projects
                .iter()
                .map(|project| project_row(project, workspace, palette)),
        )
        .children(kubernetes.map(|(status, on)| kubernetes_row(status, *on, palette)))
}

/// A small section title.
pub fn heading(title: &'static str, palette: &Palette) -> Div {
    div()
        .px(px(16.))
        .pt(px(6.))
        .pb(px(4.))
        .text_size(px(11.))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(palette.text3)
        .child(title)
}

fn row(
    id: SharedString,
    icon: CaptainIcon,
    color: Hsla,
    name: String,
    state: String,
    palette: &Palette,
) -> Stateful<Div> {
    div()
        .id(id)
        .flex_1()
        .min_w_0()
        .flex()
        .items_center()
        .gap(px(10.))
        .child(
            div()
                .size(px(22.))
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(6.))
                .bg(color.opacity(0.16))
                .child(cap_icon(icon, px(16.), color)),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .truncate()
                .font_weight(FontWeight::SEMIBOLD)
                .child(name),
        )
        .child(
            div()
                .flex_shrink_0()
                .text_size(px(11.5))
                .text_color(palette.text2)
                .child(state),
        )
}

fn line(content: Stateful<Div>, toggle: Stateful<Div>) -> Div {
    div()
        .h(px(38.))
        .px(px(16.))
        .flex()
        .items_center()
        .gap(px(10.))
        .child(content)
        .child(toggle)
}

fn project_row(project: &ProjectRow, workspace: &Entity<Workspace>, palette: &Palette) -> Div {
    let name = project.name.clone();
    let state = match (project.pending, project.active) {
        (Some(action), _) => action.progress_label().to_string(),
        (None, 0) => "Stopped".to_string(),
        (None, active) => format!("{active} of {} running", project.total),
    };
    let on = project.active > 0;
    let help = if on {
        format!("Stop every service of {name}. The containers stay.")
    } else {
        format!("Start the services of {name} (docker compose up).")
    };
    let content = row(
        format!("popover-project-{name}").into(),
        CaptainIcon::Stack,
        palette.project_color(&name),
        name.clone(),
        state,
        palette,
    )
    .help(format!(
        "The Compose project {name}: how many of its containers run."
    ));
    let (workspace, ids) = (workspace.clone(), project.ids.clone());
    let toggle = switch(
        format!("popover-project-switch-{name}"),
        on,
        project.pending.is_none(),
        help,
        move |on, _, cx| {
            let (name, ids) = (name.clone(), ids.clone());
            workspace.update(cx, |workspace, cx| {
                let action = if on {
                    ProjectAction::Up
                } else {
                    ProjectAction::Stop
                };
                if workspace.has_project_runner() {
                    workspace.run_project_action(name, action, cx);
                } else {
                    let action = if on {
                        ContainerAction::Start
                    } else {
                        ContainerAction::Stop
                    };
                    workspace.run_actions(ids, action, cx);
                }
            });
        },
    );
    line(content, toggle)
}

fn kubernetes_row(status: &KubernetesStatus, on: bool, palette: &Palette) -> Div {
    let state = match status {
        KubernetesStatus::Running { version } => format!("On · {version}"),
        status => status.label().to_string(),
    };
    let help = if on {
        "Stop Kubernetes in Captain Engine. Its settings stay."
    } else {
        "Start Kubernetes in Captain Engine."
    };
    let content = row(
        "popover-kubernetes".into(),
        CaptainIcon::Cluster,
        palette.accent,
        "Kubernetes".to_string(),
        state,
        palette,
    )
    .help("The k3s cluster in Captain Engine.");
    let toggle = switch("popover-kubernetes-switch", on, true, help, |on, _, cx| {
        if let Some(model) = kubernetes_model(cx) {
            model.update(cx, |model, cx| {
                model.turn_on(on, cx);
                model.apply(cx);
            });
        }
    });
    line(content, toggle)
}

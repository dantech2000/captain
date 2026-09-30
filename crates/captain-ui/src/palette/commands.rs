use captain_core::model::{
    ComposeProject, Container, ContainerAction, ContainerState, ProjectAction,
};
use captain_core::store::ContainerFilter;
use gpui_kit::assets::IconName;

use super::command::{Command, CommandKind, Section};
use crate::icons::CaptainIcon;
use crate::theme::Palette;
use crate::workspace::{Page, Workspace};

/// Every command the palette offers for the current workspace.
pub fn build(workspace: &Workspace, palette: &Palette) -> Vec<Command> {
    let mut commands = navigation(workspace, palette);
    commands.push(bring_data(palette));
    commands.push(disk(palette));
    commands.extend(filters(workspace, palette));
    if workspace.has_project_runner() {
        for project in workspace.compose_projects() {
            commands.extend(project_actions(&project, palette));
        }
    }
    for container in workspace.store().containers() {
        commands.extend(container_actions(container, palette));
    }
    for container in workspace.store().containers() {
        commands.push(show(container, palette));
    }
    commands
}

fn navigation(workspace: &Workspace, palette: &Palette) -> Vec<Command> {
    Page::ALL
        .into_iter()
        .map(|page| Command {
            section: Section::Navigate,
            title: format!("Go to {}", page.label()),
            meta: match page {
                Page::Containers => count_label(workspace.store().len()),
                _ => "Page".into(),
            },
            icon: page.icon(),
            color: palette.accent,
            suggested: true,
            kind: CommandKind::GoTo(page),
        })
        .collect()
}

/// Opens the Storage page. Typing `disk` finds it.
fn disk(palette: &Palette) -> Command {
    Command {
        section: Section::Navigate,
        title: "disk: Storage and cleanup".into(),
        meta: "What fills the engine's disk".into(),
        icon: CaptainIcon::Reclaim.into(),
        color: palette.accent,
        suggested: false,
        kind: CommandKind::GoTo(Page::Storage),
    }
}

/// Opens the Migration Assistant.
fn bring_data(palette: &Palette) -> Command {
    Command {
        section: Section::Actions,
        title: "Bring data from another engine…".into(),
        meta: "Migration Assistant".into(),
        icon: IconName::Download.into(),
        color: palette.teal,
        suggested: false,
        kind: CommandKind::BringData,
    }
}

fn filters(workspace: &Workspace, palette: &Palette) -> Vec<Command> {
    let containers = workspace.store().containers();
    ContainerFilter::ALL
        .into_iter()
        .map(|filter| {
            let count = containers.iter().filter(|c| filter.matches(c)).count();
            let current = if filter == workspace.filter() {
                " · current filter"
            } else {
                ""
            };
            Command {
                section: Section::Actions,
                title: match filter {
                    ContainerFilter::All => "Show all containers".into(),
                    _ => format!("Show {} containers", filter.label().to_lowercase()),
                },
                meta: format!("{}{current}", count_label(count)),
                icon: IconName::ListFilter.into(),
                color: palette.indigo,
                suggested: false,
                kind: CommandKind::SetFilter(filter),
            }
        })
        .collect()
}

/// Start or Stop, Restart, and Open in browser for each published port.
fn container_actions(container: &Container, palette: &Palette) -> Vec<Command> {
    if container.state == ContainerState::Removing {
        return Vec::new();
    }
    let name = &container.name;
    let meta = container_meta(container);
    let action = |action: ContainerAction, icon: IconName, color| Command {
        section: Section::Actions,
        title: format!("{} {name}", action.label()),
        meta: meta.clone(),
        icon: icon.into(),
        color,
        suggested: false,
        kind: CommandKind::Run {
            id: container.id.clone(),
            action,
        },
    };

    let mut commands = vec![
        if container.state.is_active() {
            action(ContainerAction::Stop, IconName::Square, palette.red)
        } else {
            action(ContainerAction::Start, IconName::Play, palette.green)
        },
        action(ContainerAction::Restart, IconName::RotateCw, palette.accent),
    ];
    commands.extend(container.published_ports().into_iter().map(|port| Command {
        section: Section::Actions,
        title: format!("Open {name} in browser"),
        meta: format!("localhost:{port}"),
        icon: IconName::ExternalLink.into(),
        color: palette.teal,
        suggested: false,
        kind: CommandKind::OpenPort(port),
    }));
    commands
}

/// Up, Down, and Restart for a Compose project.
fn project_actions(project: &ComposeProject, palette: &Palette) -> Vec<Command> {
    let meta = format!(
        "{} · {}",
        project.services_label(),
        project.status().label()
    );
    [
        (ProjectAction::Up, IconName::Play, palette.green),
        (ProjectAction::Down, IconName::PowerOff, palette.red),
        (ProjectAction::Restart, IconName::RotateCw, palette.accent),
    ]
    .into_iter()
    .map(|(action, icon, color)| Command {
        section: Section::Actions,
        title: format!("{} project {}", action.label(), project.name),
        meta: meta.clone(),
        icon: icon.into(),
        color,
        suggested: false,
        kind: CommandKind::RunProject {
            project: project.name.clone(),
            action,
        },
    })
    .collect()
}

/// Selects the container. Running containers are suggested before the user types.
fn show(container: &Container, palette: &Palette) -> Command {
    let color = palette.container_state(container.state);
    Command {
        section: Section::Containers,
        title: container.name.clone(),
        meta: format!("{} · {}", container_meta(container), container.image),
        icon: CaptainIcon::Container.into(),
        color,
        suggested: container.state == ContainerState::Running,
        kind: CommandKind::Show(container.id.clone()),
    }
}

/// The project, if any, and the state, for example `shop · running 3 hours`.
fn container_meta(container: &Container) -> String {
    let state = match container.state {
        ContainerState::Running => format!("running {}", container.uptime_label()),
        state => state.label().to_string(),
    };
    match &container.compose_project {
        Some(project) => format!("{project} · {state}"),
        None => state,
    }
}

fn count_label(count: usize) -> String {
    match count {
        1 => "1 container".into(),
        n => format!("{n} containers"),
    }
}

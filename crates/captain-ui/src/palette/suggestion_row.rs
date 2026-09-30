use captain_core::grammar::{
    Action, Catalog, Suggestion, SuggestionKind, Target, Verb, duration_label,
};
use captain_core::model::{PortLink, count_label};
use gpui_kit::Hsla;
use gpui_kit::assets::IconName;

use super::command::{Command, CommandKind, Section};
use super::commands::container_meta;
use super::ranking::Ranked;
use super::row_help;
use super::run_action::page_of;
use crate::icons::{CaptainIcon, Glyph};
use crate::theme::Palette;
use crate::workspace::Workspace;

/// A palette row for a suggestion of the grammar: the line with its typed parts in
/// bold, and what the last word names in muted text, such as "shop · running 3h".
pub fn ranked(
    suggestion: Suggestion,
    catalog: &Catalog,
    workspace: &Workspace,
    palette: &Palette,
) -> Ranked {
    let (icon, color) = look(&suggestion, catalog, palette);
    let meta = match (&suggestion.kind, &suggestion.action) {
        (SuggestionKind::Line, Some(action)) => action_meta(action, catalog),
        (kind, _) => kind_meta(kind, catalog, workspace),
    };
    let help = row_help::help(&suggestion, catalog, workspace);
    let line = suggestion.line;
    let kind = match suggestion.action {
        Some(action) => CommandKind::Act(action),
        None => CommandKind::Complete(line.clone()),
    };
    Ranked {
        command: Command {
            section: Section::Commands,
            title: line.clone(),
            meta,
            icon,
            color,
            suggested: false,
            kind,
            help,
            completion: Some(line),
        },
        ranges: suggestion.bold,
    }
}

fn verb_icon(verb: Verb) -> Glyph {
    match verb {
        Verb::Start | Verb::Resume | Verb::Up => IconName::Play.into(),
        Verb::Stop => IconName::Square.into(),
        Verb::Restart => IconName::RotateCw.into(),
        Verb::Pause => IconName::Pause.into(),
        Verb::Down => IconName::PowerOff.into(),
        Verb::Logs | Verb::Float => IconName::FileText.into(),
        Verb::Shell => CaptainIcon::Exec.into(),
        Verb::Open => IconName::ExternalLink.into(),
        Verb::Forward => CaptainIcon::Forward.into(),
        Verb::Disk => CaptainIcon::Reclaim.into(),
        Verb::Go => IconName::ArrowUpRight.into(),
    }
}

/// The icon and its color: the target's for a name, the verb's otherwise.
fn look(suggestion: &Suggestion, catalog: &Catalog, palette: &Palette) -> (Glyph, Hsla) {
    let target = |target: &Target| match target {
        Target::Container(id) => (
            CaptainIcon::Container.into(),
            catalog
                .container(id)
                .map(|c| palette.container_state(c.state))
                .unwrap_or(palette.text2),
        ),
        Target::Service { .. } => (CaptainIcon::Container.into(), palette.text2),
        Target::Project(_) => (CaptainIcon::Stack.into(), palette.indigo),
        Target::Port(_) => (IconName::ExternalLink.into(), palette.teal),
        Target::KubeService(_) => (CaptainIcon::Forward.into(), palette.accent_fg),
        Target::Page(page) => (page_of(*page).icon(), palette.accent_fg),
    };
    match &suggestion.kind {
        SuggestionKind::Target(t) => target(t),
        SuggestionKind::Verb(verb) => (verb_icon(*verb), palette.accent_fg),
        SuggestionKind::Flag(_) => (IconName::ListFilter.into(), palette.text2),
        SuggestionKind::Time(_) => (IconName::Clock.into(), palette.text2),
        SuggestionKind::Line => {
            let verb = suggestion
                .line
                .split_whitespace()
                .next()
                .and_then(Verb::parse);
            (
                verb.map_or(IconName::ArrowUpRight.into(), verb_icon),
                palette.accent_fg,
            )
        }
    }
}

/// What a suggestion adds, in muted text.
fn kind_meta(kind: &SuggestionKind, catalog: &Catalog, workspace: &Workspace) -> String {
    match kind {
        SuggestionKind::Verb(verb) => verb.usage().into(),
        SuggestionKind::Flag(flag) => flag.help().into(),
        SuggestionKind::Time(time) => format!("Only lines from {}", duration_label(*time)),
        SuggestionKind::Line => String::new(),
        SuggestionKind::Target(target) => match target {
            Target::Container(id) => catalog
                .container(id)
                .map(container_meta)
                .unwrap_or_default(),
            Target::Service { project, .. } => {
                format!(
                    "{project} · {}",
                    count_label(catalog.ids(target).len(), "container")
                )
            }
            Target::Project(name) => workspace
                .compose_projects()
                .iter()
                .find(|project| project.name == *name)
                .map(|p| format!("{} · {}", p.services_label(), p.status().label()))
                .unwrap_or_else(|| "Project".into()),
            Target::Port(port) => format!("localhost:{port}"),
            Target::KubeService(key) => format!("{} · port {}", key.namespace, key.port),
            Target::Page(_) => "Page".into(),
        },
    }
}

/// What the typed line does, in muted text.
fn action_meta(action: &Action, catalog: &Catalog) -> String {
    let container = |id: &str| {
        catalog
            .container(id)
            .map(container_meta)
            .unwrap_or_default()
    };
    match action {
        Action::Containers { ids, .. } => match ids.as_slice() {
            [id] => container(id),
            _ => count_label(ids.len(), "container"),
        },
        Action::Project { action, .. } => {
            format!("docker compose {}", action.label().to_lowercase())
        }
        Action::Logs { id, since, errors } => {
            let mut meta = container(id);
            if *errors {
                meta.push_str(" · errors only");
            }
            if let Some(since) = since {
                meta.push_str(&format!(" · {}", duration_label(*since)));
            }
            meta
        }
        Action::ProjectLog(project) => format!("One log for all services of {project}"),
        Action::Shell(id) | Action::Float(id) => container(id),
        Action::Open(PortLink::Open(url)) => url.clone(),
        Action::Open(PortLink::Copy { address, .. }) => format!("Copy {address}"),
        Action::Forward { key, local_port } => {
            let local = local_port.map_or("a free port".to_string(), |p| format!("localhost:{p}"));
            format!("{}:{} to {local}", key.service, key.port)
        }
        Action::Go(_) => "Page".into(),
    }
}

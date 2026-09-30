use captain_core::grammar::{Action, Catalog, Suggestion, SuggestionKind, Target, duration_label};
use captain_core::model::{ContainerState, PortLink, count_label};

use super::commands::project_help;
use super::run_action::page_of;
use crate::containers::action_help;
use crate::shell::page_help;
use crate::workspace::{Page, Workspace};

/// The status bar sentence for a grammar row. A row that runs uses the sentence of
/// the button that does the same thing.
pub fn help(suggestion: &Suggestion, catalog: &Catalog, workspace: &Workspace) -> String {
    match (&suggestion.action, &suggestion.kind) {
        (Some(action), _) => run_help(action, catalog, workspace),
        (None, SuggestionKind::Verb(verb)) => verb.help().into(),
        (None, SuggestionKind::Flag(flag)) => flag.help().into(),
        (None, SuggestionKind::Target(target @ Target::Service { project, service })) => {
            let count = catalog.ids(target).len();
            format!("{project}/{service} has {count} containers. Pick one.")
        }
        _ => format!(
            "Press Tab to fill in \u{201c}{}\u{201d}, then finish the command.",
            suggestion.line
        ),
    }
}

fn run_help(action: &Action, catalog: &Catalog, workspace: &Workspace) -> String {
    let name = |id: &str| {
        catalog
            .container(id)
            .map(|c| c.name.clone())
            .unwrap_or_default()
    };
    match action {
        Action::Containers { ids, action } => match ids.as_slice() {
            [id] => action_help(*action, &name(id)),
            _ => format!(
                "{} {}.",
                action.label(),
                count_label(ids.len(), "container")
            ),
        },
        Action::Project { name, action } => workspace
            .compose_projects()
            .iter()
            .find(|project| project.name == *name)
            .map(|project| project_help(project, *action))
            .unwrap_or_else(|| format!("{} project {name}.", action.label())),
        Action::Logs { id, since, errors } => {
            let mut help = format!("Show the logs of {} in the inspector.", name(id));
            if *errors {
                help.push_str(" Only error lines show.");
            }
            if let Some(since) = since {
                help.push_str(&format!(
                    " Only lines from {} show.",
                    duration_label(*since)
                ));
            }
            help
        }
        Action::ProjectLog(_) => page_help(Page::Project, None),
        Action::Shell(id) => match catalog.container(id) {
            Some(c) if c.state != ContainerState::Running => {
                format!("{} does not run, so it has no shell.", c.name)
            }
            _ => format!("Open a shell in {}.", name(id)),
        },
        Action::Float(id) => format!(
            "Open {}'s log in a small window that stays on top.",
            name(id)
        ),
        Action::Open(PortLink::Open(url)) => format!("Open {url} in your browser."),
        Action::Open(PortLink::Copy { address, service }) => {
            format!("Copy {address}. {service} does not serve web pages.")
        }
        Action::Forward { key, .. } => format!(
            "Forward a port on this computer to {}:{}.",
            key.service, key.port
        ),
        Action::Go(page) => page_help(page_of(*page), None),
    }
}

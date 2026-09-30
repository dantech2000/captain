use std::time::Duration;

use crate::kubernetes::check_local_port;
use crate::model::{ContainerAction, PortLink, ProjectAction};

use super::action::{Action, Destination};
use super::catalog::{Catalog, Resolution, Target};
use super::duration::parse_duration;
use super::verb::{Flag, Kinds, Verb};

/// Why a line does not run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// The first word is not a verb, so the line is a plain search.
    NotACommand,
    /// A word is missing.
    Incomplete(String),
    /// A name stands for more than one thing. The user picks one.
    Ambiguous { name: String, targets: Vec<Target> },
    /// An unknown name, option, or time.
    Invalid(String),
}

impl ParseError {
    /// A sentence for the palette.
    pub fn message(&self) -> String {
        match self {
            ParseError::NotACommand => "Not a command.".into(),
            ParseError::Incomplete(message) | ParseError::Invalid(message) => message.clone(),
            ParseError::Ambiguous { name, .. } => {
                format!("\u{201c}{name}\u{201d} names more than one thing. Pick one.")
            }
        }
    }
}

/// The words after the verb: the names in order, and the options.
#[derive(Debug, Default)]
pub(super) struct Scan<'a> {
    pub names: Vec<&'a str>,
    pub since: Option<Duration>,
    pub errors: bool,
    /// True when the line ends in `--since` without its time.
    pub wants_time: bool,
}

/// Splits the words after the verb into names and options.
pub(super) fn scan<'a>(verb: Verb, words: &[&'a str]) -> Result<Scan<'a>, ParseError> {
    let mut scan = Scan::default();
    let mut rest = words.iter().copied();
    while let Some(word) = rest.next() {
        let Some(option) = word.strip_prefix("--") else {
            scan.names.push(word);
            continue;
        };
        let (option, value) = match option.split_once('=') {
            Some((option, value)) => (option, Some(value)),
            None => (option, None),
        };
        let flag = verb
            .flags()
            .iter()
            .find(|flag| flag.text()[2..] == *option)
            .ok_or_else(|| ParseError::Invalid(unknown_option(verb, word)))?;
        match flag {
            Flag::Errors => scan.errors = true,
            Flag::Since => match value.or_else(|| rest.next()) {
                Some(value) => {
                    scan.since = Some(parse_duration(value).map_err(ParseError::Invalid)?);
                }
                None => scan.wants_time = true,
            },
        }
    }
    Ok(scan)
}

fn unknown_option(verb: Verb, word: &str) -> String {
    let flags: Vec<&str> = verb.flags().iter().map(|flag| flag.text()).collect();
    match flags.as_slice() {
        [] => format!("{} has no options, so {word} does not fit.", verb.name()),
        _ => format!(
            "{} has no option {word}. Use {}.",
            verb.name(),
            flags.join(" or ")
        ),
    }
}

/// Parses a complete command. Names must match live data exactly; a name that
/// matches several things is [`ParseError::Ambiguous`], never a guess.
pub fn parse(line: &str, catalog: &Catalog) -> Result<Action, ParseError> {
    let words: Vec<&str> = line.split_whitespace().collect();
    let Some((&first, rest)) = words.split_first() else {
        return Err(ParseError::NotACommand);
    };
    let Some(verb) = Verb::parse(first) else {
        // A page name alone opens the page.
        return match (Destination::parse(first), rest) {
            (Some(page), []) => Ok(Action::Go(page)),
            _ => Err(ParseError::NotACommand),
        };
    };
    let scan = scan(verb, rest)?;
    if scan.wants_time {
        return Err(ParseError::Incomplete(
            "--since needs a time, such as 10m, 1h, or 30s.".into(),
        ));
    }
    let slots = verb.slots();
    if scan.names.len() > slots.len() {
        return Err(ParseError::Invalid(format!(
            "Too many words. Use: {}",
            verb.usage()
        )));
    }
    if scan.names.len() < verb.required() {
        return Err(ParseError::Incomplete(format!(
            "{} needs a {}.",
            verb.name(),
            slots[0].nouns()
        )));
    }
    if verb == Verb::Forward && !catalog.kubernetes {
        return Err(ParseError::Invalid(
            "Kubernetes does not run. Turn it on in Settings to forward a service.".into(),
        ));
    }
    let mut targets = Vec::new();
    for (name, kinds) in scan.names.iter().zip(slots) {
        match catalog.resolve(name, *kinds) {
            Resolution::Found(target) => targets.push(target),
            Resolution::Ambiguous(targets) => {
                return Err(ParseError::Ambiguous {
                    name: name.to_string(),
                    targets,
                });
            }
            Resolution::Missing => return Err(ParseError::Invalid(missing(name, *kinds))),
        }
    }
    build(verb, &targets, &scan, catalog)
}

fn missing(name: &str, kinds: Kinds) -> String {
    if kinds.port && !kinds.container {
        return format!("\u{201c}{name}\u{201d} is not a port. Use a number above 1024.");
    }
    format!("No {} named \u{201c}{name}\u{201d}.", kinds.nouns())
}

/// The action for `verb` with its resolved words. `scan.names` names them.
pub(super) fn build(
    verb: Verb,
    targets: &[Target],
    scan: &Scan,
    catalog: &Catalog,
) -> Result<Action, ParseError> {
    let Some(target) = targets.first() else {
        return match verb {
            Verb::Disk => Ok(Action::Go(Destination::Storage)),
            _ => Err(ParseError::Incomplete(verb.usage().into())),
        };
    };
    let name = scan.names.first().copied().unwrap_or_default();
    match (verb, target) {
        (Verb::Go, Target::Page(page)) => Ok(Action::Go(*page)),
        (Verb::Up, Target::Project(project)) => compose(project, ProjectAction::Up, catalog),
        (Verb::Down, Target::Project(project)) => compose(project, ProjectAction::Down, catalog),
        (Verb::Logs, Target::Project(project)) => project_log(project, scan, catalog),
        (Verb::Logs, _) => Ok(Action::Logs {
            id: one_container(name, target, catalog)?,
            since: scan.since,
            errors: scan.errors,
        }),
        (Verb::Shell, _) => one_container(name, target, catalog).map(Action::Shell),
        (Verb::Float, _) => one_container(name, target, catalog).map(Action::Float),
        (Verb::Open, _) => open(name, target, catalog),
        (Verb::Forward, Target::KubeService(key)) => {
            let local_port = match targets.get(1) {
                Some(Target::Port(port)) => {
                    check_local_port(*port).map_err(ParseError::Invalid)?;
                    Some(*port)
                }
                _ => None,
            };
            Ok(Action::Forward {
                key: key.clone(),
                local_port,
            })
        }
        _ => run(verb, target, catalog),
    }
}

/// Start, stop, restart, pause, or resume. On a project, start, stop, and restart
/// are Compose commands; pause and resume go to each container.
fn run(verb: Verb, target: &Target, catalog: &Catalog) -> Result<Action, ParseError> {
    let (action, compose_action) = match verb {
        Verb::Start => (ContainerAction::Start, Some(ProjectAction::Up)),
        Verb::Stop => (ContainerAction::Stop, Some(ProjectAction::Stop)),
        Verb::Restart => (ContainerAction::Restart, Some(ProjectAction::Restart)),
        Verb::Pause => (ContainerAction::Pause, None),
        _ => (ContainerAction::Unpause, None),
    };
    match (target, compose_action) {
        (Target::Project(project), Some(compose_action)) => {
            compose(project, compose_action, catalog)
        }
        _ => Ok(Action::Containers {
            ids: catalog.ids(target),
            action,
        }),
    }
}

fn compose(project: &str, action: ProjectAction, catalog: &Catalog) -> Result<Action, ParseError> {
    if !catalog.compose {
        return Err(ParseError::Invalid(
            "Project commands need docker compose, and Captain cannot find it.".into(),
        ));
    }
    Ok(Action::Project {
        name: project.into(),
        action,
    })
}

/// The Project page shows one log for all services; it has no filters.
fn project_log(project: &str, scan: &Scan, catalog: &Catalog) -> Result<Action, ParseError> {
    if scan.since.is_none() && !scan.errors {
        return Ok(Action::ProjectLog(project.into()));
    }
    let example = catalog
        .members(project, None)
        .first()
        .and_then(|c| c.compose.service.clone())
        .map(|service| format!(", such as {project}/{service}"))
        .unwrap_or_default();
    Err(ParseError::Invalid(format!(
        "The project log has no filters. Name one service{example}."
    )))
}

/// The container `target` stands for. A service with several containers is
/// ambiguous: the user picks one.
fn one_container(name: &str, target: &Target, catalog: &Catalog) -> Result<String, ParseError> {
    match target {
        Target::Container(id) => Ok(id.clone()),
        _ => Err(ParseError::Ambiguous {
            name: name.into(),
            targets: catalog
                .ids(target)
                .into_iter()
                .map(Target::Container)
                .collect(),
        }),
    }
}

/// The first published port of a container or service, or a port by number.
fn open(name: &str, target: &Target, catalog: &Catalog) -> Result<Action, ParseError> {
    let ids = catalog.ids(target);
    let mappings = catalog
        .containers()
        .filter(|c| match target {
            Target::Port(_) => true,
            _ => ids.contains(&c.id),
        })
        .flat_map(|c| c.ports.iter())
        .filter_map(|p| Some((p.public_port?, p.private_port)));
    let found = match target {
        Target::Port(port) => {
            let private = mappings
                .filter(|(public, _)| public == port)
                .map(|(_, private)| private)
                .next();
            Some((*port, private.unwrap_or(*port)))
        }
        _ => mappings.into_iter().next(),
    };
    match found {
        Some((public, private)) => Ok(Action::Open(PortLink::of(public, private))),
        None => Err(ParseError::Invalid(format!("{name} publishes no port."))),
    }
}

#[cfg(test)]
mod tests;

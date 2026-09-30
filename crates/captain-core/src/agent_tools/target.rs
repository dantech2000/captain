//! Checks the names that agents pass against the live lists. Only a name that
//! matches a real container or project reaches the engine, and a value that starts
//! with `-` never does, so it cannot pass as an option to a command.

use crate::model::Container;

/// How many names an error lists.
const LISTED: usize = 20;

/// Refuses a value that is empty, starts with `-`, has control characters, or is
/// longer than any name.
pub fn check_name<'a>(what: &str, value: &'a str) -> Result<&'a str, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(format!("Give a {what} name."));
    }
    if value.starts_with('-') || value.chars().any(char::is_control) || value.len() > 256 {
        return Err(format!("\"{value}\" is not a {what} name."));
    }
    Ok(value)
}

/// The container named `name`: its name, its shown name (`pod/container` in
/// Kubernetes), its ID, or a unique ID prefix of at least 4 characters.
pub fn find_container<'a>(
    containers: &'a [Container],
    name: &str,
) -> Result<&'a Container, String> {
    let name = check_name("container", name)?;
    let exact = containers
        .iter()
        .find(|c| c.name == name || c.display_name() == name || c.id == name);
    if let Some(container) = exact {
        return Ok(container);
    }
    let prefixed: Vec<&Container> = containers
        .iter()
        .filter(|c| name.len() >= 4 && c.id.starts_with(name))
        .collect();
    match prefixed.as_slice() {
        [one] => Ok(one),
        [] => Err(format!(
            "No container is named \"{name}\". {}",
            listed("Containers", containers.iter().map(Container::display_name))
        )),
        _ => Err(format!(
            "More than one container ID starts with \"{name}\"."
        )),
    }
}

/// The containers of the Compose project `name`, which must have at least one.
pub fn find_project<'a>(
    containers: &'a [Container],
    name: &str,
) -> Result<Vec<&'a Container>, String> {
    let name = check_name("project", name)?;
    let members: Vec<&Container> = containers
        .iter()
        .filter(|c| c.compose_project.as_deref() == Some(name))
        .collect();
    if !members.is_empty() {
        return Ok(members);
    }
    let mut projects: Vec<String> = containers
        .iter()
        .filter_map(|c| c.compose_project.clone())
        .collect();
    projects.sort();
    projects.dedup();
    Err(format!(
        "No Compose project is named \"{name}\". {}",
        listed("Projects", projects.into_iter())
    ))
}

/// `Containers: a, b, c.`, at most [`LISTED`] names, or a note that there are none.
fn listed(what: &str, names: impl Iterator<Item = String>) -> String {
    let names: Vec<String> = names.collect();
    match names.len() {
        0 => format!("{what}: none."),
        n if n > LISTED => format!(
            "{what}: {}, and {} more.",
            names[..LISTED].join(", "),
            n - LISTED
        ),
        _ => format!("{what}: {}.", names.join(", ")),
    }
}

#[cfg(test)]
mod tests;

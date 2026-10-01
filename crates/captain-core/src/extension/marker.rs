//! How Captain tells an extension's backend containers from the user's own. Docker
//! Desktop hides them unless the user turns on "Show Docker Extensions system
//! containers" ([settings](https://docs.docker.com/extensions/settings-feedback/)),
//! and Captain does the same.

use super::PROJECT_PREFIX;

/// The label on every container of an extension's backend, with the extension ID
/// as its value. Docker Desktop's own extensions look for this key: the Resource
/// Usage extension counts a container that has it as a system container.
pub const EXTENSION_LABEL: &str = "com.docker.desktop.extension";

/// The extension that runs the container: the [`EXTENSION_LABEL`] value, or else the
/// Compose project name after `captain-ext-`, for backends that Captain started
/// before it set the label. `None` for every other container.
pub fn backend_extension(label: Option<&str>, project: Option<&str>) -> Option<String> {
    label
        .or_else(|| project.and_then(|project| project.strip_prefix(PROJECT_PREFIX)))
        .filter(|id| !id.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests;

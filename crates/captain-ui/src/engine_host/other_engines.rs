//! The engines the setup screen offers to copy from: the detected ones, without
//! Captain Engine itself.

use std::path::{Path, PathBuf};

use crate::settings::DetectedEndpoint;

/// `detected` without the entries that point at `captain`, Captain Engine's own
/// endpoint. A socket that links to Captain Engine's socket counts as Captain.
pub(super) fn other_engines(
    detected: Vec<DetectedEndpoint>,
    captain: Option<&str>,
) -> Vec<DetectedEndpoint> {
    let Some(captain) = captain else {
        return detected;
    };
    let captain_path = socket_path(captain);
    detected
        .into_iter()
        .filter(|engine| {
            engine.host.as_ref() != captain
                && (captain_path.is_none() || socket_path(&engine.host) != captain_path)
        })
        .collect()
}

/// The resolved file behind a `unix://` URL, if it exists.
fn socket_path(host: &str) -> Option<PathBuf> {
    let path = host.strip_prefix("unix://")?;
    Path::new(path).canonicalize().ok()
}

#[cfg(test)]
mod tests;

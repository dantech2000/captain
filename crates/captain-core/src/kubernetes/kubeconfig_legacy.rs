//! Captain's kubeconfig entries from before the rename to `captain-desktop`, when
//! the cluster, user, and context were all named `captain`. Captain removes them
//! only when they are its own: the `captain` cluster has a server on this Mac and
//! the certificate authority of Captain's cluster. A user's own `captain` context
//! stays. See ADR 0010.

use serde_json::Value;

use super::kubeconfig::{CONTEXT, remove_named};

/// The old name of Captain's cluster, user, and context.
pub const LEGACY_CONTEXT: &str = "captain";

/// `config` without Captain's old `captain` entries, when `cas` has the certificate
/// authority of its `captain` cluster. A current context of `captain` becomes
/// `captain-desktop`. Other configs come back unchanged.
pub fn remove_legacy(config: &Value, cas: &[String]) -> Value {
    let mut config = config.clone();
    if is_captains(&config, cas) {
        remove_named(&mut config, LEGACY_CONTEXT, CONTEXT);
    }
    config
}

/// True when the `captain` cluster points at `https://127.0.0.1:<port>` with one of
/// `cas`, and the `captain` context, if any, joins it to the `captain` user.
fn is_captains(config: &Value, cas: &[String]) -> bool {
    let named = |list: &str| {
        config[list]
            .as_array()
            .into_iter()
            .flatten()
            .find(|entry| entry["name"] == LEGACY_CONTEXT)
    };
    let Some(cluster) = named("clusters").map(|entry| &entry["cluster"]) else {
        return false;
    };
    let local = cluster["server"]
        .as_str()
        .is_some_and(|server| server.starts_with("https://127.0.0.1:"));
    let ours = cluster["certificate-authority-data"]
        .as_str()
        .is_some_and(|ca| cas.iter().any(|known| known == ca));
    let joined = named("contexts").is_none_or(|entry| {
        entry["context"]["cluster"] == LEGACY_CONTEXT && entry["context"]["user"] == LEGACY_CONTEXT
    });
    local && ours && joined
}

#[cfg(test)]
mod tests;

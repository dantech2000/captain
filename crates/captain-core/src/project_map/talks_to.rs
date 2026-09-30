use super::MapService;
use crate::model::EnvVar;

/// Key fragments that say a variable holds a host, so a bare name in its value counts.
const HOST_KEYS: &[&str] = &[
    "HOST", "ADDR", "SERVER", "URL", "URI", "DSN", "ENDPOINT", "BROKER",
];

/// The IDs of the other services that `service` names as a host in its environment,
/// sorted by name. See [`host_tokens`].
pub fn talks_to<'a>(service: &MapService, all: &'a [MapService]) -> Vec<&'a str> {
    let hosts: Vec<String> = service.env.iter().flat_map(host_tokens).collect();
    let mut targets: Vec<&MapService> = all
        .iter()
        .filter(|other| other.id != service.id)
        .filter(|other| {
            hosts.iter().any(|host| {
                host.eq_ignore_ascii_case(&other.name)
                    || host.eq_ignore_ascii_case(&other.container_name)
            })
        })
        .collect();
    targets.sort_by(|a, b| a.name.cmp(&b.name));
    targets.into_iter().map(|t| t.id.as_str()).collect()
}

/// The host names a variable's value names: the host of a URL
/// (`postgres://app@postgres:5432/db`), the host of `host:port`, and, when the key says
/// it holds a host (`REDIS_HOST`), a bare name. A value can list several, split by
/// commas, semicolons, or spaces.
pub fn host_tokens(var: &EnvVar) -> Vec<String> {
    let key = var.key.to_ascii_uppercase();
    let hosty = HOST_KEYS.iter().any(|marker| key.contains(marker));
    var.value
        .split([',', ';', ' '])
        .filter_map(|piece| host_of(piece.trim(), hosty))
        .collect()
}

/// The host of one piece of a value, if it names one.
fn host_of(piece: &str, hosty: bool) -> Option<String> {
    let (rest, url) = match piece.split_once("://") {
        Some((_, rest)) => (rest, true),
        None => (piece, false),
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    // `user:password@host` in a URL: the host follows the last `@`.
    let authority = authority.rsplit('@').next().unwrap_or_default();
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => (host, Some(port)),
        None => (authority, None),
    };
    let numeric_port = port.is_some_and(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()));
    let counts = url || numeric_port || (hosty && port.is_none() && rest == authority);
    let valid = !host.is_empty()
        && host
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.');
    (counts && valid).then(|| host.to_string())
}

#[cfg(test)]
mod tests;

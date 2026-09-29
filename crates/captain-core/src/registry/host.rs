/// Docker Hub's host name.
pub const DOCKER_HUB: &str = "docker.io";
/// The server address that the Docker CLI stores Docker Hub logins under.
const DOCKER_HUB_SERVER: &str = "https://index.docker.io/v1/";
/// Other names of Docker Hub in config keys.
const DOCKER_HUB_ALIASES: &[&str] = &["index.docker.io", "registry-1.docker.io"];

/// The registry of an image reference. The first path part is a registry when it
/// has a dot or a colon, or is `localhost`, as in the Docker CLI. Otherwise the image
/// is on Docker Hub.
pub fn registry_host(reference: &str) -> String {
    match reference.split_once('/') {
        Some((first, _)) if first.contains('.') || first.contains(':') || first == "localhost" => {
            normalize_host(first)
        }
        _ => DOCKER_HUB.to_string(),
    }
}

/// The server address to ask a credential helper for, and to send to the engine.
pub fn server_address(host: &str) -> String {
    if host == DOCKER_HUB {
        DOCKER_HUB_SERVER.to_string()
    } else {
        host.to_string()
    }
}

/// A config key or server URL as a host name, without the scheme and path, like the
/// CLI's `ConvertToHostname`. Docker Hub's other names become `docker.io`.
pub fn normalize_host(key: &str) -> String {
    let key = key
        .strip_prefix("https://")
        .or_else(|| key.strip_prefix("http://"))
        .unwrap_or(key);
    let host = key.split('/').next().unwrap_or(key);
    if DOCKER_HUB_ALIASES.contains(&host) {
        DOCKER_HUB.to_string()
    } else {
        host.to_string()
    }
}

#[cfg(test)]
mod tests;

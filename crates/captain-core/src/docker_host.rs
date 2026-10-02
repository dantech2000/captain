//! `DOCKER_HOST` values for the docker CLI.

/// The `DOCKER_HOST` value that points the docker CLI at `endpoint`, a URL Captain
/// connects to. The CLI knows `tcp://` but not `http://`
/// (<https://github.com/docker/cli/blob/master/opts/hosts.go>).
pub fn cli_host(endpoint: &str) -> String {
    match endpoint.strip_prefix("http://") {
        Some(address) => format!("tcp://{address}"),
        None => endpoint.to_string(),
    }
}

#[cfg(test)]
mod tests;

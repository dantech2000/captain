use std::time::Duration;

use bollard::{API_DEFAULT_VERSION, Docker};
use captain_core::EngineError;
use tokio::runtime::Runtime;

use crate::{Endpoint, mapping, runtime};

mod containers;
mod images;
mod networks;
mod volumes;

/// Seconds bollard waits for a single request.
const REQUEST_TIMEOUT_SECS: u64 = 120;
/// How long `connect` waits for the engine to answer the version handshake.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// An [`Engine`] backed by the Docker Engine API.
pub struct DockerEngine {
    docker: Docker,
    endpoint: Endpoint,
    runtime: Runtime,
}

impl DockerEngine {
    /// Connects to `endpoint` and agrees on an API version with the engine.
    ///
    /// This blocks for up to a few seconds. Call it from a background thread,
    /// never from inside a tokio runtime.
    pub fn connect(endpoint: Endpoint) -> Result<Self, EngineError> {
        let (docker, runtime) = connect(&endpoint)?;
        tracing::info!(%endpoint, "connected to Docker engine");
        Ok(Self {
            docker,
            endpoint,
            runtime,
        })
    }
}

/// A client for `endpoint` with an agreed API version, and the runtime it runs on.
/// Blocks for up to a few seconds; call it from a background thread.
pub(crate) fn connect(endpoint: &Endpoint) -> Result<(Docker, Runtime), EngineError> {
    let runtime = runtime::build().map_err(|err| EngineError::Unreachable(err.to_string()))?;
    let docker = runtime.block_on(async {
        let docker = client(endpoint).map_err(mapping::engine_error)?;
        tokio::time::timeout(CONNECT_TIMEOUT, docker.negotiate_version())
            .await
            .map_err(|_| EngineError::Unreachable(format!("{endpoint} did not answer")))?
            .map_err(mapping::engine_error)
    })?;
    Ok((docker, runtime))
}

fn client(endpoint: &Endpoint) -> Result<Docker, bollard::errors::Error> {
    match endpoint {
        Endpoint::Unix(_) | Endpoint::NamedPipe(_) => Docker::connect_with_socket(
            &endpoint.to_string(),
            REQUEST_TIMEOUT_SECS,
            API_DEFAULT_VERSION,
        ),
        Endpoint::Tcp(address) => {
            Docker::connect_with_http(address, REQUEST_TIMEOUT_SECS, API_DEFAULT_VERSION)
        }
    }
}

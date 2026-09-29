//! Bollard clients for copying. Bollard applies one timeout to every request on a
//! client (fussybeaver/bollard#165), and a large image load fails with "Request
//! Timed out" at the normal limit (fussybeaver/bollard#503). So transfers use their
//! own clients with a long limit, and stop through cancellation instead.

use std::time::Duration;

use bollard::{API_DEFAULT_VERSION, Docker};
use captain_core::EngineError;
use tokio::runtime::Handle;

use crate::{Endpoint, mapping};

/// A day: longer than any single copy should take.
const TRANSFER_TIMEOUT_SECS: u64 = 24 * 60 * 60;
/// How long `connect` waits for the engine to answer the version handshake.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// Connects to `endpoint` on `runtime` and agrees on an API version. It blocks, so
/// call it from a plain thread.
pub fn connect(runtime: &Handle, endpoint: &Endpoint) -> Result<Docker, EngineError> {
    runtime.block_on(async {
        let docker = match endpoint {
            Endpoint::Unix(_) | Endpoint::NamedPipe(_) => Docker::connect_with_socket(
                &endpoint.to_string(),
                TRANSFER_TIMEOUT_SECS,
                API_DEFAULT_VERSION,
            ),
            Endpoint::Tcp(address) => {
                Docker::connect_with_http(address, TRANSFER_TIMEOUT_SECS, API_DEFAULT_VERSION)
            }
        }
        .map_err(mapping::engine_error)?;
        tokio::time::timeout(CONNECT_TIMEOUT, docker.negotiate_version())
            .await
            .map_err(|_| EngineError::Unreachable(format!("{endpoint} did not answer")))?
            .map_err(mapping::engine_error)
    })
}

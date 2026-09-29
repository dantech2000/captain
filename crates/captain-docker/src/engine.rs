use std::pin::pin;
use std::time::Duration;

use bollard::query_parameters::ListContainersOptionsBuilder;
use bollard::{API_DEFAULT_VERSION, Docker};
use captain_core::model::{Container, EngineEvent, EngineInfo};
use captain_core::{Engine, EngineError};
use futures::StreamExt;
use futures::future::BoxFuture;
use futures::stream::BoxStream;
use tokio::runtime::Runtime;

use crate::{Endpoint, mapping, runtime};

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
        let runtime = runtime::build().map_err(|err| EngineError::Unreachable(err.to_string()))?;
        let docker = runtime.block_on(async {
            let docker = client(&endpoint).map_err(mapping::engine_error)?;
            tokio::time::timeout(CONNECT_TIMEOUT, docker.negotiate_version())
                .await
                .map_err(|_| EngineError::Unreachable(format!("{endpoint} did not answer")))?
                .map_err(mapping::engine_error)
        })?;
        tracing::info!(%endpoint, "connected to Docker engine");
        Ok(Self {
            docker,
            endpoint,
            runtime,
        })
    }
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

impl Engine for DockerEngine {
    fn info(&self) -> BoxFuture<'static, Result<EngineInfo, EngineError>> {
        let docker = self.docker.clone();
        let endpoint = self.endpoint.clone();
        runtime::spawn(self.runtime.handle(), async move {
            let version = docker.version().await.map_err(mapping::engine_error)?;
            Ok(mapping::engine_info(version, &endpoint))
        })
    }

    fn list_containers(&self) -> BoxFuture<'static, Result<Vec<Container>, EngineError>> {
        let docker = self.docker.clone();
        runtime::spawn(self.runtime.handle(), async move {
            let options = ListContainersOptionsBuilder::default().all(true).build();
            let summaries = docker
                .list_containers(Some(options))
                .await
                .map_err(mapping::engine_error)?;
            Ok(summaries.into_iter().map(mapping::container).collect())
        })
    }

    fn events(&self) -> BoxStream<'static, Result<EngineEvent, EngineError>> {
        let docker = self.docker.clone();
        runtime::forward(self.runtime.handle(), move |tx| async move {
            let mut events = pin!(docker.events(None));
            while let Some(item) = events.next().await {
                let item = item.map(mapping::event).map_err(mapping::engine_error);
                if tx.unbounded_send(item).is_err() {
                    break;
                }
            }
        })
    }
}

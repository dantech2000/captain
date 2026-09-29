//! Builds the connector that the UI runs on a background thread.

use std::sync::Arc;

use captain_core::{Engine, EngineError};
use captain_docker::{DiscoveryInput, DockerEngine, discover};
use captain_ui::Connector;

pub fn docker() -> Connector {
    Box::new(|| {
        let endpoint = discover(&DiscoveryInput::from_env(), |path| path.exists())
            .map_err(|err| EngineError::Unreachable(err.to_string()))?;
        tracing::info!(%endpoint, "connecting");
        let engine: Arc<dyn Engine> = Arc::new(DockerEngine::connect(endpoint)?);
        Ok(engine)
    })
}

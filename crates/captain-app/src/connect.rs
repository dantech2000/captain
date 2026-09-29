//! Builds the connectors that the UI runs on a background thread, and tells the
//! Settings page which engines exist.

use std::sync::Arc;

use captain_core::extension::{ExtensionManager, ExtensionPaths};
use captain_core::{Engine, EngineError, ImageBuilder, ProjectRunner};
use captain_docker::{
    BuildCli, CandidateSource, ComposeCli, DiscoveryInput, DockerEngine, DockerExtensions,
    Endpoint, candidates, discover,
};
use captain_ui::{Connector, DetectedEndpoint, EngineSource};

/// The connector for the main window: the endpoint saved in the settings, or discovery.
pub fn docker(endpoint: Option<String>) -> Connector {
    connector(endpoint)
}

fn connector(endpoint: Option<String>) -> Connector {
    Box::new(move || {
        let endpoint = match endpoint {
            Some(host) => Endpoint::parse(&host).map_err(|err| err.to_string()),
            None => discover(&DiscoveryInput::from_env(), |path| path.exists())
                .map_err(|err| err.to_string()),
        }
        .map_err(EngineError::Unreachable)?;
        tracing::info!(%endpoint, "connecting");
        let engine: Arc<dyn Engine> = Arc::new(DockerEngine::connect(endpoint.clone())?);
        Ok((
            engine,
            compose(&endpoint),
            builder(&endpoint),
            extensions(&endpoint),
        ))
    })
}

/// The `docker compose` CLI for `endpoint`, or `None` when it is missing. See
/// docs/adr/0005-compose-via-cli.md.
fn compose(endpoint: &Endpoint) -> Option<Arc<dyn ProjectRunner>> {
    match ComposeCli::detect(endpoint) {
        Ok(cli) => Some(Arc::new(cli)),
        Err(reason) => {
            tracing::warn!(%reason, "docker compose is not available");
            None
        }
    }
}

/// The `docker buildx` CLI for `endpoint`, or `None` when it is missing. See
/// docs/features/0019-image-build-push-scan.md.
fn builder(endpoint: &Endpoint) -> Option<Arc<dyn ImageBuilder>> {
    match BuildCli::detect(endpoint) {
        Ok(cli) => Some(Arc::new(cli)),
        Err(reason) => {
            tracing::warn!(%reason, "docker buildx is not available");
            None
        }
    }
}

/// The extension manager for `endpoint`, with extensions in `~/.captain/extensions`.
/// See docs/adr/0011-extensions.md.
fn extensions(endpoint: &Endpoint) -> Option<Arc<dyn ExtensionManager>> {
    let paths = ExtensionPaths::in_home(&dirs::home_dir()?);
    match DockerExtensions::connect(endpoint, paths) {
        Ok(manager) => Some(Arc::new(manager)),
        Err(error) => {
            tracing::warn!(%error, "extensions are not available");
            None
        }
    }
}

/// The Docker implementation of the UI's [`EngineSource`].
pub struct DockerSource;

impl EngineSource for DockerSource {
    fn connector(&self, endpoint: Option<&str>) -> Connector {
        connector(endpoint.map(ToString::to_string))
    }

    fn check_endpoint(&self, host: &str) -> Result<(), String> {
        let has_address = host
            .split_once("://")
            .is_some_and(|(_, address)| !address.is_empty());
        match Endpoint::parse(host) {
            Ok(_) if has_address => Ok(()),
            _ => Err(format!(
                "Captain cannot use \"{host}\". Use a unix://, npipe://, tcp://, or http:// URL."
            )),
        }
    }

    fn detected(&self) -> Vec<DetectedEndpoint> {
        candidates(&DiscoveryInput::from_env(), |path| path.exists())
            .into_iter()
            .map(|candidate| DetectedEndpoint {
                source: match (candidate.source, &candidate.endpoint) {
                    (CandidateSource::DockerHost, _) => "DOCKER_HOST".into(),
                    (CandidateSource::Context, _) => "Current context".into(),
                    (CandidateSource::Socket, Endpoint::NamedPipe(_)) => "Named pipe".into(),
                    (CandidateSource::Socket, _) => "Socket".into(),
                },
                host: candidate.endpoint.to_string().into(),
            })
            .collect()
    }
}

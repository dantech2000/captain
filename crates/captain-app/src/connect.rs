//! Builds the connectors that the UI runs on a background thread, and tells the
//! Settings page which engines exist.

use std::sync::Arc;

use captain_core::docker_context::{ContextList, read_contexts};
use captain_core::extension::{ExtensionManager, ExtensionPaths};
use captain_core::ssh::{SshTarget, is_ssh};
use captain_core::{Engine, EngineError, ImageBuilder, ProjectRunner};
use captain_docker::{
    BuildCli, CandidateSource, ComposeCli, DiscoveryInput, DockerEngine, DockerExtensions,
    Endpoint, candidates, discover_host, save_context, use_context,
};
use captain_ui::{Connector, ContextJob, DetectedEndpoint, EngineSource};

/// The connector for the main window: the endpoint saved in the settings, or discovery.
pub fn docker(endpoint: Option<String>) -> Connector {
    connector(endpoint)
}

fn connector(endpoint: Option<String>) -> Connector {
    Box::new(move || {
        let host = match endpoint {
            Some(host) => host,
            None => discover_host(&DiscoveryInput::from_env(), |path| path.exists())
                .map_err(|err| EngineError::Unreachable(err.to_string()))?,
        };
        // An ssh:// host goes through the SSH tunnel; see feature 0026.
        let endpoint = Endpoint::resolve(&host).map_err(EngineError::Unreachable)?;
        tracing::info!(%host, %endpoint, "connecting");
        let engine: Arc<dyn Engine> =
            Arc::new(DockerEngine::connect(endpoint.clone())?.with_label(host.clone()));
        Ok((
            engine,
            compose(&endpoint),
            builder(&endpoint),
            extensions(&endpoint, host),
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
/// Extensions record `host` as their engine. See docs/adr/0011-extensions.md.
fn extensions(endpoint: &Endpoint, host: String) -> Option<Arc<dyn ExtensionManager>> {
    let paths = ExtensionPaths::in_home(&dirs::home_dir()?);
    match DockerExtensions::connect(endpoint, paths) {
        Ok(manager) => Some(Arc::new(manager.with_label(host))),
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
        if is_ssh(host) {
            return SshTarget::parse(host).map(drop).map_err(|err| {
                format!("Captain cannot use \"{host}\": {err}. Use ssh://user@host[:port].")
            });
        }
        let has_address = host
            .split_once("://")
            .is_some_and(|(_, address)| !address.is_empty());
        match Endpoint::parse(host) {
            Ok(_) if has_address => Ok(()),
            _ => Err(format!(
                "Captain cannot use \"{host}\". Use a unix://, npipe://, tcp://, http://, \
                 or ssh:// URL."
            )),
        }
    }

    fn detected(&self) -> Vec<DetectedEndpoint> {
        // The contexts have their own rows, so the current one is left out here.
        candidates(&DiscoveryInput::from_env(), |path| path.exists())
            .into_iter()
            .filter_map(|candidate| {
                let source = match (candidate.source, &candidate.endpoint) {
                    (CandidateSource::DockerHost, _) => "DOCKER_HOST",
                    (CandidateSource::Context, _) => return None,
                    (CandidateSource::Socket, Endpoint::NamedPipe(_)) => "Named pipe",
                    (CandidateSource::Socket, _) => "Socket",
                };
                Some(DetectedEndpoint {
                    source: source.into(),
                    host: candidate.endpoint.to_string().into(),
                })
            })
            .collect()
    }

    fn contexts(&self) -> ContextList {
        DiscoveryInput::from_env()
            .docker_dir()
            .map(|dir| read_contexts(&dir))
            .unwrap_or_default()
    }

    fn save_context(&self, name: &str, description: &str, host: &str) -> ContextJob {
        let (name, description, host) = (name.to_owned(), description.to_owned(), host.to_owned());
        Box::new(move || save_context(&docker_dir()?, &name, &description, &host))
    }

    fn use_context(&self, name: &str) -> ContextJob {
        let name = name.to_owned();
        Box::new(move || use_context(&docker_dir()?, &name))
    }
}

/// The user's Docker CLI config dir, where the contexts live.
fn docker_dir() -> Result<std::path::PathBuf, String> {
    DiscoveryInput::from_env()
        .docker_dir()
        .ok_or_else(|| "Captain cannot find your home folder.".to_string())
}

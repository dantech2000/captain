use bollard::Docker;
use captain_core::extension::{
    BridgeRequest, BridgeStream, ExtensionCandidate, ExtensionManager, ExtensionPaths,
    InstalledExtension, UpdateCheck,
};
use captain_core::{EngineError, EngineFuture};
use tokio::runtime::Runtime;

use super::{bridge, install, update};
use crate::compose::{DockerCli, docker_host};
use crate::{Endpoint, engine, runtime};

/// What every extension step needs: the engine, the `docker` CLI pointed at it, and
/// the extensions folder.
#[derive(Debug, Clone)]
pub(super) struct Context {
    pub docker: Docker,
    /// `None` when the CLI is missing. Backends and `docker.cli.exec` need it.
    pub cli: Option<DockerCli>,
    /// The engine as a `DOCKER_HOST` value.
    pub host: String,
    pub paths: ExtensionPaths,
}

impl Context {
    pub fn cli(&self) -> Result<&DockerCli, EngineError> {
        self.cli
            .as_ref()
            .ok_or_else(|| EngineError::Api("the docker CLI is not installed".into()))
    }

    /// A `docker` command pointed at the engine.
    pub fn docker_command(&self) -> Result<std::process::Command, EngineError> {
        let mut command = self.cli()?.command();
        command
            .env("DOCKER_HOST", &self.host)
            // DOCKER_HOST and DOCKER_CONTEXT together make the CLI refuse to run.
            .env_remove("DOCKER_CONTEXT");
        Ok(command)
    }
}

/// The Docker implementation of [`ExtensionManager`].
pub struct DockerExtensions {
    context: Context,
    runtime: Runtime,
}

impl DockerExtensions {
    /// Connects to `endpoint`. Blocks for up to a few seconds; call it from a
    /// background thread.
    pub fn connect(endpoint: &Endpoint, paths: ExtensionPaths) -> Result<Self, EngineError> {
        let (docker, runtime) = engine::connect(endpoint)?;
        let context = Context {
            docker,
            cli: DockerCli::find(),
            host: docker_host(endpoint),
            paths,
        };
        Ok(Self { context, runtime })
    }
}

impl ExtensionManager for DockerExtensions {
    fn list(&self) -> EngineFuture<Vec<InstalledExtension>> {
        let paths = self.context.paths.clone();
        runtime::spawn(self.runtime.handle(), async move {
            tokio::task::spawn_blocking(move || install::list(&paths))
                .await
                .map_err(|error| EngineError::Api(error.to_string()))?
        })
    }

    fn prepare(&self, reference: &str) -> EngineFuture<ExtensionCandidate> {
        let context = self.context.clone();
        let reference = reference.to_string();
        runtime::spawn(self.runtime.handle(), async move {
            install::prepare(&context, &reference).await
        })
    }

    fn install(&self, candidate: ExtensionCandidate) -> EngineFuture<InstalledExtension> {
        let context = self.context.clone();
        runtime::spawn(self.runtime.handle(), async move {
            install::install(&context, candidate).await
        })
    }

    fn check_update(
        &self,
        extension: InstalledExtension,
        tag: String,
    ) -> EngineFuture<UpdateCheck> {
        let context = self.context.clone();
        runtime::spawn(self.runtime.handle(), async move {
            update::check(&context, extension, &tag).await
        })
    }

    fn update(
        &self,
        extension: InstalledExtension,
        candidate: ExtensionCandidate,
    ) -> EngineFuture<InstalledExtension> {
        let context = self.context.clone();
        runtime::spawn(self.runtime.handle(), async move {
            update::apply(&context, extension, candidate).await
        })
    }

    fn remove(&self, extension: InstalledExtension) -> EngineFuture<()> {
        let context = self.context.clone();
        runtime::spawn(self.runtime.handle(), async move {
            install::remove(&context, &extension).await
        })
    }

    fn paths(&self) -> &ExtensionPaths {
        &self.context.paths
    }

    fn call(&self, extension: &InstalledExtension, request: BridgeRequest) -> BridgeStream {
        bridge::call(
            self.runtime.handle(),
            self.context.clone(),
            extension.clone(),
            request,
        )
    }
}

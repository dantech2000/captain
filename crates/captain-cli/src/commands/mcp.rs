//! `captain mcp`: Captain's MCP server on stdin and stdout, for AI agents. stdout
//! carries only protocol messages; logs go to stderr.

use std::sync::Arc;

use anyhow::Result;
use captain_core::Engine;
use captain_core::settings::EngineChoice;
use captain_docker::{DiscoveryInput, DockerEngine, Endpoint, discover_host};
use rmcp::ServiceExt;
use tracing_subscriber::EnvFilter;

use crate::context::Context;
use crate::mcp::{CaptainServer, Connect, Source};

pub fn run(context: &Context) -> Result<()> {
    // `CAPTAIN_LOG=debug` shows more; stdout must stay clean for the protocol.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .with_env_filter(EnvFilter::try_from_env("CAPTAIN_LOG").unwrap_or_else(|_| "warn".into()))
        .init();
    let source = source(context);
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async {
        let server = CaptainServer::new(source)
            .serve(rmcp::transport::stdio())
            .await?;
        server.waiting().await?;
        Ok(())
    })
}

/// The engine the settings choose, as the app connects to it at launch: Captain
/// Engine's socket, or the saved endpoint, or discovery (`DOCKER_HOST`, the current
/// Docker context, then known sockets).
fn source(context: &Context) -> Source {
    let settings = context.load_or_default();
    let choice = settings.engine_choice(context.machine.captain_available);
    let host = (choice == EngineChoice::Captain).then(|| context.host(&settings));
    let saved = settings.engine_endpoint.clone();
    let socket = host.clone();
    let connect: Connect = Arc::new(move || {
        let address = match (&socket, &saved) {
            (Some(host), _) => host
                .endpoint()
                .ok_or("Captain Engine has no socket on this computer")?,
            (None, Some(saved)) => saved.clone(),
            (None, None) => discover_host(&DiscoveryInput::from_env(), |path| path.exists())
                .map_err(|error| error.to_string())?,
        };
        let endpoint = Endpoint::resolve(&address)?;
        let engine = DockerEngine::connect(endpoint).map_err(|error| error.to_string())?;
        Ok(Arc::new(engine.with_label(address)) as Arc<dyn Engine>)
    });
    Source::new(
        choice.label(),
        host,
        connect,
        choice == EngineChoice::Captain && settings.kubernetes.enabled,
    )
}

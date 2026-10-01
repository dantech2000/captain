//! `captain mcp`: Captain's MCP server on stdin and stdout, for AI agents. stdout
//! carries only protocol messages; logs go to stderr.

use std::sync::Arc;

use anyhow::Result;
use captain_core::agent_tools::activity_path;
use captain_core::settings::{EngineChoice, Settings};
use captain_core::{Engine, ProjectRunner};
use captain_docker::{ComposeCli, DiscoveryInput, DockerEngine, Endpoint, discover_host};
use rmcp::ServiceExt;
use tracing_subscriber::EnvFilter;

use crate::context::Context;
use crate::mcp::{CaptainServer, Connect, Connected, ReadSettings, ReadShowExtensions, Source};

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
        let engine = DockerEngine::connect(endpoint.clone()).map_err(|error| error.to_string())?;
        // The app runs project actions and tasks through the same CLI.
        let runner = ComposeCli::detect(&endpoint)
            .map_err(|reason| tracing::warn!(%reason, "docker compose is not available"))
            .ok()
            .map(|cli| Arc::new(cli) as Arc<dyn ProjectRunner>);
        Ok(Connected {
            engine: Arc::new(engine.with_label(address)) as Arc<dyn Engine>,
            runner,
        })
    });
    // Read on every call, so a change in Captain's Settings applies at once. A file
    // that cannot be read turns the tools off.
    let path = context.settings_path.clone();
    let read: ReadSettings = Arc::new(move || {
        Settings::load(&path)
            .map(|settings| settings.agent_tools)
            .unwrap_or_default()
    });
    let path = context.settings_path.clone();
    let show_extensions: ReadShowExtensions = Arc::new(move || {
        Settings::load(&path).is_ok_and(|settings| settings.show_extension_containers)
    });
    let source = Source::new(
        choice.label(),
        host,
        connect,
        choice == EngineChoice::Captain && settings.kubernetes.enabled,
    )
    .with_settings(read)
    .with_extension_containers(show_extensions);
    match dirs::home_dir() {
        Some(home) => source.with_activity(activity_path(&home)),
        None => source,
    }
}

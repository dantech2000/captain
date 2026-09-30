//! `captain status`: the engine state, the socket, and the versions.

use anyhow::Result;
use captain_core::diagnostics::ToolProbe;
use captain_core::settings::EngineChoice;
use captain_core::{ContainerApi, HostStatus};
use captain_docker::{DockerEngine, Endpoint};
use futures::executor::block_on;
use serde::Serialize;

use crate::context::Context;

#[derive(Serialize)]
struct Status {
    engine: EngineChoice,
    state: &'static str,
    /// Why the engine is not installed or failed.
    detail: Option<String>,
    socket: Option<String>,
    app_running: bool,
    versions: Versions,
}

#[derive(Serialize)]
struct Versions {
    captain: &'static str,
    lima: Option<String>,
    docker: Option<String>,
}

pub fn run(context: &Context, json: bool) -> Result<()> {
    let settings = context.load_or_default();
    let host = context.host(&settings);
    let status = block_on(host.status())?;
    let socket = host.endpoint();
    let docker = socket
        .as_deref()
        .filter(|_| status.is_running())
        .and_then(docker_version);
    let report = Status {
        engine: settings.engine_choice(context.machine.captain_available),
        state: status.key(),
        detail: status.detail().map(ToString::to_string),
        socket,
        app_running: context.app_running(),
        versions: Versions {
            captain: env!("CARGO_PKG_VERSION"),
            lima: lima_version(),
            docker,
        },
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_text(&report, &status);
    }
    Ok(())
}

fn print_text(report: &Status, status: &HostStatus) {
    let none = || "-".to_string();
    let state = match &report.detail {
        Some(detail) => format!("{} ({detail})", status.label()),
        None => status.label().to_string(),
    };
    let app = if report.app_running {
        "running"
    } else {
        "not running"
    };
    println!("Engine choice:   {}", report.engine.label());
    println!("Captain Engine:  {state}");
    println!(
        "Socket:          {}",
        report.socket.clone().unwrap_or_else(none)
    );
    println!("Captain app:     {app}");
    println!("Captain:         {}", report.versions.captain);
    println!(
        "Lima:            {}",
        report.versions.lima.clone().unwrap_or_else(none)
    );
    println!(
        "Docker:          {}",
        report.versions.docker.clone().unwrap_or_else(none)
    );
}

fn lima_version() -> Option<String> {
    if !cfg!(target_os = "macos") {
        return None;
    }
    match captain_host::probe::lima_version() {
        ToolProbe::Found(version) => Some(version),
        ToolProbe::Missing | ToolProbe::Broken(_) => None,
    }
}

/// The version the engine reports, or `None` when it does not answer.
fn docker_version(socket: &str) -> Option<String> {
    let engine = DockerEngine::connect(Endpoint::parse(socket).ok()?).ok()?;
    block_on(engine.info()).ok().map(|info| info.version)
}

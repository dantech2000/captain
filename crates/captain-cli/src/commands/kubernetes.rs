//! `captain kubernetes enable|disable|status|reset`. The work is in `captain-host`,
//! the same code the Kubernetes card in Settings uses. See
//! docs/features/0024-kubernetes.md.

mod status;

use std::sync::Arc;

use anyhow::{Context as _, Result, bail};
use captain_core::kubernetes::{K3sVersion, KubernetesHost, KubernetesSettings};
use captain_core::{EngineHost, HostStream};
use futures::executor::{block_on, block_on_stream};

use super::snapshot::confirm;
use crate::cli::KubernetesCommand;
use crate::context::Context;

pub fn run(context: &Context, command: KubernetesCommand) -> Result<()> {
    match command {
        KubernetesCommand::Enable {
            version,
            port,
            traefik,
        } => enable(context, version, port, traefik),
        KubernetesCommand::Disable => disable(context),
        KubernetesCommand::Status { json } => status::run(context, json),
        KubernetesCommand::Reset { yes } => reset(context, yes),
    }
}

/// Captain Engine and its cluster, with the saved settings.
fn cluster(context: &Context) -> Result<(Arc<dyn EngineHost>, Arc<dyn KubernetesHost>)> {
    let host = context.host(&context.load_or_default());
    let kubernetes = host
        .kubernetes()
        .context("Kubernetes needs Captain Engine, which runs on macOS only")?;
    Ok((host, kubernetes))
}

fn enable(
    context: &Context,
    version: Option<String>,
    port: Option<u16>,
    traefik: Option<bool>,
) -> Result<()> {
    let (host, kubernetes) = cluster(context)?;
    let mut wanted = KubernetesSettings {
        enabled: true,
        ..context.load()?.kubernetes
    };
    if let Some(version) = version {
        version.parse::<K3sVersion>().map_err(anyhow::Error::msg)?;
        wanted.version = Some(version);
    }
    if wanted.version.is_none() {
        let list = block_on(kubernetes.versions(false))?;
        let stable = list
            .stable()
            .context("cannot find the stable k3s version; check the network")?;
        wanted.version = Some(stable.to_string());
    }
    wanted.port = port.unwrap_or(wanted.port);
    wanted.traefik = traefik.unwrap_or(wanted.traefik);
    save(context, &wanted)?;
    host.set_kubernetes(wanted.clone());
    let version = wanted.version.clone().unwrap_or_default();
    if !engine_running(&host)? {
        println!("Kubernetes {version} starts with Captain Engine. Run `captain start`.");
        return Ok(());
    }
    print_stream(kubernetes.enable(wanted))
}

fn disable(context: &Context) -> Result<()> {
    let (host, kubernetes) = cluster(context)?;
    let wanted = KubernetesSettings {
        enabled: false,
        ..context.load()?.kubernetes
    };
    save(context, &wanted)?;
    host.set_kubernetes(wanted);
    if engine_running(&host)? {
        block_on(kubernetes.disable())?;
    }
    println!("Kubernetes is off. Its state stays for the next `captain kubernetes enable`.");
    Ok(())
}

fn reset(context: &Context, yes: bool) -> Result<()> {
    let (host, kubernetes) = cluster(context)?;
    if !engine_running(&host)? {
        bail!("Captain Engine is not running. Run `captain start` first.");
    }
    confirm(
        "Reset Kubernetes? This deletes all workloads and the cluster state. Images stay.",
        yes,
    )?;
    print_stream(kubernetes.reset())
}

/// Saves the Kubernetes settings. Like `captain set`, it refuses while the app runs,
/// because the app writes the whole settings file on each change.
fn save(context: &Context, kubernetes: &KubernetesSettings) -> Result<()> {
    if context.app_running() {
        bail!("Captain is running. Change Kubernetes in Settings, or quit Captain first.");
    }
    let mut settings = context.load()?;
    settings.kubernetes = kubernetes.clone();
    settings.save(&context.settings_path)?;
    Ok(())
}

fn engine_running(host: &Arc<dyn EngineHost>) -> Result<bool> {
    Ok(block_on(host.status())?.is_running())
}

fn print_stream(stream: HostStream<String>) -> Result<()> {
    for line in block_on_stream(stream) {
        println!("{}", line?);
    }
    Ok(())
}

//! `captain kubernetes status`: the saved settings and the cluster's state.

use anyhow::Result;
use captain_core::kubernetes::{KubernetesStatus, load_contexts, user_kubeconfig_paths};
use futures::executor::block_on;
use serde::Serialize;

use super::cluster;
use crate::context::Context;

#[derive(Serialize)]
struct Status {
    enabled: bool,
    version: Option<String>,
    port: u16,
    traefik: bool,
    state: &'static str,
    /// The version k3s runs, or why it failed.
    detail: Option<String>,
    kubeconfig: String,
    /// True if the user's kubeconfig has the `captain` context.
    context: bool,
}

pub fn run(context: &Context, json: bool) -> Result<()> {
    let settings = context.load_or_default().kubernetes;
    let (host, kubernetes) = cluster(context)?;
    let state = if block_on(host.status())?.is_running() {
        block_on(kubernetes.status())?
    } else {
        KubernetesStatus::Off
    };
    let detail = match &state {
        KubernetesStatus::Running { version } => Some(version.clone()),
        KubernetesStatus::Failed(why) => Some(why.clone()),
        _ => None,
    };
    let report = Status {
        enabled: settings.enabled,
        version: settings.version,
        port: settings.port,
        traefik: settings.traefik,
        state: state.label(),
        detail,
        kubeconfig: kubernetes.kubeconfig().display().to_string(),
        context: load_contexts(&user_kubeconfig_paths())
            .names
            .iter()
            .any(|name| name == "captain"),
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }
    let on = if report.enabled { "on" } else { "off" };
    println!("Kubernetes:  {on}");
    println!("State:       {}", report.state);
    if let Some(detail) = &report.detail {
        println!("Detail:      {detail}");
    }
    let version = report.version.as_deref().unwrap_or("not picked yet");
    println!("Version:     {version}");
    println!("API:         https://127.0.0.1:{}", report.port);
    println!("Traefik:     {}", if report.traefik { "on" } else { "off" });
    println!("Kubeconfig:  {}", report.kubeconfig);
    let context = if report.context { "yes" } else { "no" };
    println!("Context:     captain in your kubeconfig: {context}");
    Ok(())
}

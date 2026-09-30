//! The steps behind each Kubernetes action on a running VM. Each blocks, so it runs
//! on its own thread. See ADR 0010.

use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use captain_core::HostError;
use captain_core::kubernetes::{
    K3sAssets, K3sVersion, KubernetesSettings, KubernetesStatus, captain_config, cluster_ca,
    install_captain, user_kubeconfig_paths, write_config,
};
use serde_json::Value;

use super::guest;
use crate::cancel::Cancel;
use crate::k3s;
use crate::lima::limactl::Limactl;
use crate::lima::paths::LimaPaths;

/// How long the first start of k3s may take before Captain gives up waiting.
const READY_TIMEOUT: Duration = Duration::from_secs(240);
/// How long Lima may take to forward the API port to this Mac.
const HOST_PORT_TIMEOUT: Duration = Duration::from_secs(60);
/// How long a status check in the VM may take.
const QUICK_TIMEOUT: Duration = Duration::from_secs(30);

/// Downloads the version if needed, installs and starts k3s, waits for its API,
/// and writes the kubeconfig files. Returns the version.
pub fn install(
    limactl: &Limactl,
    paths: &LimaPaths,
    settings: &KubernetesSettings,
    cancel: &Cancel,
    sink: &mut dyn FnMut(String),
) -> Result<K3sVersion, HostError> {
    let version = wanted_version(paths, settings, cancel)?;
    let assets = K3sAssets::for_arch(std::env::consts::ARCH)
        .ok_or_else(|| HostError("Kubernetes needs an Arm or Intel Mac.".into()))?;
    let instance = &paths.instance;
    let run = |args: Vec<String>| limactl.run(&args, None, cancel);
    let installed = run(guest::version_args(instance))?;
    if let Ok(installed) = installed.trim().parse::<K3sVersion>()
        && version < installed
    {
        return Err(HostError(format!(
            "Kubernetes {installed} is installed. Going back to {version} needs Reset Kubernetes, which deletes the workloads."
        )));
    }
    // A running k3s holds its own port through Lima, so only a new port is checked.
    let current = run(guest::port_args(instance))?;
    if current.trim().parse::<u16>().ok() != Some(settings.port)
        || !status(limactl, paths).is_running()
    {
        check_port(settings.port)?;
    }
    let folder = k3s::ensure(&paths.k3s_cache(), &version, &assets, cancel, sink)?;
    sink(format!("Installing Kubernetes {version}."));
    run(guest::install_args(
        instance,
        &folder.display().to_string(),
        assets.binary,
        assets.images,
        settings,
        version.as_str(),
    ))?;
    sink("Waiting for the Kubernetes API.".into());
    wait_ready(limactl, instance, cancel)?;
    let captain = captain_config(&run(guest::kubeconfig_args(instance))?, settings.port)
        .map_err(HostError)?;
    wait_host_port(settings.port, &trust(paths, &captain)?, cancel)?;
    if !settings.traefik
        && let Err(error) = run(guest::remove_traefik_args(instance))
    {
        tracing::warn!(%error, "cannot remove Traefik");
    }
    cancel.check()?;
    write_kubeconfig(paths, &captain)?;
    sink(format!("Kubernetes {version} is running."));
    Ok(version)
}

/// The saved version, or the stable one when none is saved yet.
fn wanted_version(
    paths: &LimaPaths,
    settings: &KubernetesSettings,
    cancel: &Cancel,
) -> Result<K3sVersion, HostError> {
    if let Some(version) = &settings.version {
        return version.parse().map_err(HostError);
    }
    let list = k3s::list(
        &paths.k3s_versions_file(),
        &paths.k3s_cache(),
        false,
        cancel,
    );
    cancel.check()?;
    list.stable().cloned().ok_or_else(|| {
        HostError(
            "Captain cannot find the stable Kubernetes version. Check the network, then try again."
                .into(),
        )
    })
}

/// Fails when another program listens on `port` on this Mac, because Lima could
/// not forward the API there.
fn check_port(port: u16) -> Result<(), HostError> {
    if port == 0 {
        return Err(HostError("The Kubernetes port must be 1 to 65535.".into()));
    }
    TcpListener::bind(("127.0.0.1", port))
        .map(drop)
        .map_err(|_| {
            HostError(format!(
                "Port {port} is in use on this Mac. Set another kubernetes.port in the settings file."
            ))
        })
}

fn wait_ready(limactl: &Limactl, instance: &str, cancel: &Cancel) -> Result<(), HostError> {
    let deadline = Instant::now() + READY_TIMEOUT;
    while Instant::now() < deadline {
        if limactl
            .run(&guest::ready_args(instance), None, cancel)
            .is_ok_and(|out| out.trim() == "ready")
        {
            return Ok(());
        }
        cancel.check()?;
        std::thread::sleep(Duration::from_secs(2));
    }
    Err(HostError(
        "Kubernetes did not answer in time. See `journalctl -u k3s` in the VM.".into(),
    ))
}

/// Saves k3s's certificate authority for the host port check, and returns its file.
fn trust(paths: &LimaPaths, captain: &Value) -> Result<PathBuf, HostError> {
    let file = paths.kubernetes_ca();
    let ca = cluster_ca(captain).map_err(HostError)?;
    std::fs::write(&file, ca)
        .map_err(|error| HostError(format!("Cannot write {}: {error}", file.display())))?;
    Ok(file)
}

/// Waits until k3s answers on `127.0.0.1:port` on this Mac, through Lima's port
/// forward, with the certificate of its own authority `ca`, so the kubeconfig
/// does not point at another program or cluster.
fn wait_host_port(port: u16, ca: &Path, cancel: &Cancel) -> Result<(), HostError> {
    let deadline = Instant::now() + HOST_PORT_TIMEOUT;
    while Instant::now() < deadline {
        if k3s::ping(port, ca, cancel) {
            return Ok(());
        }
        cancel.check()?;
        std::thread::sleep(Duration::from_secs(1));
    }
    Err(HostError(format!(
        "Kubernetes runs in the VM, but port {port} on this Mac does not reach it. Another program may use that port. Set another kubernetes.port in the settings file."
    )))
}

/// Writes `~/.captain/kubeconfig` and merges the `captain` context into the user's
/// kubeconfig, with a backup first.
fn write_kubeconfig(paths: &LimaPaths, captain: &Value) -> Result<(), HostError> {
    write_config(&paths.kubeconfig(), captain).map_err(HostError)?;
    let target = install_captain(&user_kubeconfig_paths(), captain).map_err(HostError)?;
    tracing::info!(file = %target.display(), "merged the captain context");
    Ok(())
}

pub fn status(limactl: &Limactl, paths: &LimaPaths) -> KubernetesStatus {
    match limactl.output_within(&guest::status_args(&paths.instance), QUICK_TIMEOUT) {
        Ok(output) => guest::parse_status(&output),
        Err(error) => KubernetesStatus::Failed(error.0),
    }
}

pub fn disable(limactl: &Limactl, paths: &LimaPaths, cancel: &Cancel) -> Result<(), HostError> {
    limactl
        .run(&guest::disable_args(&paths.instance), None, cancel)
        .map(drop)
}

pub fn reset(limactl: &Limactl, paths: &LimaPaths, cancel: &Cancel) -> Result<(), HostError> {
    limactl
        .run(&guest::reset_args(&paths.instance), None, cancel)
        .map(drop)
}

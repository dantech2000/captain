//! The blocking work behind each [`LimaHost`](super::LimaHost) action. Each runs on
//! its own thread.

use std::time::Duration;

use captain_core::{HostError, HostResources, HostStatus};

use super::{Inner, Phase, Seen, daemon_steps, engine_lock, kube_steps, lock};
use crate::lima::args;
use crate::lima::instance::{LimaInstance, find_instance};
use crate::lima::limactl::Limactl;
use crate::lima::snapshot::check_finished;
use crate::lima::template;

/// How long `limactl list` and the other status checks may take.
pub const QUICK_TIMEOUT: Duration = Duration::from_secs(30);

pub fn status(inner: &Inner) -> HostStatus {
    match inner.phase() {
        Phase::Starting => return HostStatus::Starting,
        Phase::Stopping => return HostStatus::Stopping,
        Phase::Idle => {}
    }
    if let Some(status) = engine_lock::other_process(inner) {
        return status;
    }
    if let Err(message) = inner.paths.check_socket_paths() {
        return HostStatus::Failed(message);
    }
    let limactl = match inner.limactl() {
        Ok(limactl) => limactl,
        Err(reason) => return HostStatus::NotInstalled(reason),
    };
    let status = match find(inner, &limactl) {
        Ok(Some(instance)) => instance.host_status(),
        Ok(None) => HostStatus::NotCreated,
        Err(error) => HostStatus::Failed(error.0),
    };
    daemon_steps::track(inner, &limactl, &status);
    status
}

/// Creates the instance if needed, applies changed resources, and starts it.
pub fn start(inner: &Inner, sink: &mut dyn FnMut(String)) -> Result<(), HostError> {
    let _guard = inner.begin(Phase::Starting)?;
    let _lock = engine_lock::acquire(inner, true)?;
    inner.cancel.reset();
    check_finished(&inner.paths.instance_dir()).map_err(HostError)?;
    inner.paths.check_socket_paths().map_err(HostError)?;
    let limactl = inner.limactl().map_err(HostError)?;
    let wanted = *lock(&inner.resources);

    match find(inner, &limactl)? {
        None => create(inner, &limactl, &wanted, sink)?,
        Some(instance) if instance.status == "Running" => {
            sink("Captain Engine is already running.".into());
            return Ok(());
        }
        Some(instance) => {
            if let Some(edit) = args::edit(&inner.paths.instance, &instance.resources(), &wanted) {
                sink("Applying the new resources.".into());
                limactl.stream(&edit, &inner.cancel, sink)?;
            }
        }
    }
    inner.cancel.check()?;
    sink("Starting Captain Engine.".into());
    limactl.stream(&args::start(&inner.paths.instance), &inner.cancel, sink)?;
    let socket = inner.paths.docker_socket();
    if !socket.exists() {
        return Err(HostError(format!(
            "Captain Engine started, but the Docker socket is missing at {}.",
            socket.display()
        )));
    }
    // Learn the resources it started with, for "Restart to apply".
    find(inner, &limactl).ok();
    // Each step below runs under `inner.cancel`, so a stop ends it at once.
    daemon_steps::apply(inner, &limactl, sink)?;
    kube_steps::on_start(inner, &limactl, sink);
    inner.cancel.check()?;
    sink("Captain Engine is running.".into());
    Ok(())
}

fn create(
    inner: &Inner,
    limactl: &Limactl,
    resources: &HostResources,
    sink: &mut dyn FnMut(String),
) -> Result<(), HostError> {
    let file = inner.paths.template_file();
    let yaml = template::render(resources, inner.rosetta);
    let written = file
        .parent()
        .map_or(Ok(()), std::fs::create_dir_all)
        .and_then(|()| std::fs::write(&file, yaml));
    written.map_err(|error| HostError(format!("Cannot write {}: {error}", file.display())))?;
    // The template mounts /tmp/lima; Lima warns when it does not exist.
    std::fs::create_dir_all("/tmp/lima").ok();
    sink("Setting up Captain Engine. The first start downloads about 600 MB.".into());
    limactl.stream(
        &args::create(&inner.paths.instance, &file),
        &inner.cancel,
        sink,
    )
}

/// Stops the instance. A start that is running is cancelled first.
pub fn stop(inner: &Inner) -> Result<(), HostError> {
    cancel_start(inner);
    let _guard = inner.begin(Phase::Stopping)?;
    // The start has ended, so the stop's own commands may run.
    inner.cancel.reset();
    let _lock = engine_lock::acquire(inner, false)?;
    let limactl = inner.limactl().map_err(HostError)?;
    match find(inner, &limactl)? {
        None => Ok(()),
        Some(instance) if instance.status == "Stopped" => Ok(()),
        Some(_) => {
            let mut ignore = |_| {};
            limactl
                .stream(
                    &args::stop(&inner.paths.instance, false),
                    &inner.cancel,
                    &mut ignore,
                )
                .or_else(|error| {
                    tracing::warn!(%error, "limactl stop failed; forcing it");
                    limactl.stream(
                        &args::stop(&inner.paths.instance, true),
                        &inner.cancel,
                        &mut ignore,
                    )
                })
        }
    }
}

/// Applies the resources now if the instance exists and is stopped. Otherwise the
/// next start applies them.
pub fn apply_resources(inner: &Inner) -> Result<(), HostError> {
    if inner.phase() != Phase::Idle {
        return Ok(());
    }
    let Ok(limactl) = inner.limactl() else {
        return Ok(());
    };
    let Some(_lock) = engine_lock::try_acquire(inner, engine_lock::EDITING)? else {
        return Ok(());
    };
    let wanted = *lock(&inner.resources);
    match find(inner, &limactl)? {
        Some(instance) if instance.status == "Stopped" => {
            match args::edit(&inner.paths.instance, &instance.resources(), &wanted) {
                Some(edit) => {
                    limactl.output(&edit)?;
                    // The disk floor follows the grown disk.
                    find(inner, &limactl).map(drop)
                }
                None => Ok(()),
            }
        }
        _ => Ok(()),
    }
}

/// Deletes the instance and the template.
pub fn reset(inner: &Inner) -> Result<(), HostError> {
    cancel_start(inner);
    let _guard = inner.begin(Phase::Stopping)?;
    let _lock = engine_lock::acquire(inner, false)?;
    let limactl = inner.limactl().map_err(HostError)?;
    if find(inner, &limactl)?.is_some() {
        limactl.output(&args::delete(&inner.paths.instance))?;
    }
    std::fs::remove_file(inner.paths.template_file()).ok();
    Ok(())
}

/// Kills a running start and waits for it to end, so the stop that follows is not
/// refused. Each start step runs under `inner.cancel` or a short timeout, so the
/// wait is short.
fn cancel_start(inner: &Inner) {
    if inner.phase() != Phase::Starting {
        return;
    }
    inner.cancel.cancel();
    while inner.phase() == Phase::Starting {
        std::thread::sleep(Duration::from_millis(100));
        inner.cancel.kill();
    }
}

/// The instance as `limactl list` reports it. It also records what it saw, for
/// `running_resources` and `current_disk`.
pub fn find(inner: &Inner, limactl: &Limactl) -> Result<Option<LimaInstance>, HostError> {
    let json = limactl.output_within(&args::list(), QUICK_TIMEOUT)?;
    let instance = find_instance(&json, &inner.paths.instance);
    *lock(&inner.seen) = instance.as_ref().map(|instance| Seen {
        running: instance.status == "Running",
        resources: instance.resources(),
    });
    Ok(instance)
}

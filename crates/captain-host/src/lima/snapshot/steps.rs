//! The blocking work behind each [`LimaSnapshots`](super::LimaSnapshots) call. Create,
//! restore, and delete hold the engine lock, so no start runs meanwhile.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use captain_core::HostError;
use captain_core::snapshot::{Snapshot, SnapshotList, SnapshotMetadata, check_name, check_space};

use super::copy::{allocated, copy_file, same_volume};
use super::plan::{DISK, files, restorable, source_of};
use super::swap::swap;
use super::{folder, plan};
use crate::LimaHost;
use crate::lima::instance::LimaInstance;
use crate::probe::free_space;

pub fn list(host: &LimaHost) -> SnapshotList {
    let root = host.paths().snapshots_dir();
    SnapshotList {
        snapshots: folder::read_all(&root),
        free_bytes: free_space(existing_parent(&root)),
        engine_disk_bytes: Some(host.paths().instance_dir().join(DISK))
            .filter(|disk| disk.exists())
            .map(|disk| allocated(&source_of(&disk))),
    }
}

pub fn create(host: &LimaHost, name: &str, description: &str) -> Result<Snapshot, HostError> {
    let _lock = host.lock_for_snapshot()?;
    let instance = match host.instance()? {
        Some(instance) if instance.status == "Stopped" => instance,
        Some(_) => return Err(must_stop()),
        None => return Err(HostError("Set up Captain Engine first.".into())),
    };
    let paths = host.paths();
    let root = paths.snapshots_dir();
    std::fs::create_dir_all(&root).map_err(|error| io_error(&root, error))?;
    folder::remove_incomplete(&root);
    let existing = folder::read_all(&root);
    check_name(name, existing.iter().map(|s| s.metadata.name.as_str())).map_err(HostError)?;

    let disk = source_of(&paths.instance_dir().join(DISK));
    let disk_allocated = allocated(&disk);
    let clone = same_volume(&disk, &root);
    check_space(free_space(&root), disk_allocated, clone).map_err(HostError)?;

    let id = folder::new_id();
    let dir = root.join(&id);
    // The saved daemon and Kubernetes settings, not what the copied disk has: each
    // start applies the saved settings, so a restore that adopts them gets the
    // engine the user had set up.
    let metadata = SnapshotMetadata {
        disk_allocated,
        daemon: Some(host.daemon_settings()),
        kubernetes: Some(host.kubernetes_settings()),
        ..metadata(&paths.instance_dir(), &instance, name, description)
    };
    let written = std::fs::create_dir(&dir)
        .map_err(|error| io_error(&dir, error))
        .and_then(|()| copy_files(paths, &dir))
        .and_then(|()| folder::finish(&dir, &metadata).map_err(|error| io_error(&dir, error)));
    if let Err(error) = written {
        std::fs::remove_dir_all(&dir).ok();
        return Err(error);
    }
    tracing::info!(%id, name, "created a snapshot");
    Ok(Snapshot { id, metadata })
}

pub fn restore(host: &LimaHost, id: &str) -> Result<Snapshot, HostError> {
    let _lock = host.lock_for_snapshot()?;
    let snapshot = find(host, id)?;
    if host
        .instance()?
        .is_some_and(|instance| instance.status != "Stopped")
    {
        return Err(must_stop());
    }
    let paths = host.paths();
    let dir = paths.snapshots_dir().join(&snapshot.id);
    let instance_dir = paths.instance_dir();
    std::fs::create_dir_all(&instance_dir).map_err(|error| io_error(&instance_dir, error))?;
    let clone = same_volume(&dir, &instance_dir);
    check_space(
        free_space(&instance_dir),
        snapshot.metadata.disk_allocated,
        clone,
    )
    .map_err(HostError)?;

    let replace = restorable(files(paths), &dir).map_err(HostError)?;
    // An old instance's `disk` links to `diffdisk`; the restored `disk` is a file.
    // `diffdisk` goes only once the swap put that file in place of the link.
    let live_disk = instance_dir.join(DISK);
    let legacy = source_of(&live_disk);
    swap(&replace, &instance_dir, &copy_file).map_err(HostError)?;
    if legacy != live_disk && live_disk.is_file() && !live_disk.is_symlink() {
        std::fs::remove_file(&legacy).ok();
    }
    tracing::info!(id, name = %snapshot.metadata.name, "restored a snapshot");
    Ok(snapshot)
}

pub fn delete(host: &LimaHost, id: &str) -> Result<(), HostError> {
    let _lock = host.lock_for_snapshot()?;
    let snapshot = find(host, id)?;
    let root = host.paths().snapshots_dir();
    let dir = root.join(&snapshot.id);
    std::fs::remove_dir_all(&dir).map_err(|error| io_error(&dir, error))?;
    folder::remove_incomplete(&root);
    tracing::info!(id, "deleted a snapshot");
    Ok(())
}

pub fn edit(
    host: &LimaHost,
    id: &str,
    name: &str,
    description: &str,
) -> Result<Snapshot, HostError> {
    let _lock = host.lock_for_snapshot()?;
    let snapshot =
        folder::edit(&host.paths().snapshots_dir(), id, name, description).map_err(HostError)?;
    tracing::info!(id, name, "edited a snapshot");
    Ok(snapshot)
}

fn copy_files(paths: &crate::LimaPaths, dir: &Path) -> Result<(), HostError> {
    for file in plan::files(paths) {
        let source = source_of(&file.live);
        if !source.exists() {
            if file.required {
                return Err(HostError(format!("{} is missing.", source.display())));
            }
            continue;
        }
        copy_file(&source, &dir.join(file.name)).map_err(|error| io_error(&source, error))?;
    }
    Ok(())
}

fn metadata(
    instance_dir: &Path,
    instance: &LimaInstance,
    name: &str,
    description: &str,
) -> SnapshotMetadata {
    let lima_version = std::fs::read_to_string(instance_dir.join("lima-version"))
        .map(|text| text.trim().to_string())
        .unwrap_or_default();
    SnapshotMetadata {
        name: name.into(),
        description: description.trim().into(),
        created: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_secs()),
        captain_version: env!("CARGO_PKG_VERSION").into(),
        lima_version,
        disk_allocated: 0,
        resources: instance.resources(),
        daemon: None,
        kubernetes: None,
    }
}

fn find(host: &LimaHost, id: &str) -> Result<Snapshot, HostError> {
    folder::read_all(&host.paths().snapshots_dir())
        .into_iter()
        .find(|snapshot| snapshot.id == id)
        .ok_or_else(|| HostError("The snapshot does not exist.".into()))
}

/// `path`, or its nearest folder that exists, for `df`.
fn existing_parent(path: &Path) -> &Path {
    path.ancestors().find(|dir| dir.exists()).unwrap_or(path)
}

fn must_stop() -> HostError {
    HostError("Stop Captain Engine first.".into())
}

fn io_error(path: &Path, error: std::io::Error) -> HostError {
    HostError(format!("{}: {error}", path.display()))
}

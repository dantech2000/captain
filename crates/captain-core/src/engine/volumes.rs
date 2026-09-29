use super::EngineFuture;
use crate::model::{Volume, VolumePrune, VolumeUser};

/// Volumes: list, create, remove, prune, and the containers that use one.
pub trait VolumeApi {
    /// All volumes, with disk usage and container counts when the engine reports them.
    fn list_volumes(&self) -> EngineFuture<Vec<Volume>>;

    /// Creates a volume with the `local` driver.
    fn create_volume(&self, name: &str) -> EngineFuture<()>;

    /// Removes a volume. The engine refuses if a container uses it.
    fn remove_volume(&self, name: &str) -> EngineFuture<()>;

    /// Containers, running or stopped, that mount the volume `name`.
    fn volume_users(&self, name: &str) -> EngineFuture<Vec<VolumeUser>>;

    /// Removes volumes that no container uses. On Docker API 1.42 and later, only
    /// anonymous volumes go, unless `all` is true, which removes unused named volumes
    /// too. `label` limits the prune to volumes with that label, as `key` or
    /// `key=value`.
    fn prune_unused_volumes(&self, all: bool, label: Option<&str>) -> EngineFuture<VolumePrune>;
}

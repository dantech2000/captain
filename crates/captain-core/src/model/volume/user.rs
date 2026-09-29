use crate::model::ContainerState;

/// A container that mounts a volume, as the volume inspector lists it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumeUser {
    pub container_id: String,
    /// The container name without the leading `/`.
    pub name: String,
    pub state: ContainerState,
    /// Where the volume shows inside the container, for example `/var/lib/postgresql/data`.
    pub destination: String,
    pub read_only: bool,
}

impl VolumeUser {
    /// The destination, with ` (read-only)` after it for a read-only mount.
    pub fn destination_label(&self) -> String {
        if self.read_only {
            format!("{} (read-only)", self.destination)
        } else {
            self.destination.clone()
        }
    }
}

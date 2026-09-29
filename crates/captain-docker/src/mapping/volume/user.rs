use bollard::models::ContainerSummary;
use captain_core::model::VolumeUser;

use crate::mapping::container;

/// The containers from a list filtered by `volume=<name>`, each with where it mounts
/// the volume, sorted by name. The filter also matches a mount by destination path,
/// so a container without a mount of `volume` by name is left out.
pub fn volume_users(summaries: Vec<ContainerSummary>, volume: &str) -> Vec<VolumeUser> {
    let mut users: Vec<VolumeUser> = summaries
        .into_iter()
        .filter_map(|mut summary| {
            let mount = summary
                .mounts
                .take()
                .unwrap_or_default()
                .into_iter()
                .find(|mount| mount.name.as_deref() == Some(volume))?;
            let container = container(summary);
            Some(VolumeUser {
                container_id: container.id,
                name: container.name,
                state: container.state,
                destination: mount.destination.unwrap_or_default(),
                read_only: mount.rw == Some(false),
            })
        })
        .collect();
    users.sort_by(|a, b| a.name.cmp(&b.name));
    users
}

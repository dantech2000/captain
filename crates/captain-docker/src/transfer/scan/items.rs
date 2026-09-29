//! Turns the source engine's lists into plan items. Pure, so tests need no engine.

use std::collections::HashMap;
use std::path::Path;

use bollard::models::{ContainerSummary, Network};
use captain_core::migration::{HELPER_PREFIX, MigrationItem};
use captain_core::model::{ComposeProject, Container, Image, Volume};
use captain_core::store::compose_projects;

use crate::mapping;

/// Networks the engine makes itself. They exist in every engine.
const BUILT_IN_NETWORKS: [&str; 4] = ["bridge", "host", "none", "docker_gwbridge"];

/// User-defined networks with local scope.
pub fn networks(networks: Vec<Network>) -> Vec<MigrationItem> {
    networks
        .into_iter()
        .filter(|n| n.scope.as_deref().is_none_or(|scope| scope == "local"))
        .filter(|n| !matches!(n.driver.as_deref(), Some("host" | "null")))
        .filter_map(|n| {
            let name = n.name?;
            (!BUILT_IN_NETWORKS.contains(&name.as_str())).then(|| MigrationItem::Network {
                name,
                driver: n.driver.unwrap_or_default(),
            })
        })
        .collect()
}

/// Volumes, each with the running containers that mount it.
pub fn volumes(volumes: Vec<Volume>, containers: &[ContainerSummary]) -> Vec<MigrationItem> {
    volumes
        .into_iter()
        .map(|volume| MigrationItem::Volume {
            used_by_running: running_users(&volume.name, containers),
            name: volume.name,
            size: volume.size_bytes,
        })
        .collect()
}

fn running_users(volume: &str, containers: &[ContainerSummary]) -> Vec<String> {
    containers
        .iter()
        .filter(|c| c.state.is_some_and(|s| s.as_ref() == "running"))
        .filter(|c| {
            c.mounts
                .iter()
                .flatten()
                .any(|m| m.name.as_deref() == Some(volume))
        })
        .filter_map(|c| {
            c.names
                .as_ref()?
                .first()
                .map(|n| n.trim_start_matches('/').to_string())
        })
        .collect()
}

pub fn images(images: Vec<Image>) -> Vec<MigrationItem> {
    images
        .into_iter()
        .map(|image| MigrationItem::Image {
            in_use: image.in_use(),
            id: image.id,
            tags: image.repo_tags,
            size: image.size,
        })
        .collect()
}

/// Compose projects, then the containers that belong to no project. Captain's own
/// helper containers are left out. `exists` checks whether a path exists here.
pub fn containers(
    summaries: Vec<ContainerSummary>,
    exists: impl Fn(&Path) -> bool,
) -> Vec<MigrationItem> {
    let mut changed: HashMap<String, u64> = HashMap::new();
    let mut mounted: HashMap<String, Vec<String>> = HashMap::new();
    let containers: Vec<Container> = summaries
        .into_iter()
        .map(|summary| {
            let size_rw = summary.size_rw.and_then(|s| u64::try_from(s).ok());
            let volumes = named_volumes(&summary);
            let container = mapping::container(summary);
            changed.insert(container.id.clone(), size_rw.unwrap_or(0));
            mounted.insert(container.id.clone(), volumes);
            container
        })
        .filter(|c| !c.name.starts_with(&format!("{HELPER_PREFIX}-")))
        .collect();
    let size_rw = |c: &Container| changed.get(&c.id).copied().unwrap_or(0);
    let volumes = |c: &Container| mounted.get(&c.id).cloned().unwrap_or_default();
    let mut items: Vec<MigrationItem> = compose_projects(&containers)
        .into_iter()
        .map(|project| {
            let members: Vec<&Container> = project
                .services
                .iter()
                .flat_map(|s| &s.containers)
                .collect();
            let running_services = project
                .services
                .iter()
                .filter(|s| s.containers.iter().any(|c| c.state.is_active()))
                .map(|s| s.name.clone())
                .collect();
            let running_containers = members
                .iter()
                .filter(|c| c.state.is_active())
                .map(|c| c.name.clone())
                .collect();
            let mut project_volumes: Vec<String> =
                members.iter().flat_map(|c| volumes(c)).collect();
            project_volumes.sort();
            project_volumes.dedup();
            MigrationItem::ComposeProject {
                files_exist: files_exist(&project, &exists),
                containers: members.iter().map(|c| c.name.clone()).collect(),
                changed: members
                    .iter()
                    .filter(|c| size_rw(c) > 0)
                    .map(|c| c.name.clone())
                    .collect(),
                running_services,
                running_containers,
                volumes: project_volumes,
                name: project.name,
                working_dir: project.working_dir,
                config_files: project.config_files,
            }
        })
        .collect();
    items.extend(
        containers
            .iter()
            .filter(|c| c.compose_project.is_none())
            .map(|c| MigrationItem::Container {
                id: c.id.clone(),
                name: c.name.clone(),
                image: c.image.clone(),
                running: c.state.is_active(),
                size_rw: size_rw(c),
                volumes: volumes(c),
            }),
    );
    items
}

/// The names of the named volumes a container mounts, in mount order.
fn named_volumes(summary: &ContainerSummary) -> Vec<String> {
    summary
        .mounts
        .iter()
        .flatten()
        .filter(|m| m.typ.as_deref() == Some("volume"))
        .filter_map(|m| m.name.clone())
        .collect()
}

/// True if the project's folder and every Compose file exist on this computer.
fn files_exist(project: &ComposeProject, exists: &impl Fn(&Path) -> bool) -> bool {
    let Some(dir) = project.working_dir.as_deref().map(Path::new) else {
        return false;
    };
    !project.config_files.is_empty()
        && exists(dir)
        && project.config_files.iter().all(|f| exists(&dir.join(f)))
}

#[cfg(test)]
mod tests;

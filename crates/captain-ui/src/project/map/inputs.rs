use std::collections::HashMap;

use captain_core::model::{Container, ContainerDetail, ContainerState};
use captain_core::project_map::MapService;

use crate::project::group_info::service_name;

/// The name each container goes by on the map: its service, or its container name
/// when a scaled service has several containers.
pub fn titles(containers: &[&Container]) -> Vec<String> {
    let services: Vec<String> = containers.iter().map(|c| service_name(c)).collect();
    containers
        .iter()
        .zip(&services)
        .map(|(container, service)| {
            if services.iter().filter(|s| *s == service).count() > 1 {
                container.display_name()
            } else {
                service.clone()
            }
        })
        .collect()
}

/// What the layout needs from each container and its `inspect` result.
pub fn services(
    containers: &[&Container],
    titles: &[String],
    details: &HashMap<String, (ContainerState, ContainerDetail)>,
) -> Vec<MapService> {
    containers
        .iter()
        .zip(titles)
        .map(|(container, title)| {
            let detail = details.get(&container.id).map(|(_, detail)| detail);
            MapService {
                id: container.id.clone(),
                name: title.clone(),
                container_name: container.name.clone(),
                networks: detail.map(|d| d.networks.clone()).unwrap_or_default(),
                ports: container.published_ports(),
                volumes: detail
                    .map(|d| {
                        d.mounts
                            .iter()
                            .filter(|m| m.volume)
                            .map(|m| m.source.clone())
                            .collect()
                    })
                    .unwrap_or_default(),
                env: detail.map(|d| d.env.clone()).unwrap_or_default(),
            }
        })
        .collect()
}

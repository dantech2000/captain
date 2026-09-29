//! Builds the create request for a copy of a container from its inspect data. Pure,
//! so tests need no engine.

use std::collections::HashMap;

use bollard::models::{
    ContainerCreateBody, ContainerInspectResponse, EndpointSettings, HostConfig, Mount, MountType,
    NetworkingConfig,
};

/// What it takes to recreate a container.
#[derive(Debug, Clone, Default)]
pub struct Recreate {
    pub body: ContainerCreateBody,
    /// Networks to connect after the create, beyond the one in the body.
    pub extra_networks: Vec<(String, EndpointSettings)>,
    /// True if the source container runs, so the copy starts too.
    pub start: bool,
}

/// The create request for a copy of `inspect` that runs `image`. It keeps the
/// image, command, entrypoint, environment, ports, mounts, labels, restart policy,
/// and networks, and leaves out what belongs to the old engine: IDs, addresses,
/// and cgroup and device settings.
pub fn recreate(inspect: &ContainerInspectResponse, image: &str) -> Recreate {
    let config = inspect.config.clone().unwrap_or_default();
    let host = inspect.host_config.clone().unwrap_or_default();
    let short_id: String = inspect
        .id
        .clone()
        .unwrap_or_default()
        .chars()
        .take(12)
        .collect();
    let mut networks = networks(inspect, &short_id);
    let mode = host.network_mode.clone().unwrap_or_default();
    let primary = networks
        .iter()
        .position(|(name, _)| *name == mode)
        .map(|ix| networks.remove(ix));
    let host_config = HostConfig {
        mounts: mounts(inspect, &host),
        binds: host.binds,
        port_bindings: host.port_bindings,
        publish_all_ports: host.publish_all_ports,
        restart_policy: host.restart_policy,
        network_mode: host.network_mode,
        extra_hosts: host.extra_hosts,
        init: host.init,
        cap_add: host.cap_add,
        cap_drop: host.cap_drop,
        privileged: host.privileged,
        dns: host.dns,
        dns_search: host.dns_search,
        dns_options: host.dns_options,
        tmpfs: host.tmpfs,
        shm_size: host.shm_size,
        sysctls: host.sysctls,
        ulimits: host.ulimits,
        readonly_rootfs: host.readonly_rootfs,
        group_add: host.group_add,
        auto_remove: host.auto_remove,
        ..HostConfig::default()
    };
    let body = ContainerCreateBody {
        // A hostname equal to the old ID came from the engine, not the user.
        hostname: config.hostname.filter(|h| !h.is_empty() && *h != short_id),
        domainname: config.domainname,
        user: config.user,
        exposed_ports: config.exposed_ports,
        tty: config.tty,
        open_stdin: config.open_stdin,
        stdin_once: config.stdin_once,
        env: config.env,
        cmd: config.cmd,
        healthcheck: config.healthcheck,
        image: Some(image.into()),
        volumes: config.volumes,
        working_dir: config.working_dir,
        entrypoint: config.entrypoint,
        labels: config.labels,
        stop_signal: config.stop_signal,
        stop_timeout: config.stop_timeout,
        host_config: Some(host_config),
        networking_config: primary.map(|(name, endpoint)| NetworkingConfig {
            endpoints_config: Some(HashMap::from([(name, endpoint)])),
        }),
        ..ContainerCreateBody::default()
    };
    let start = inspect
        .state
        .as_ref()
        .and_then(|state| state.running)
        .unwrap_or(false);
    Recreate {
        body,
        extra_networks: networks,
        start,
    }
}

/// The user-defined networks and their aliases, without the old addresses. The
/// built-in networks come from the network mode alone.
fn networks(inspect: &ContainerInspectResponse, short_id: &str) -> Vec<(String, EndpointSettings)> {
    let mut networks: Vec<(String, EndpointSettings)> = inspect
        .network_settings
        .as_ref()
        .and_then(|settings| settings.networks.clone())
        .unwrap_or_default()
        .into_iter()
        .filter(|(name, _)| !matches!(name.as_str(), "bridge" | "host" | "none"))
        .map(|(name, old)| {
            let aliases = old
                .aliases
                .map(|aliases| aliases.into_iter().filter(|a| a != short_id).collect());
            let endpoint = EndpointSettings {
                aliases,
                links: old.links,
                driver_opts: old.driver_opts,
                ..EndpointSettings::default()
            };
            (name, endpoint)
        })
        .collect();
    networks.sort_by(|a, b| a.0.cmp(&b.0));
    networks
}

/// The container's mounts, plus a mount for each volume that the image declared
/// and the engine created, so the copy uses the copied volume, not a new empty one.
fn mounts(inspect: &ContainerInspectResponse, host: &HostConfig) -> Option<Vec<Mount>> {
    let mut mounts = host.mounts.clone().unwrap_or_default();
    let bound: Vec<&str> = host
        .binds
        .iter()
        .flatten()
        .filter_map(|bind| bind.split(':').nth(1))
        .chain(mounts.iter().filter_map(|m| m.target.as_deref()))
        .collect();
    let implicit: Vec<Mount> = inspect
        .mounts
        .iter()
        .flatten()
        .filter(|m| m.typ.as_deref() == Some("volume"))
        .filter(|m| {
            m.destination
                .as_deref()
                .is_some_and(|d| !bound.contains(&d))
        })
        .map(|m| Mount {
            target: m.destination.clone(),
            source: m.name.clone(),
            typ: Some(MountType::VOLUME),
            read_only: m.rw.map(|rw| !rw),
            ..Mount::default()
        })
        .collect();
    mounts.extend(implicit);
    (!mounts.is_empty()).then_some(mounts)
}

#[cfg(test)]
mod tests;

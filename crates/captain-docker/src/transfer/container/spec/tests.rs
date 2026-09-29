use std::collections::HashMap;

use bollard::models::{
    ContainerConfig, ContainerInspectResponse, ContainerState, EndpointSettings, HostConfig,
    MountPoint, NetworkSettings, RestartPolicy, RestartPolicyNameEnum,
};

use super::recreate;

fn inspect() -> ContainerInspectResponse {
    let endpoint = |aliases: Vec<&str>| EndpointSettings {
        aliases: Some(aliases.into_iter().map(String::from).collect()),
        ip_address: Some("172.30.0.5".into()),
        network_id: Some("net-id".into()),
        ..EndpointSettings::default()
    };
    ContainerInspectResponse {
        id: Some("0123456789abcdef".into()),
        state: Some(ContainerState {
            running: Some(true),
            ..ContainerState::default()
        }),
        config: Some(ContainerConfig {
            hostname: Some("0123456789ab".into()),
            image: Some("sha256:old".into()),
            env: Some(vec!["MODE=prod".into()]),
            cmd: Some(vec!["serve".into()]),
            labels: Some(HashMap::from([("team".into(), "web".into())])),
            ..ContainerConfig::default()
        }),
        host_config: Some(HostConfig {
            binds: Some(vec!["data:/data".into()]),
            network_mode: Some("backend".into()),
            restart_policy: Some(RestartPolicy {
                name: Some(RestartPolicyNameEnum::ALWAYS),
                maximum_retry_count: None,
            }),
            cpuset_cpus: Some("0-1".into()),
            ..HostConfig::default()
        }),
        mounts: Some(vec![
            MountPoint {
                typ: Some("volume".into()),
                name: Some("data".into()),
                destination: Some("/data".into()),
                rw: Some(true),
                ..MountPoint::default()
            },
            MountPoint {
                typ: Some("volume".into()),
                name: Some("f00d".into()),
                destination: Some("/var/cache".into()),
                rw: Some(true),
                ..MountPoint::default()
            },
        ]),
        network_settings: Some(NetworkSettings {
            networks: Some(HashMap::from([
                ("backend".into(), endpoint(vec!["web", "0123456789ab"])),
                ("frontend".into(), endpoint(vec!["site"])),
                ("bridge".into(), endpoint(vec![])),
            ])),
            ..NetworkSettings::default()
        }),
        ..ContainerInspectResponse::default()
    }
}

#[test]
fn keeps_the_run_settings() {
    let spec = recreate(&inspect(), "web:1");
    let body = &spec.body;
    assert!(spec.start);
    assert_eq!(body.image.as_deref(), Some("web:1"));
    assert_eq!(body.hostname, None);
    assert_eq!(body.env, Some(vec!["MODE=prod".into()]));
    assert_eq!(body.cmd, Some(vec!["serve".into()]));
    assert_eq!(body.labels.as_ref().unwrap()["team"], "web");
    let host = body.host_config.as_ref().unwrap();
    assert_eq!(host.binds, Some(vec!["data:/data".into()]));
    assert_eq!(
        host.restart_policy.as_ref().unwrap().name,
        Some(RestartPolicyNameEnum::ALWAYS)
    );
    assert_eq!(host.cpuset_cpus, None);
}

#[test]
fn mounts_the_volume_the_engine_created() {
    let spec = recreate(&inspect(), "web:1");
    let mounts = spec.body.host_config.unwrap().mounts.unwrap();
    assert_eq!(mounts.len(), 1);
    assert_eq!(mounts[0].source.as_deref(), Some("f00d"));
    assert_eq!(mounts[0].target.as_deref(), Some("/var/cache"));
    assert_eq!(mounts[0].read_only, Some(false));
}

#[test]
fn joins_networks_without_old_addresses() {
    let spec = recreate(&inspect(), "web:1");
    let endpoints = spec
        .body
        .networking_config
        .unwrap()
        .endpoints_config
        .unwrap();
    let backend = &endpoints["backend"];
    assert_eq!(backend.aliases, Some(vec!["web".into()]));
    assert_eq!(backend.ip_address, None);
    assert_eq!(backend.network_id, None);
    let extra: Vec<&str> = spec
        .extra_networks
        .iter()
        .map(|(n, _)| n.as_str())
        .collect();
    assert_eq!(extra, ["frontend"]);
}

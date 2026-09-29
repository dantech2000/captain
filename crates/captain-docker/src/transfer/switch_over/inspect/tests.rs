use std::collections::HashMap;

use bollard::models::{
    ContainerConfig, ContainerInspectResponse, ContainerState, Health, HealthStatusEnum,
    NetworkSettings, PortBinding,
};

use super::{DEFAULT_STOP_TIMEOUT, health, published_tcp_ports, stop_timeout};

fn with_stop_timeout(seconds: Option<i64>) -> ContainerInspectResponse {
    ContainerInspectResponse {
        config: Some(ContainerConfig {
            stop_timeout: seconds,
            ..ContainerConfig::default()
        }),
        ..ContainerInspectResponse::default()
    }
}

fn with_health(status: Option<HealthStatusEnum>) -> ContainerInspectResponse {
    ContainerInspectResponse {
        state: Some(ContainerState {
            health: status.map(|status| Health {
                status: Some(status),
                ..Health::default()
            }),
            ..ContainerState::default()
        }),
        ..ContainerInspectResponse::default()
    }
}

fn binding(ip: &str, port: &str) -> PortBinding {
    PortBinding {
        host_ip: Some(ip.into()),
        host_port: Some(port.into()),
    }
}

#[test]
fn stop_timeout_prefers_the_containers_own() {
    assert_eq!(stop_timeout(&with_stop_timeout(Some(120))), 120);
    assert_eq!(stop_timeout(&with_stop_timeout(None)), DEFAULT_STOP_TIMEOUT);
    assert_eq!(
        stop_timeout(&with_stop_timeout(Some(0))),
        DEFAULT_STOP_TIMEOUT
    );
    assert_eq!(
        stop_timeout(&ContainerInspectResponse::default()),
        DEFAULT_STOP_TIMEOUT
    );
}

#[test]
fn health_is_none_without_a_check() {
    assert_eq!(health(&with_health(None)), None);
    assert_eq!(health(&with_health(Some(HealthStatusEnum::NONE))), None);
    assert_eq!(
        health(&with_health(Some(HealthStatusEnum::STARTING))),
        Some(HealthStatusEnum::STARTING)
    );
}

#[test]
fn published_ports_are_tcp_and_reachable_from_loopback() {
    let ports = HashMap::from([
        (
            "5432/tcp".to_string(),
            Some(vec![binding("0.0.0.0", "5432"), binding("::", "5432")]),
        ),
        (
            "80/tcp".to_string(),
            Some(vec![binding("127.0.0.1", "8080")]),
        ),
        ("53/udp".to_string(), Some(vec![binding("0.0.0.0", "53")])),
        (
            "9000/tcp".to_string(),
            Some(vec![binding("192.168.1.5", "9000")]),
        ),
        ("6379/tcp".to_string(), None),
    ]);
    let inspect = ContainerInspectResponse {
        network_settings: Some(NetworkSettings {
            ports: Some(ports),
            ..NetworkSettings::default()
        }),
        ..ContainerInspectResponse::default()
    };
    assert_eq!(published_tcp_ports(&inspect), [5432, 8080]);
}

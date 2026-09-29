use serde_json::{Map, Value, json};

use super::{check_custom, daemon_json};
use crate::daemon::DaemonSettings;

fn custom(value: Value) -> Map<String, Value> {
    value.as_object().cloned().unwrap()
}

#[test]
fn defaults_turn_on_the_features_only() {
    let json = daemon_json(&DaemonSettings::default());
    assert_eq!(
        json,
        json!({"features": {"cdi": true, "containerd-snapshotter": true}})
    );
}

#[test]
fn merges_fields_custom_keys_and_features() {
    let settings = DaemonSettings {
        registry_mirrors: vec!["https://mirror.gcr.io".into()],
        insecure_registries: vec!["registry.local:5000".into()],
        custom: custom(json!({
            "log-level": "warn",
            "features": {"buildkit": true}
        })),
        ..DaemonSettings::default()
    };
    assert_eq!(
        daemon_json(&settings),
        json!({
            "log-level": "warn",
            "registry-mirrors": ["https://mirror.gcr.io"],
            "insecure-registries": ["registry.local:5000"],
            "features": {"buildkit": true, "cdi": true, "containerd-snapshotter": true}
        })
    );
}

#[test]
fn managed_keys_from_a_hand_edited_file_never_reach_the_engine() {
    let settings = DaemonSettings {
        custom: custom(json!({
            "hosts": ["tcp://0.0.0.0:2375"],
            "containerd": "/other.sock",
            "registry-mirrors": ["https://ignored"],
            "features": {"containerd-snapshotter": false}
        })),
        ..DaemonSettings::default()
    };
    assert_eq!(
        daemon_json(&settings),
        json!({"features": {"cdi": true, "containerd-snapshotter": true}})
    );
}

#[test]
fn check_rejects_managed_keys_and_allows_other_features() {
    let hosts = check_custom(&custom(json!({"hosts": []}))).unwrap_err();
    assert!(hosts.contains("\"hosts\""), "{hosts}");
    let snapshotter = check_custom(&custom(
        json!({"features": {"containerd-snapshotter": false}}),
    ));
    assert!(snapshotter.is_err());
    assert!(check_custom(&custom(json!({"features": {"buildkit": true}}))).is_ok());
}

#[test]
fn tcp_port_only_while_the_switch_is_on() {
    let off = DaemonSettings::default();
    assert_eq!(off.state().tcp_port, None);
    let on = DaemonSettings {
        tcp: true,
        tcp_port: 2376,
        ..off
    };
    assert_eq!(on.state().tcp_port, Some(2376));
}

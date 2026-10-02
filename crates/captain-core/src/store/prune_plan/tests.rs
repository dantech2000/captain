use std::collections::BTreeMap;

use super::{label_matches, prunable_networks, prunable_volumes};
use crate::model::{Network, Volume};

fn labels(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

fn volume(name: &str, containers: Option<usize>, pairs: &[(&str, &str)]) -> Volume {
    Volume {
        name: name.into(),
        containers,
        labels: labels(pairs),
        ..Volume::default()
    }
}

fn network(name: &str, containers: usize, pairs: &[(&str, &str)]) -> Network {
    Network {
        id: format!("id-{name}"),
        name: name.into(),
        containers,
        labels: labels(pairs),
        ..Network::default()
    }
}

fn volume_names(volumes: Vec<&Volume>) -> Vec<&str> {
    volumes.into_iter().map(|v| v.name.as_str()).collect()
}

fn network_names(networks: Vec<&Network>) -> Vec<&str> {
    networks.into_iter().map(|n| n.name.as_str()).collect()
}

#[test]
fn label_filters_match_a_key_or_a_key_and_value() {
    let set = labels(&[("captain-agent-test", "42")]);
    assert!(label_matches(&set, None));
    assert!(label_matches(&set, Some("captain-agent-test")));
    assert!(label_matches(&set, Some("captain-agent-test=42")));
    assert!(!label_matches(&set, Some("captain-agent-test=7")));
    assert!(!label_matches(&set, Some("other")));
    assert!(!label_matches(&BTreeMap::new(), Some("captain-agent-test")));
}

#[test]
fn volume_prune_keeps_named_volumes_unless_all() {
    let anonymous = "a".repeat(64);
    let volumes = vec![
        volume(&anonymous, Some(0), &[]),
        volume("labelled", Some(0), &[("com.docker.volume.anonymous", "")]),
        volume("pgdata", Some(0), &[]),
        volume("used", Some(1), &[("com.docker.volume.anonymous", "")]),
        volume("unknown", None, &[("com.docker.volume.anonymous", "")]),
    ];
    assert_eq!(
        volume_names(prunable_volumes(&volumes, false, None)),
        [anonymous.as_str(), "labelled"]
    );
    assert_eq!(
        volume_names(prunable_volumes(&volumes, true, None)),
        [anonymous.as_str(), "labelled", "pgdata"]
    );
}

#[test]
fn volume_prune_honors_the_label_filter() {
    let volumes = vec![
        volume("mine", Some(0), &[("captain-agent-test", "1")]),
        volume("theirs", Some(0), &[]),
    ];
    let pruned = prunable_volumes(&volumes, true, Some("captain-agent-test"));
    assert_eq!(volume_names(pruned), ["mine"]);
}

#[test]
fn network_prune_skips_built_in_and_used_networks() {
    let networks = vec![
        network("bridge", 0, &[]),
        network("busy", 2, &[]),
        network("idle", 0, &[]),
        network("mine", 0, &[("captain-agent-test", "")]),
    ];
    assert_eq!(
        network_names(prunable_networks(&networks, None)),
        ["idle", "mine"]
    );
    assert_eq!(
        network_names(prunable_networks(&networks, Some("captain-agent-test"))),
        ["mine"]
    );
}

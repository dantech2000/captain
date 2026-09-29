use std::collections::HashMap;

use bollard::models::{
    EndpointResource, Ipam, IpamConfig, Network as DockerNetwork, NetworkInspect,
};

use super::{network, network_detail, network_prune_filters};

fn config(subnet: &str, gateway: &str) -> IpamConfig {
    IpamConfig {
        subnet: Some(subnet.into()),
        gateway: Some(gateway.into()),
        ..IpamConfig::default()
    }
}

#[test]
fn maps_fields_and_prefers_ipv4() {
    let docker = DockerNetwork {
        id: Some("abc123".into()),
        name: Some("shop_default".into()),
        driver: Some("bridge".into()),
        scope: Some("local".into()),
        internal: Some(true),
        ipam: Some(Ipam {
            config: Some(vec![
                config("fd00::/64", "fd00::1"),
                config("172.18.0.0/16", "172.18.0.1"),
            ]),
            ..Ipam::default()
        }),
        labels: Some(HashMap::from([(
            "com.docker.compose.project".into(),
            "shop".into(),
        )])),
        ..DockerNetwork::default()
    };
    let n = network(docker, 3);
    assert_eq!(n.id, "abc123");
    assert_eq!(n.name, "shop_default");
    assert_eq!(n.driver, "bridge");
    assert_eq!(n.scope, "local");
    assert!(n.internal);
    assert_eq!(n.subnet.as_deref(), Some("172.18.0.0/16"));
    assert_eq!(n.gateway.as_deref(), Some("172.18.0.1"));
    assert_eq!(n.containers, 3);
    assert_eq!(n.compose_project.as_deref(), Some("shop"));
}

#[test]
fn missing_ipam_gives_no_subnet() {
    let docker = DockerNetwork {
        name: Some("none".into()),
        ..DockerNetwork::default()
    };
    let n = network(docker, 0);
    assert_eq!(n.subnet, None);
    assert_eq!(n.gateway, None);
    assert!(n.is_built_in());
}

#[test]
fn falls_back_to_ipv6_and_drops_empty_gateways() {
    let ipam = Ipam {
        config: Some(vec![config("fd00::/64", "")]),
        ..Ipam::default()
    };
    let docker = DockerNetwork {
        ipam: Some(ipam),
        ..DockerNetwork::default()
    };
    let n = network(docker, 0);
    assert_eq!(n.subnet.as_deref(), Some("fd00::/64"));
    assert_eq!(n.gateway, None);
}

#[test]
fn keeps_labels() {
    let docker = DockerNetwork {
        labels: Some(HashMap::from([("team".into(), "web".into())])),
        ..DockerNetwork::default()
    };
    let n = network(docker, 0);
    assert_eq!(n.labels.get("team").map(String::as_str), Some("web"));
    assert_eq!(n.compose_project, None);
}

fn resource(name: &str, ipv4: &str, mac: &str) -> EndpointResource {
    EndpointResource {
        name: Some(name.into()),
        ipv4_address: Some(ipv4.into()),
        ipv6_address: Some(String::new()),
        mac_address: Some(mac.into()),
        ..EndpointResource::default()
    }
}

#[test]
fn detail_lists_all_subnets_and_sorted_endpoints() {
    let inspect = NetworkInspect {
        id: Some("abc123".into()),
        name: Some("shop_default".into()),
        driver: Some("bridge".into()),
        created: Some("2026-09-28T10:00:00Z".into()),
        attachable: Some(true),
        ipam: Some(Ipam {
            config: Some(vec![
                config("172.18.0.0/16", "172.18.0.1"),
                config("fd00::/64", ""),
            ]),
            ..Ipam::default()
        }),
        containers: Some(HashMap::from([
            (
                "c2".to_string(),
                resource("web", "172.18.0.3/16", "02:42:ac:12:00:03"),
            ),
            (
                "c1".to_string(),
                resource("db", "172.18.0.2/16", "02:42:ac:12:00:02"),
            ),
        ])),
        ..NetworkInspect::default()
    };
    let detail = network_detail(inspect);
    assert_eq!(detail.network.id, "abc123");
    assert_eq!(detail.network.subnet.as_deref(), Some("172.18.0.0/16"));
    assert_eq!(detail.network.containers, 2);
    assert!(detail.attachable);
    assert_eq!(detail.created_date(), "2026-09-28");
    assert_eq!(detail.subnets_label(), "172.18.0.0/16, fd00::/64");
    assert_eq!(detail.subnets[1].gateway, None);
    let names: Vec<&str> = detail.endpoints.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, ["db", "web"]);
    let db = &detail.endpoints[0];
    assert_eq!(db.container_id, "c1");
    assert_eq!(db.address(), Some("172.18.0.2"));
    assert_eq!(db.ipv6, None);
    assert_eq!(db.mac.as_deref(), Some("02:42:ac:12:00:02"));
}

#[test]
fn prune_filters_add_the_label_only_when_asked() {
    assert!(network_prune_filters(None).is_empty());
    assert_eq!(
        network_prune_filters(Some("captain-agent-test"))["label"],
        ["captain-agent-test"]
    );
}

use std::collections::HashMap;

use bollard::models::{Ipam, IpamConfig, NetworkInspect};

use super::{create_request, has_subnet, is_overlap};

fn network(config: Option<Vec<IpamConfig>>) -> NetworkInspect {
    NetworkInspect {
        name: Some("backend".into()),
        driver: Some("bridge".into()),
        internal: Some(true),
        labels: Some(HashMap::from([("team".into(), "web".into())])),
        ipam: Some(Ipam {
            driver: Some("default".into()),
            config,
            options: None,
        }),
        ..NetworkInspect::default()
    }
}

#[test]
fn keeps_driver_labels_and_subnet_or_lets_the_engine_pick() {
    let subnet = IpamConfig {
        subnet: Some("172.30.0.0/16".into()),
        ..IpamConfig::default()
    };
    let request = create_request(&network(Some(vec![subnet])), "backend-copy");
    assert_eq!(request.name, "backend-copy");
    assert_eq!(request.driver.as_deref(), Some("bridge"));
    assert_eq!(request.internal, Some(true));
    assert_eq!(request.labels.unwrap()["team"], "web");
    assert!(has_subnet(&create_request(
        &network(Some(vec![IpamConfig {
            subnet: Some("10.0.0.0/24".into()),
            ..IpamConfig::default()
        }])),
        "x"
    )));
    // No subnet means the engine picks.
    let request = create_request(&network(Some(vec![])), "backend");
    assert!(!has_subnet(&request));
    assert_eq!(request.ipam.unwrap().config, None);
}

#[test]
fn knows_the_overlap_error() {
    assert!(is_overlap(
        "Pool overlaps with other one on this address space"
    ));
    assert!(!is_overlap("network backend already exists"));
}

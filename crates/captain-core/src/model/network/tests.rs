use super::Network;

fn network(name: &str, containers: usize) -> Network {
    Network {
        id: "0123456789abcdef".into(),
        name: name.into(),
        containers,
        ..Network::default()
    }
}

#[test]
fn only_empty_user_networks_can_be_removed() {
    for name in ["bridge", "host", "none"] {
        let n = network(name, 0);
        assert!(n.is_built_in());
        assert!(!n.can_remove());
        assert!(n.remove_blocker().is_some());
    }
    let n = network("app_default", 2);
    assert!(!n.is_built_in());
    assert!(n.is_in_use());
    assert!(!n.can_remove());
    assert_eq!(n.remove_blocker(), Some("Disconnect its containers first"));
    let n = network("app_default", 0);
    assert!(n.can_remove());
    assert_eq!(n.remove_blocker(), None);
}

#[test]
fn endpoint_address_drops_the_prefix_and_falls_back_to_ipv6() {
    use super::NetworkEndpoint;

    let v4 = NetworkEndpoint {
        ipv4: Some("172.18.0.2/16".into()),
        ipv6: Some("fd00::2/64".into()),
        ..NetworkEndpoint::default()
    };
    assert_eq!(v4.address(), Some("172.18.0.2"));
    let v6 = NetworkEndpoint {
        ipv6: Some("fd00::2/64".into()),
        ..NetworkEndpoint::default()
    };
    assert_eq!(v6.address(), Some("fd00::2"));
    assert_eq!(NetworkEndpoint::default().address(), None);
}

#[test]
fn detail_labels_join_subnets_and_gateways() {
    use super::{NetworkDetail, Subnet};

    let empty = NetworkDetail::default();
    assert_eq!(empty.subnets_label(), "—");
    assert_eq!(empty.gateways_label(), "—");
    assert_eq!(empty.created_date(), "—");

    let detail = NetworkDetail {
        created: "2026-09-28T10:00:00.123Z".into(),
        subnets: vec![
            Subnet {
                subnet: "172.18.0.0/16".into(),
                gateway: Some("172.18.0.1".into()),
            },
            Subnet {
                subnet: "fd00::/64".into(),
                gateway: None,
            },
        ],
        ..NetworkDetail::default()
    };
    assert_eq!(detail.subnets_label(), "172.18.0.0/16, fd00::/64");
    assert_eq!(detail.gateways_label(), "172.18.0.1");
    assert_eq!(detail.created_date(), "2026-09-28");
}

#[test]
fn detail_from_a_list_entry_keeps_its_subnet() {
    use super::NetworkDetail;

    let mut n = network("shop_default", 0);
    n.subnet = Some("172.20.0.0/16".into());
    n.gateway = Some("172.20.0.1".into());
    let detail = NetworkDetail::from_network(n.clone());
    assert_eq!(detail.subnets_label(), "172.20.0.0/16");
    assert_eq!(detail.gateways_label(), "172.20.0.1");
    assert_eq!(detail.network, n);
}

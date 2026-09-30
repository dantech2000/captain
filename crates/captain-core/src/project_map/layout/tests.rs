use super::layout;
use crate::model::EnvVar;
use crate::project_map::{EdgeKind, MapService};

fn service(name: &str, network: &str, ports: &[u16], volumes: &[&str]) -> MapService {
    MapService {
        id: format!("id-{name}"),
        name: name.into(),
        container_name: format!("shop-{name}-1"),
        networks: if network.is_empty() {
            vec![]
        } else {
            vec![network.into()]
        },
        ports: ports.to_vec(),
        volumes: volumes.iter().map(|v| v.to_string()).collect(),
        env: vec![],
    }
}

fn shop() -> Vec<MapService> {
    let mut api = service("api", "shop_default", &[3000], &[]);
    api.env = vec![EnvVar::parse("DATABASE_URL=postgres://app@postgres/shop")];
    vec![
        service("worker", "shop_default", &[], &[]),
        service("web", "shop_default", &[8080], &["node_modules"]),
        service("postgres", "shop_default", &[5432], &["shop_pgdata"]),
        api,
        service("mailer", "mail", &[], &[]),
        service("pending", "", &[], &[]),
    ]
}

#[test]
fn puts_publishing_services_first_and_one_lane_per_network() {
    let map = layout(&shop());
    let lanes: Vec<&str> = map.lanes.iter().map(|l| l.network.as_str()).collect();
    // Networks by name, the lane without a network last.
    assert_eq!(lanes, ["mail", "shop_default", ""]);
    let x = |name: &str| map.node(&format!("id-{name}")).unwrap().x;
    let y = |name: &str| map.node(&format!("id-{name}")).unwrap().y;
    // Services with ports share the first column, in name order; the rest the second.
    assert_eq!(x("api"), x("postgres"));
    assert_eq!(x("api"), x("web"));
    assert!(x("worker") > x("api"));
    assert!(y("api") < y("postgres") && y("postgres") < y("web"));
    // Each service sits inside its lane.
    let lane = map.lanes[1].rect;
    assert!(y("api") > lane.y && y("web") + 104. < lane.y + lane.h);
}

#[test]
fn is_the_same_for_any_input_order() {
    let mut reversed = shop();
    reversed.reverse();
    assert_eq!(layout(&shop()), layout(&reversed));
}

#[test]
fn places_pins_and_volumes_beside_their_services_with_edges() {
    let map = layout(&shop());
    let mid = |id: &str| {
        let node = map.node(id).unwrap();
        node.y + node.h / 2.
    };
    let pin = map.pins.iter().find(|(port, ..)| *port == 5432).unwrap();
    assert_eq!(pin.1, "id-postgres");
    assert_eq!(pin.2.y + pin.2.h / 2., mid("id-postgres"));
    let volume = map.volumes.iter().find(|v| v.key == "shop_pgdata").unwrap();
    assert_eq!(volume.rect.y + volume.rect.h / 2., mid("id-postgres"));
    let count = |kind| map.edges.iter().filter(|e| e.kind == kind).count();
    assert_eq!(count(EdgeKind::Port), 3);
    assert_eq!(count(EdgeKind::Mount), 2);
    assert_eq!(count(EdgeKind::TalksTo), 1);
    assert!(map.width > volume.rect.x + volume.rect.w);
}

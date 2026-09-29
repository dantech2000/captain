use super::NetworkStore;
use crate::model::Network;
use crate::store::UsageFilter;

fn network(name: &str, project: Option<&str>, containers: usize) -> Network {
    Network {
        id: format!("id-{name}"),
        name: name.into(),
        compose_project: project.map(Into::into),
        containers,
        ..Network::default()
    }
}

fn store() -> NetworkStore {
    let mut store = NetworkStore::default();
    store.replace(vec![
        network("shop_default", Some("shop"), 3),
        network("none", None, 0),
        network("lab", None, 0),
        network("bridge", None, 1),
        network("host", None, 0),
    ]);
    store
}

#[test]
fn sorts_built_in_networks_first() {
    let names: Vec<_> = store().networks().iter().map(|n| n.name.clone()).collect();
    assert_eq!(names, ["bridge", "host", "none", "lab", "shop_default"]);
}

#[test]
fn groups_and_filters() {
    let store = store();
    let groups = store.groups(UsageFilter::All);
    assert_eq!(groups[0].project.as_deref(), Some("shop"));
    assert_eq!(groups[1].items.len(), 4);

    assert_eq!(store.count(UsageFilter::InUse), 2);
    assert_eq!(store.count(UsageFilter::Unused), 3);
    let removable: Vec<_> = store
        .networks()
        .iter()
        .filter(|n| n.can_remove())
        .map(|n| n.name.as_str())
        .collect();
    assert_eq!(removable, ["lab"]);
}

#[test]
fn summary_counts_networks_in_use() {
    assert_eq!(store().summary(), "5 networks · 2 in use");
    let mut one = NetworkStore::default();
    one.replace(vec![network("lab", None, 0)]);
    assert_eq!(one.summary(), "1 network");
}

#[test]
fn find_and_first() {
    let store = store();
    assert_eq!(store.find("id-lab").unwrap().name, "lab");
    assert_eq!(store.first(UsageFilter::All).unwrap().name, "shop_default");
    assert_eq!(store.first(UsageFilter::Unused).unwrap().name, "host");
}

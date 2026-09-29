//! Runs against the real engine that discovery finds. Ignored by default:
//! `cargo test -p captain-docker -- --ignored`. It only lists and inspects, and it
//! creates and removes networks named `captain-agent-...`. The prune test prunes only
//! networks with a `captain-agent-test` label that it put on its own network.

use std::collections::HashMap;

use bollard::models::NetworkCreateRequest;
use bollard::{API_DEFAULT_VERSION, Docker};
use captain_core::NetworkApi;
use captain_docker::{DiscoveryInput, DockerEngine, Endpoint, discover};
use futures::executor::block_on;

fn endpoint() -> Endpoint {
    discover(&DiscoveryInput::from_env(), |path| path.exists()).expect("discover")
}

fn connect() -> DockerEngine {
    DockerEngine::connect(endpoint()).expect("connect")
}

/// Creates a bridge network with `label` through bollard, because Captain creates
/// networks without labels. Returns the network ID.
fn create_labelled(name: &str, label: (&str, &str)) -> String {
    let runtime = tokio::runtime::Runtime::new().expect("runtime");
    runtime.block_on(async {
        let docker = match endpoint() {
            Endpoint::Tcp(address) => Docker::connect_with_http(&address, 30, API_DEFAULT_VERSION),
            other => Docker::connect_with_socket(&other.to_string(), 30, API_DEFAULT_VERSION),
        }
        .expect("bollard");
        let request = NetworkCreateRequest {
            name: name.into(),
            driver: Some("bridge".into()),
            labels: Some(HashMap::from([(label.0.into(), label.1.into())])),
            ..NetworkCreateRequest::default()
        };
        docker.create_network(request).await.expect("create").id
    })
}

/// Removes the test network even if an assertion fails.
struct Cleanup<'a>(&'a DockerEngine, String);

impl Drop for Cleanup<'_> {
    fn drop(&mut self) {
        block_on(self.0.remove_network(&self.1)).ok();
    }
}

#[test]
#[ignore = "needs a running Docker engine"]
fn lists_networks_with_built_ins() {
    let engine = connect();
    let networks = block_on(engine.list_networks()).expect("list");
    let bridge = networks
        .iter()
        .find(|n| n.name == "bridge")
        .expect("bridge");
    assert!(bridge.is_built_in());
    assert!(!bridge.can_remove());
    for network in &networks {
        println!(
            "{} {} {} {:?} {} containers",
            network.short_id(),
            network.name,
            network.driver,
            network.subnet,
            network.containers
        );
    }
}

#[test]
#[ignore = "needs a running Docker engine"]
fn inspects_networks_with_their_containers() {
    let engine = connect();
    let networks = block_on(engine.list_networks()).expect("list");
    for network in networks.iter().filter(|n| n.driver == "bridge") {
        let detail = block_on(engine.inspect_network(&network.id)).expect("inspect");
        assert_eq!(detail.network.id, network.id);
        assert_eq!(detail.endpoints.len(), detail.network.containers);
        println!(
            "{} subnets {} gateways {}",
            network.name,
            detail.subnets_label(),
            detail.gateways_label()
        );
        for endpoint in &detail.endpoints {
            println!(
                "  {} {:?} {:?}",
                endpoint.name,
                endpoint.address(),
                endpoint.mac
            );
        }
    }
}

#[test]
#[ignore = "needs a running Docker engine"]
fn creates_and_removes_a_network() {
    let engine = connect();
    let name = format!("captain-agent-network-{}", std::process::id());
    let id = block_on(engine.create_network(&name)).expect("create");
    let _cleanup = Cleanup(&engine, id.clone());

    let networks = block_on(engine.list_networks()).expect("list");
    let created = networks.iter().find(|n| n.id == id).expect("new network");
    assert_eq!(created.name, name);
    assert_eq!(created.driver, "bridge");
    assert_eq!(created.containers, 0);
    assert!(created.can_remove());

    let detail = block_on(engine.inspect_network(&id)).expect("inspect");
    assert_eq!(detail.network.name, name);
    assert!(!detail.subnets.is_empty());
    assert!(detail.endpoints.is_empty());

    block_on(engine.remove_network(&id)).expect("remove");
    let networks = block_on(engine.list_networks()).expect("list");
    assert!(networks.iter().all(|n| n.id != id));
}

#[test]
#[ignore = "needs a running Docker engine"]
fn prunes_only_labelled_networks() {
    let engine = connect();
    let pid = std::process::id().to_string();
    let name = format!("captain-agent-prune-{pid}");
    let id = create_labelled(&name, ("captain-agent-test", &pid));
    let _cleanup = Cleanup(&engine, id.clone());

    let filter = format!("captain-agent-test={pid}");
    let removed = block_on(engine.prune_unused_networks(Some(&filter))).expect("prune");
    assert_eq!(removed, [name]);

    let networks = block_on(engine.list_networks()).expect("list");
    assert!(networks.iter().all(|n| n.id != id));
}

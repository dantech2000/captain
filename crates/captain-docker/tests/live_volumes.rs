//! Runs against the real engine that discovery finds. Ignored by default:
//! `cargo test -p captain-docker -- --ignored`. It only lists and inspects, and it
//! creates and removes volumes named `captain-agent-...`. The prune test prunes only
//! volumes with a `captain-agent-test` label that it put on its own volumes.

use std::collections::HashMap;

use bollard::models::VolumeCreateRequest;
use bollard::{API_DEFAULT_VERSION, Docker};
use captain_core::VolumeApi;
use captain_docker::{DiscoveryInput, DockerEngine, Endpoint, discover};
use futures::executor::block_on;

fn endpoint() -> Endpoint {
    discover(&DiscoveryInput::from_env(), |path| path.exists()).expect("discover")
}

fn connect() -> DockerEngine {
    DockerEngine::connect(endpoint()).expect("connect")
}

/// Creates a volume with `label` through bollard, because Captain creates volumes
/// without labels. `None` makes an anonymous volume. Returns the volume name.
fn create_labelled(name: Option<&str>, label: (&str, &str)) -> String {
    let runtime = tokio::runtime::Runtime::new().expect("runtime");
    runtime.block_on(async {
        let docker = match endpoint() {
            Endpoint::Tcp(address) => Docker::connect_with_http(&address, 30, API_DEFAULT_VERSION),
            other => Docker::connect_with_socket(&other.to_string(), 30, API_DEFAULT_VERSION),
        }
        .expect("bollard");
        let request = VolumeCreateRequest {
            name: name.map(Into::into),
            labels: Some(HashMap::from([(label.0.into(), label.1.into())])),
            ..VolumeCreateRequest::default()
        };
        docker.create_volume(request).await.expect("create").name
    })
}

/// Removes the test volumes even if an assertion fails.
struct Cleanup<'a>(&'a DockerEngine, Vec<String>);

impl Drop for Cleanup<'_> {
    fn drop(&mut self) {
        for name in &self.1 {
            block_on(self.0.remove_volume(name)).ok();
        }
    }
}

#[test]
#[ignore = "needs a running Docker engine"]
fn lists_volumes_with_usage() {
    let engine = connect();
    let volumes = block_on(engine.list_volumes()).expect("list");
    for volume in &volumes {
        assert!(!volume.name.is_empty());
        println!(
            "{} {:?} bytes, {:?} containers, project {:?}",
            volume.display_name(),
            volume.size_bytes,
            volume.containers,
            volume.compose_project
        );
    }
}

#[test]
#[ignore = "needs a running Docker engine"]
fn creates_and_removes_a_volume() {
    let engine = connect();
    let name = format!("captain-agent-volume-{}", std::process::id());
    block_on(engine.create_volume(&name)).expect("create");
    let _cleanup = Cleanup(&engine, vec![name.clone()]);

    let volumes = block_on(engine.list_volumes()).expect("list");
    let created = volumes.iter().find(|v| v.name == name).expect("new volume");
    assert_eq!(created.driver, "local");
    assert_eq!(created.scope, "local");
    assert!(!created.is_in_use());
    assert!(created.can_remove());

    block_on(engine.remove_volume(&name)).expect("remove");
    let volumes = block_on(engine.list_volumes()).expect("list");
    assert!(volumes.iter().all(|v| v.name != name));
}

#[test]
#[ignore = "needs a running Docker engine"]
fn lists_the_users_of_volumes() {
    let engine = connect();
    let name = format!("captain-agent-users-{}", std::process::id());
    block_on(engine.create_volume(&name)).expect("create");
    let _cleanup = Cleanup(&engine, vec![name.clone()]);
    assert!(
        block_on(engine.volume_users(&name))
            .expect("users")
            .is_empty()
    );

    // Read only: print who uses the volumes that are in use.
    let volumes = block_on(engine.list_volumes()).expect("list");
    for volume in volumes.iter().filter(|v| v.is_in_use()).take(3) {
        let users = block_on(engine.volume_users(&volume.name)).expect("users");
        assert!(!users.is_empty(), "{} has no users", volume.name);
        for user in users {
            println!(
                "{} <- {} at {}",
                volume.display_name(),
                user.name,
                user.destination
            );
        }
    }
}

#[test]
#[ignore = "needs a running Docker engine"]
fn prunes_only_labelled_volumes() {
    let engine = connect();
    let pid = std::process::id().to_string();
    let label = ("captain-agent-test", pid.as_str());
    let filter = format!("captain-agent-test={pid}");
    let named = format!("captain-agent-prune-{pid}");
    let anonymous = create_labelled(None, label);
    create_labelled(Some(&named), label);
    let _cleanup = Cleanup(&engine, vec![anonymous.clone(), named.clone()]);

    // Without `all`, the named volume stays.
    let first = block_on(engine.prune_unused_volumes(false, Some(&filter))).expect("prune");
    assert_eq!(first.removed, std::slice::from_ref(&anonymous));

    let second = block_on(engine.prune_unused_volumes(true, Some(&filter))).expect("prune");
    assert_eq!(second.removed, std::slice::from_ref(&named));

    let volumes = block_on(engine.list_volumes()).expect("list");
    assert!(
        volumes
            .iter()
            .all(|v| v.name != anonymous && v.name != named)
    );
}

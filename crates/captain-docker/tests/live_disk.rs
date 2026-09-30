//! Runs against the real engine that discovery finds. Ignored by default:
//! `cargo test -p captain-docker -- --ignored`. It only reads disk use; it never
//! prunes, because a prune would touch the user's own containers and cache.

use captain_core::ContainerApi;
use captain_docker::{DiscoveryInput, DockerEngine, discover};
use futures::executor::block_on;

#[test]
#[ignore = "needs a running Docker engine"]
fn disk_usage_lists_items_with_their_users() {
    let endpoint = discover(&DiscoveryInput::from_env(), |path| path.exists()).expect("discover");
    let engine = DockerEngine::connect(endpoint).expect("connect");
    let usage = block_on(engine.disk_usage()).expect("disk usage");
    let containers = block_on(engine.list_containers()).expect("containers");

    assert_eq!(usage.containers.len(), containers.len());
    if !usage.images.is_empty() {
        assert!(usage.images_bytes > 0);
    }
    let names: Vec<&str> = usage.volumes.iter().map(|v| v.name.as_str()).collect();
    for container in &usage.containers {
        for volume in &container.volumes {
            assert!(names.contains(&volume.as_str()), "{volume} is not listed");
        }
    }
}

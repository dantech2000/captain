//! Runs against the real engine that discovery finds. Ignored by default:
//! `cargo test -p captain-docker -- --ignored`.

use captain_core::Engine;
use captain_docker::{DiscoveryInput, DockerEngine, discover};
use futures::executor::block_on;

fn connect() -> DockerEngine {
    let endpoint = discover(&DiscoveryInput::from_env(), |path| path.exists()).expect("discover");
    DockerEngine::connect(endpoint).expect("connect")
}

#[test]
#[ignore = "needs a running Docker engine"]
fn reads_version_and_containers() {
    let engine = connect();
    let info = block_on(engine.info()).expect("info");
    assert!(!info.version.is_empty());

    let containers = block_on(engine.list_containers()).expect("list");
    println!(
        "{} {} {} containers",
        info.endpoint,
        info.version,
        containers.len()
    );
}

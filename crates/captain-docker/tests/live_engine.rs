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

#[test]
#[ignore = "needs a running Docker engine with a running container"]
fn inspects_and_streams_stats_of_a_running_container() {
    use futures::StreamExt;

    let engine = connect();
    let containers = block_on(engine.list_containers()).expect("list");
    let running = containers
        .iter()
        .find(|c| c.state.is_active())
        .expect("a running container");

    let detail = block_on(engine.inspect_container(&running.id)).expect("inspect");
    assert_eq!(detail.id, running.id);

    let samples: Vec<_> = block_on(engine.stats(&running.id).take(2).collect());
    let last = samples.last().expect("a sample").as_ref().expect("stats");
    assert!(last.memory_bytes > 0);
    println!(
        "{}: {} env vars, cpu {:.2}%, mem {} bytes",
        running.name,
        detail.env.len(),
        last.cpu_percent,
        last.memory_bytes
    );
}

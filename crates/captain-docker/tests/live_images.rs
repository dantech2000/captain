//! Image calls against the real engine that discovery finds. Ignored by default:
//! `cargo test -p captain-docker --test live_images -- --ignored`.
//!
//! These tests list and inspect images, and pull `hello-world`. They remove
//! `hello-world:latest` afterwards only if it was not there before. They never prune.
//! The run test creates `captain-agent-run` from `busybox` or `hello-world`, and
//! removes it again even when an assertion fails.

use captain_core::model::{ContainerAction, EnvVar, RestartPolicy, RunSpec};
use captain_core::store::PullTracker;
use captain_core::{ContainerApi, ImageApi};
use captain_docker::{DiscoveryInput, DockerEngine, discover};
use futures::StreamExt;
use futures::executor::block_on;

const TEST_IMAGE: &str = "hello-world:latest";

fn connect() -> DockerEngine {
    let endpoint = discover(&DiscoveryInput::from_env(), |path| path.exists()).expect("discover");
    DockerEngine::connect(endpoint).expect("connect")
}

fn has_test_image(engine: &DockerEngine) -> bool {
    let images = block_on(engine.list_images()).expect("list images");
    images
        .iter()
        .any(|image| image.repo_tags.iter().any(|tag| tag == TEST_IMAGE))
}

#[test]
#[ignore = "needs a running Docker engine"]
fn lists_images_with_usage_counts() {
    let engine = connect();
    let images = block_on(engine.list_images()).expect("list images");
    for image in &images {
        assert!(!image.id.is_empty());
        assert_eq!(image.dangling, image.repo_tags.is_empty());
    }
    let in_use = images.iter().filter(|image| image.in_use()).count();
    println!("{} images, {in_use} in use", images.len());
}

#[test]
#[ignore = "needs a running Docker engine and network access"]
fn pulls_and_removes_a_tiny_image() {
    let engine = connect();
    let existed = has_test_image(&engine);

    let mut tracker = PullTracker::new(TEST_IMAGE);
    let messages: Vec<_> = block_on(engine.pull_image("hello-world").collect());
    for message in &messages {
        tracker.apply(message.as_ref().expect("pull message"));
    }
    tracker.finish();
    println!(
        "{} messages, status {:?}, fraction {:.2}",
        messages.len(),
        tracker.status(),
        tracker.fraction()
    );
    assert!(tracker.status().starts_with("Status:"));
    assert_eq!(tracker.fraction(), 1.0);
    assert!(has_test_image(&engine));

    if !existed {
        block_on(engine.remove_image(TEST_IMAGE)).expect("remove the image this test pulled");
        assert!(!has_test_image(&engine));
    }
}

#[test]
#[ignore = "needs a running Docker engine and network access"]
fn a_bad_reference_ends_the_pull_with_an_error() {
    let engine = connect();
    let messages: Vec<_> = block_on(
        engine
            .pull_image("captain-agent-test/does-not-exist:nope")
            .collect(),
    );
    assert!(matches!(messages.last(), Some(Err(_))), "{messages:?}");
}

#[test]
#[ignore = "needs a running Docker engine"]
fn inspects_images_and_their_history() {
    let engine = connect();
    let images = block_on(engine.list_images()).expect("list images");
    // The pull test may remove the test image while this loop runs.
    let stable = images
        .iter()
        .filter(|image| !image.repo_tags.iter().any(|tag| tag == TEST_IMAGE));
    for image in stable {
        let detail = block_on(engine.inspect_image(&image.id)).expect("inspect image");
        assert_eq!(detail.id, image.id);
        assert!(!detail.platform().is_empty(), "{}", image.id);
        // An imported image has no history, and the engine answers null for it.
        let layers = block_on(engine.image_history(&image.id)).expect("image history");
        println!(
            "{} {} {} layers, {} ports, {} env",
            image.display_name(),
            detail.platform(),
            layers.len(),
            detail.config.exposed_ports.len(),
            detail.config.env.len()
        );
    }
    assert!(block_on(engine.inspect_image("captain-agent-test/missing:nope")).is_err());
}

const RUN_NAME: &str = "captain-agent-run";

/// Force-removes `captain-agent-run` when dropped. Missing is fine.
struct RunCleanup<'a>(&'a DockerEngine);

impl Drop for RunCleanup<'_> {
    fn drop(&mut self) {
        block_on(self.0.run_action(RUN_NAME, ContainerAction::ForceRemove)).ok();
    }
}

#[test]
#[ignore = "needs a running Docker engine; creates and removes captain-agent-run"]
fn runs_a_container_from_an_image() {
    let engine = connect();
    let images = block_on(engine.list_images()).expect("list images");
    let image = ["busybox:latest", TEST_IMAGE]
        .into_iter()
        .find(|name| images.iter().any(|i| i.repo_tags.iter().any(|t| t == name)))
        .expect("busybox or hello-world must be present");
    let cleanup = RunCleanup(&engine);
    // A container left behind by an earlier, killed run.
    block_on(engine.run_action(RUN_NAME, ContainerAction::ForceRemove)).ok();

    let spec = RunSpec {
        image: image.into(),
        name: Some(RUN_NAME.into()),
        env: vec![EnvVar::parse("CAPTAIN_TEST=1")],
        restart: RestartPolicy::No,
        ..RunSpec::default()
    };
    let id = block_on(engine.run_container(spec.clone())).expect("run container");
    let containers = block_on(engine.list_containers()).expect("list containers");
    let container = containers
        .iter()
        .find(|c| c.id == id)
        .expect("new container");
    assert_eq!(container.name, RUN_NAME);
    let detail = block_on(engine.inspect_container(&id)).expect("inspect container");
    assert!(detail.env.iter().any(|var| var.key == "CAPTAIN_TEST"));

    let again = block_on(engine.run_container(spec));
    assert!(again.is_err(), "a second run with the same name must fail");
    drop(cleanup);
    let containers = block_on(engine.list_containers()).expect("list containers");
    assert!(containers.iter().all(|c| c.id != id));
}

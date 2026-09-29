use futures::StreamExt;
use futures::executor::block_on;

use crate::model::{
    Container, ContainerState, Image, ImageDetail, ImageLayer, PullProgress, RunSpec,
};
use crate::{EngineError, FakeEngine, FakeImages, ImageApi};

fn engine() -> FakeEngine {
    let image = |id: &str, size, containers, dangling| Image {
        id: id.into(),
        repo_tags: if dangling {
            Vec::new()
        } else {
            vec![format!("{id}:latest")]
        },
        size,
        containers,
        dangling,
        ..Image::default()
    };
    FakeEngine {
        images: FakeImages {
            images: vec![
                image("nginx", 100, 1, false),
                image("old", 30, 0, true),
                image("older", 20, 0, true),
                image("kept", 50, 2, true),
            ],
            pull: vec![PullProgress::status("Pulling from library/busybox")],
            ..FakeImages::default()
        },
        ..FakeEngine::default()
    }
}

#[test]
fn prune_counts_unused_dangling_images() {
    assert_eq!(block_on(engine().prune_dangling_images()), Ok(50));
}

#[test]
fn remove_refuses_images_in_use() {
    let engine = engine();
    assert_eq!(block_on(engine.remove_image("old")), Ok(()));
    assert!(matches!(
        block_on(engine.remove_image("nginx")),
        Err(EngineError::Api(_))
    ));
    assert!(block_on(engine.remove_image("missing")).is_err());
}

#[test]
fn pull_streams_the_scripted_messages() {
    let engine = engine();
    let messages: Vec<_> = block_on(engine.pull_image("busybox").collect());
    assert_eq!(messages.len(), 1);
    assert!(messages[0].is_ok());

    let bad: Vec<_> = block_on(engine.pull_image("").collect());
    assert!(matches!(bad.as_slice(), [Err(EngineError::Api(_))]));
}

#[test]
fn inspect_and_history_need_a_known_image() {
    let mut engine = engine();
    engine.images.details = vec![ImageDetail {
        id: "nginx".into(),
        size: 100,
        ..ImageDetail::default()
    }];
    engine.images.history = vec![ImageLayer {
        created_by: "CMD [\"nginx\"]".into(),
        ..ImageLayer::default()
    }];
    assert_eq!(block_on(engine.inspect_image("nginx")).unwrap().size, 100);
    assert!(block_on(engine.inspect_image("old")).is_err());
    assert_eq!(block_on(engine.image_history("nginx")).unwrap().len(), 1);
    assert!(block_on(engine.image_history("missing")).is_err());
}

#[test]
fn run_needs_a_known_image_and_a_free_name() {
    let mut engine = engine();
    engine.containers = vec![Container {
        id: "c1".into(),
        name: "web".into(),
        image: "nginx:latest".into(),
        state: ContainerState::Running,
        status: "Up 1 minute".into(),
        ports: Vec::new(),
        created: 0,
        compose_project: None,
        compose: Default::default(),
        health: None,
    }];
    let spec = |image: &str, name: Option<&str>| RunSpec {
        image: image.into(),
        name: name.map(String::from),
        ..RunSpec::default()
    };
    assert_eq!(
        block_on(engine.run_container(spec("nginx:latest", Some("api")))),
        Ok(super::FAKE_RUN_ID.to_string())
    );
    assert!(block_on(engine.run_container(spec("nginx", None))).is_ok());
    assert!(block_on(engine.run_container(spec("ghost", None))).is_err());
    assert!(block_on(engine.run_container(spec("nginx", Some("web")))).is_err());
}

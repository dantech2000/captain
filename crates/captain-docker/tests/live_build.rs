//! Builds, tags, and removes an image through `docker buildx build` and the engine.
//! It writes a one-line Dockerfile in a temp folder, builds `captain-agent-build:test`
//! from `busybox`, tags it `captain-agent-build:tagged`, and removes both tags even
//! when an assertion fails. It never pushes.
//! Ignored by default: `cargo test -p captain-docker --test live_build -- --ignored`.

use std::path::PathBuf;

use captain_core::model::BuildSpec;
use captain_core::{ImageApi, ImageBuilder};
use captain_docker::{BuildCli, DiscoveryInput, DockerEngine, discover};
use futures::StreamExt;
use futures::executor::block_on;

const TAG: &str = "captain-agent-build:test";
const EXTRA_TAG: &str = "captain-agent-build:tagged";

/// Removes the temp folder and both tags when dropped.
struct Fixture<'a> {
    engine: &'a DockerEngine,
    dir: PathBuf,
}

impl Drop for Fixture<'_> {
    fn drop(&mut self) {
        for tag in [EXTRA_TAG, TAG] {
            block_on(self.engine.remove_image(tag)).ok();
        }
        std::fs::remove_dir_all(&self.dir).ok();
    }
}

#[test]
#[ignore = "needs a running Docker engine and the docker buildx CLI"]
fn builds_tags_and_removes_an_image() {
    let endpoint = discover(&DiscoveryInput::from_env(), |path| path.exists()).expect("discover");
    let builder = BuildCli::detect(&endpoint).expect("docker buildx");
    let engine = DockerEngine::connect(endpoint).expect("connect");
    let dir = std::env::temp_dir().join("captain-agent-build");
    std::fs::create_dir_all(&dir).expect("temp folder");
    std::fs::write(
        dir.join("Dockerfile"),
        "FROM busybox:latest\nARG MSG\nRUN echo $MSG > /msg\n",
    )
    .expect("Dockerfile");
    let fixture = Fixture {
        engine: &engine,
        dir: dir.clone(),
    };

    let spec = BuildSpec {
        context: dir,
        dockerfile: PathBuf::from("Dockerfile"),
        tag: TAG.into(),
        build_args: vec![("MSG".into(), "hello".into())],
        target: None,
    };
    let lines: Vec<_> = block_on(builder.build(&spec).collect());
    let failure = lines.iter().find_map(|line| line.as_ref().err());
    assert!(failure.is_none(), "build failed: {failure:?}");
    println!("buildx {}: {} lines", builder.version(), lines.len());

    block_on(engine.tag_image(TAG, EXTRA_TAG)).expect("tag");
    let images = block_on(engine.list_images()).expect("list images");
    let built = images
        .iter()
        .find(|image| image.repo_tags.iter().any(|tag| tag == TAG))
        .expect("the built image");
    assert!(built.repo_tags.iter().any(|tag| tag == EXTRA_TAG));

    let broken = BuildSpec {
        dockerfile: PathBuf::from("Missing.Dockerfile"),
        ..spec
    };
    let result: Vec<_> = block_on(builder.build(&broken).collect());
    let error = result.last().and_then(|line| line.as_ref().err()).cloned();
    assert!(error.is_some_and(|e| e.to_string().contains("ERROR")));
    drop(fixture);
}

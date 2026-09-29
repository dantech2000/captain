//! Installs an extension with a host binary, runs the binary through the bridge,
//! updates the extension to a second image, and removes it, in a temp extensions
//! folder. Both images are built here: `captain-agent-ext-host:test` and
//! `captain-agent-ext-host:latest`. Each has a page and a `darwin` shell script.
//! Ignored by default:
//! `cargo test -p captain-docker --test live_extension_host -- --ignored`.
#![cfg(target_os = "macos")]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use captain_core::extension::{
    BridgeEvent, BridgeRequest, ExecRequest, ExecScope, ExtensionManager, ExtensionPaths,
    InstalledExtension, UpdateCheck,
};
use captain_core::model::BuildSpec;
use captain_core::{ImageApi, ImageBuilder};
use captain_docker::{BuildCli, DiscoveryInput, DockerEngine, DockerExtensions, discover};
use futures::StreamExt;
use futures::executor::block_on;

const IMAGE: &str = "captain-agent-ext-host:test";
const NEWER: &str = "captain-agent-ext-host:latest";
const BINARY: &str = "captain-agent-hello";

/// Removes the extension, the temp folder, and the test images when dropped, so a
/// failed assertion leaves nothing behind.
struct Fixture {
    engine: DockerEngine,
    manager: DockerExtensions,
    dir: PathBuf,
    installed: Option<InstalledExtension>,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if let Some(extension) = self.installed.take() {
            block_on(self.manager.remove(extension)).ok();
        }
        for image in [IMAGE, NEWER] {
            block_on(self.engine.remove_image(image)).ok();
        }
        std::fs::remove_dir_all(&self.dir).ok();
    }
}

#[test]
#[ignore = "needs a running Docker engine and the docker CLI with buildx"]
fn runs_a_host_binary_and_updates_the_extension() {
    let endpoint = discover(&DiscoveryInput::from_env(), |path| path.exists()).expect("discover");
    let dir = std::env::temp_dir().join("captain-agent-ext-host");
    std::fs::remove_dir_all(&dir).ok();
    let mut fixture = Fixture {
        engine: DockerEngine::connect(endpoint.clone()).expect("connect"),
        manager: DockerExtensions::connect(&endpoint, ExtensionPaths::new(dir.join("ext")))
            .expect("extensions"),
        dir: dir.clone(),
        installed: None,
    };
    let builder = BuildCli::detect(&endpoint).expect("docker buildx");
    build(&builder, &dir.join("v1"), IMAGE, "1.0.0");

    let manager = &fixture.manager;
    let candidate = block_on(manager.prepare(IMAGE)).expect("prepare");
    assert_eq!(candidate.binary_names(), [BINARY]);
    let extension = block_on(manager.install(candidate)).expect("install");
    fixture.installed = Some(extension.clone());
    let manager = &fixture.manager;
    let binary = manager.paths().bin_dir(&extension.id).join(BINARY);
    let mode = std::fs::metadata(&binary)
        .expect("binary")
        .permissions()
        .mode();
    assert_eq!(
        mode & 0o111,
        0o111,
        "{} is not executable",
        binary.display()
    );
    assert_eq!(hello(manager, &extension), "hello world from 1.0.0");
    let refused = block_on(
        manager
            .call(&extension, exec(&binary.to_string_lossy()))
            .collect::<Vec<_>>(),
    );
    assert!(
        matches!(&refused[..], [BridgeEvent::Reject(_)]),
        "{refused:?}"
    );

    // Same image, same tag: nothing to update.
    let same = block_on(manager.check_update(extension.clone(), "test".into())).expect("check");
    assert!(matches!(same, UpdateCheck::UpToDate { .. }), "{same:?}");

    // `latest` exists only in the engine, so the failed pull falls back to it.
    build(&builder, &dir.join("v2"), NEWER, "2.0.0");
    let check = block_on(manager.check_update(extension.clone(), String::new())).expect("check");
    let UpdateCheck::Available(update) = check else {
        panic!("no update: {check:?}");
    };
    assert_eq!(update.candidate.labels.version, "2.0.0");
    let data = manager.paths().dir(&extension.id).join("data.txt");
    std::fs::write(&data, "kept").expect("data");
    let updated = block_on(manager.update(update.extension, update.candidate)).expect("update");
    fixture.installed = Some(updated.clone());
    let manager = &fixture.manager;
    assert_eq!(updated.image, NEWER);
    assert_eq!(hello(manager, &updated), "hello world from 2.0.0");
    assert!(
        data.is_file(),
        "the update deleted the extension's other files"
    );
    assert!(block_on(fixture.engine.inspect_image(IMAGE)).is_err());

    let extension = fixture.installed.take().expect("installed");
    block_on(fixture.manager.remove(extension.clone())).expect("remove");
    assert!(!fixture.manager.paths().dir(&extension.id).exists());
}

/// Runs the host binary through the bridge and returns its output.
fn hello(manager: &DockerExtensions, extension: &InstalledExtension) -> String {
    let events: Vec<_> = block_on(manager.call(extension, exec(BINARY)).collect());
    match &events[..] {
        [BridgeEvent::Resolve(result)] => result["stdout"].as_str().unwrap_or("").trim().into(),
        other => panic!("exec failed: {other:?}"),
    }
}

fn exec(cmd: &str) -> BridgeRequest {
    BridgeRequest::Exec {
        scope: ExecScope::Host,
        exec: ExecRequest {
            cmd: cmd.into(),
            args: vec!["world".into()],
            cwd: None,
            env: Default::default(),
            stream: false,
        },
    }
}

/// Builds an extension image `tag` from scratch: a page, `metadata.json`, and a
/// `darwin` script that prints "hello <arg> from <version>".
fn build(builder: &BuildCli, context: &Path, tag: &str, version: &str) {
    std::fs::create_dir_all(context.join("ui")).expect("context");
    std::fs::create_dir_all(context.join("darwin")).expect("context");
    std::fs::write(context.join("ui/index.html"), "<h1>host</h1>").expect("page");
    let script = format!("#!/bin/sh\necho \"hello $1 from {version}\"\n");
    std::fs::write(context.join("darwin").join(BINARY), script).expect("script");
    let metadata = format!(
        r#"{{"ui":{{"dashboard-tab":{{"title":"Host","root":"/ui","src":"index.html"}}}},"host":{{"binaries":[{{"darwin":[{{"path":"/darwin/{BINARY}"}}]}}]}}}}"#
    );
    std::fs::write(context.join("metadata.json"), metadata).expect("metadata");
    let dockerfile = format!(
        "FROM scratch\n\
         LABEL com.docker.desktop.extension.api.version=\">= 0.3.0\" \
         org.opencontainers.image.vendor=\"Captain tests\" \
         org.opencontainers.image.version=\"{version}\"\n\
         COPY metadata.json /metadata.json\nCOPY ui /ui\nCOPY darwin /darwin\n"
    );
    std::fs::write(context.join("Dockerfile"), dockerfile).expect("dockerfile");
    let spec = BuildSpec {
        context: context.to_path_buf(),
        dockerfile: "Dockerfile".into(),
        tag: tag.into(),
        build_args: Vec::new(),
        target: None,
    };
    let built: Vec<_> = block_on(builder.build(&spec).collect());
    assert!(built.iter().all(Result::is_ok), "{built:?}");
}

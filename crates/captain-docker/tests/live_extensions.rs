//! Installs and removes two extensions on a live engine, in a temp extensions
//! folder. The UI-only one is Docker's Disk Usage extension, re-tagged
//! `captain-agent-ext-ui`. The one with a backend is built here as
//! `captain-agent-ext-vm`: socat answers HTTP on a socket in `/run/guest-services`.
//! Ignored by default: `cargo test -p captain-docker --test live_extensions -- --ignored`.

use std::path::PathBuf;

use captain_core::extension::{
    BridgeEvent, BridgeRequest, ExecRequest, ExecScope, ExtensionManager, ExtensionPaths,
    InstalledExtension, ListOptions, ServiceRequest, UpdateCheck,
};
use captain_core::model::BuildSpec;
use captain_core::{ImageApi, ImageBuilder};
use captain_docker::{BuildCli, DiscoveryInput, DockerEngine, DockerExtensions, discover};
use futures::StreamExt;
use futures::executor::block_on;

const UI_SOURCE: &str = "docker/disk-usage-extension:0.2.9";
const UI_IMAGE: &str = "captain-agent-ext-ui:latest";
const VM_IMAGE: &str = "captain-agent-ext-vm:test";
/// The same repository, with a backend image that does not exist.
const BROKEN_IMAGE: &str = "captain-agent-ext-vm:broken";

/// Removes the installed extensions, the temp folder, and the test images when
/// dropped, so a failed assertion leaves nothing behind.
struct Fixture {
    engine: DockerEngine,
    manager: DockerExtensions,
    dir: PathBuf,
    images: Vec<&'static str>,
    installed: Vec<InstalledExtension>,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        for extension in self.installed.drain(..) {
            block_on(self.manager.remove(extension)).ok();
        }
        for image in &self.images {
            block_on(self.engine.remove_image(image)).ok();
        }
        std::fs::remove_dir_all(&self.dir).ok();
    }
}

fn setup(name: &str) -> Fixture {
    let endpoint = discover(&DiscoveryInput::from_env(), |path| path.exists()).expect("discover");
    let engine = DockerEngine::connect(endpoint.clone()).expect("connect");
    let dir = std::env::temp_dir().join(name);
    std::fs::remove_dir_all(&dir).ok();
    let manager =
        DockerExtensions::connect(&endpoint, ExtensionPaths::new(dir.clone())).expect("extensions");
    Fixture {
        engine,
        manager,
        dir,
        images: Vec::new(),
        installed: Vec::new(),
    }
}

fn call(
    manager: &DockerExtensions,
    extension: &InstalledExtension,
    request: BridgeRequest,
) -> Vec<BridgeEvent> {
    block_on(manager.call(extension, request).collect())
}

fn exec(scope: ExecScope, cmd: &str, args: &[&str], stream: bool) -> BridgeRequest {
    BridgeRequest::Exec {
        scope,
        exec: ExecRequest {
            cmd: cmd.into(),
            args: args.iter().map(ToString::to_string).collect(),
            cwd: None,
            env: Default::default(),
            stream,
        },
    }
}

#[test]
#[ignore = "needs a running Docker engine, the docker CLI, and Docker Hub"]
fn installs_and_removes_a_ui_only_extension() {
    let mut fixture = setup("captain-agent-ext-ui");
    fixture.images.push(UI_IMAGE);
    let had_source = block_on(fixture.engine.inspect_image(UI_SOURCE)).is_ok();
    block_on(fixture.engine.pull_image(UI_SOURCE).collect::<Vec<_>>());
    block_on(fixture.engine.tag_image(UI_SOURCE, UI_IMAGE)).expect("tag");
    if !had_source {
        block_on(fixture.engine.remove_image(UI_SOURCE)).expect("untag");
    }

    let manager = &fixture.manager;
    let candidate = block_on(manager.prepare("captain-agent-ext-ui")).expect("prepare");
    assert_eq!(candidate.labels.title, "Disk Usage");
    assert_eq!(candidate.metadata.backend(&candidate.image), None);
    let extension = block_on(manager.install(candidate)).expect("install");
    fixture.installed.push(extension.clone());
    let manager = &fixture.manager;
    let page = manager.paths().ui_dir(&extension.id).join("index.html");
    assert!(page.is_file(), "{} is missing", page.display());
    assert_eq!(
        block_on(manager.list()).expect("list"),
        std::slice::from_ref(&extension)
    );

    let version = call(
        manager,
        &extension,
        exec(
            ExecScope::Docker,
            "version",
            &["--format", "\"{{.Server.Version}}\""],
            false,
        ),
    );
    assert!(
        matches!(&version[..], [BridgeEvent::Resolve(result)] if result["code"] == 0),
        "{version:?}"
    );
    let lines = call(
        manager,
        &extension,
        exec(ExecScope::Docker, "version", &[], true),
    );
    assert!(
        matches!(lines.last(), Some(BridgeEvent::Exit(0))),
        "{lines:?}"
    );
    let listed = call(
        manager,
        &extension,
        BridgeRequest::ListImages(ListOptions::default()),
    );
    assert!(matches!(&listed[..], [BridgeEvent::Resolve(images)] if images.is_array()));
    let refused = call(manager, &extension, exec(ExecScope::Host, "sh", &[], false));
    assert!(matches!(&refused[..], [BridgeEvent::Reject(_)]));

    fixture.installed.clear();
    block_on(manager.remove(extension.clone())).expect("remove");
    assert!(!manager.paths().dir(&extension.id).exists());
    assert!(block_on(fixture.engine.inspect_image(UI_IMAGE)).is_err());
}

const METADATA: &str = r#"{"vm":{"image":"${DESKTOP_PLUGIN_IMAGE}","exposes":{"socket":"backend.sock"}},"ui":{"dashboard-tab":{"title":"Agent","root":"/ui","src":"index.html"}}}"#;
const BROKEN_METADATA: &str = r#"{"vm":{"image":"captain-agent-ext-missing:none","exposes":{"socket":"backend.sock"}},"ui":{"dashboard-tab":{"title":"Agent","root":"/ui","src":"index.html"}}}"#;

/// Builds an extension image `tag` on socat that answers HTTP on the backend socket.
fn build_vm(fixture: &Fixture, tag: &str, metadata: &str) {
    let context = fixture.dir.join("build").join(tag.replace(':', "-"));
    std::fs::create_dir_all(context.join("ui")).expect("context");
    std::fs::write(context.join("ui/index.html"), "<h1>agent</h1>").expect("page");
    std::fs::write(context.join("metadata.json"), metadata).expect("metadata");
    let dockerfile = "FROM alpine/socat:1.8.1.3\n\
        LABEL com.docker.desktop.extension.api.version=\">= 0.3.0\" org.opencontainers.image.vendor=\"Captain tests\"\n\
        COPY metadata.json /metadata.json\nCOPY ui /ui\n\
        ENTRYPOINT [\"socat\", \"UNIX-LISTEN:/run/guest-services/backend.sock,fork\", \"SYSTEM:echo HTTP/1.0 200 OK; echo; echo hello\"]\n";
    std::fs::write(context.join("Dockerfile"), dockerfile).expect("dockerfile");
    let endpoint = discover(&DiscoveryInput::from_env(), |path| path.exists()).expect("discover");
    let builder = BuildCli::detect(&endpoint).expect("docker buildx");
    let spec = BuildSpec {
        context: context.clone(),
        dockerfile: "Dockerfile".into(),
        tag: tag.into(),
        build_args: Vec::new(),
        target: None,
    };
    let built: Vec<_> = block_on(builder.build(&spec).collect());
    assert!(built.iter().all(Result::is_ok), "{built:?}");
}

/// Calls the backend through the proxy until it answers, and returns the answer.
fn hello(manager: &DockerExtensions, extension: &InstalledExtension) -> Vec<BridgeEvent> {
    let service = ServiceRequest {
        method: "GET".into(),
        path: "/hello".into(),
        headers: Default::default(),
        body: None,
    };
    let mut answer = Vec::new();
    // The proxy may need a moment before it accepts connections.
    for _ in 0..20 {
        answer = call(manager, extension, BridgeRequest::Service(service.clone()));
        if matches!(&answer[..], [BridgeEvent::Resolve(_)]) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    answer
}

fn answers_hello(answer: &[BridgeEvent]) -> bool {
    matches!(answer, [BridgeEvent::Resolve(body)] if body.as_str().unwrap_or("").contains("hello"))
}

#[test]
#[ignore = "needs a running Docker engine, the docker CLI with Compose and buildx, and Docker Hub"]
fn installs_an_extension_with_a_backend() {
    let mut fixture = setup("captain-agent-ext-vm");
    fixture.images.extend([VM_IMAGE, BROKEN_IMAGE]);
    build_vm(&fixture, VM_IMAGE, METADATA);

    let candidate = block_on(fixture.manager.prepare(VM_IMAGE)).expect("prepare");
    let extension = block_on(fixture.manager.install(candidate)).expect("install");
    fixture.installed.push(extension.clone());
    let manager = &fixture.manager;
    let answer = hello(manager, &extension);
    assert!(answers_hello(&answer), "{answer:?}");
    let listed = call(
        manager,
        &extension,
        exec(ExecScope::Vm, "ls", &["/run/guest-services"], false),
    );
    assert!(
        matches!(&listed[..], [BridgeEvent::Resolve(r)] if r["stdout"].as_str().unwrap_or("").contains("backend.sock")),
        "{listed:?}"
    );

    // Installing the same repository again is refused, and the backend keeps running.
    let again = block_on(manager.prepare(VM_IMAGE)).expect("prepare");
    let refused = block_on(manager.install(again)).expect_err("a second install");
    assert!(
        refused.to_string().contains("already installed"),
        "{refused}"
    );
    assert!(answers_hello(&hello(manager, &extension)));

    // Another engine neither lists nor removes it.
    let endpoint = discover(&DiscoveryInput::from_env(), |path| path.exists()).expect("discover");
    let other = DockerExtensions::connect(&endpoint, manager.paths().clone())
        .expect("extensions")
        .with_label("unix:///captain-agent-other.sock");
    assert_eq!(block_on(other.list()).expect("list"), []);
    assert!(block_on(other.remove(extension.clone())).is_err());

    // An update whose backend cannot start puts the old version back.
    build_vm(&fixture, BROKEN_IMAGE, BROKEN_METADATA);
    let check = block_on(manager.check_update(extension.clone(), "broken".into())).expect("check");
    let UpdateCheck::Available(update) = check else {
        panic!("no update: {check:?}");
    };
    let failed = block_on(manager.update(update.extension, update.candidate)).expect_err("update");
    assert!(!failed.to_string().contains("restore"), "{failed}");
    assert_eq!(
        block_on(manager.list()).expect("list"),
        std::slice::from_ref(&extension)
    );
    let answer = hello(manager, &extension);
    assert!(answers_hello(&answer), "{answer:?}");

    assert!(fixture.installed.pop().is_some());
    block_on(manager.remove(extension)).expect("remove");
}

//! Runs k3s in a throwaway Captain Engine VM, runs a pod from an image built with
//! `docker build`, forwards its Service, and resets the cluster. It downloads an
//! Ubuntu image (unless Lima has it) and k3s (about 250 MB), so it is ignored by
//! default. Run it with a kubeconfig of its own, never your real one:
//!
//! ```sh
//! mkdir -p ~/.ck8 && PATH=~/.rd/bin:$PATH KUBECONFIG=~/.ck8/kubeconfig \
//!   cargo test -p captain-kube --test live_kubernetes -- --ignored --nocapture
//! ```
//!
//! It uses a VM named `captain-agent-k8s` in `~/.ck8/lima` and deletes both at the
//! end, also when it fails.
#![cfg(target_os = "macos")]

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use captain_core::kubernetes::{
    CONTEXT, ForwardKey, KubernetesSettings, PortForwarding, load_contexts, user_kubeconfig_paths,
};
use captain_core::{EngineHost, GIB, HostResources};
use captain_host::{LimaHost, LimaPaths};
use captain_kube::KubeForwarder;
use futures::StreamExt;
use futures::executor::block_on;

const NAME: &str = "captain-agent-k8s";

struct Cleanup {
    host: LimaHost,
    root: PathBuf,
}

impl Drop for Cleanup {
    fn drop(&mut self) {
        if let Err(error) = block_on(self.host.reset()) {
            eprintln!("reset failed: {error}");
        }
        std::fs::remove_dir_all(&self.root).ok();
    }
}

#[test]
#[ignore = "creates a real Lima VM and downloads k3s"]
fn runs_a_local_image_in_k3s_and_forwards_its_port() {
    tracing_subscriber::fmt().with_env_filter("warn").init();
    let home = std::env::home_dir().unwrap();
    let root = home.join(".ck8");
    let kubeconfig = std::env::var_os("KUBECONFIG").map(PathBuf::from);
    assert!(
        kubeconfig
            .as_deref()
            .is_some_and(|path| path.starts_with(&root)),
        "set KUBECONFIG to a file in {}",
        root.display()
    );
    let paths = LimaPaths {
        lima_home: root.join("lima"),
        instance: NAME.into(),
    };
    let resources = HostResources {
        cpus: 2,
        memory_bytes: 4 * GIB,
        disk_bytes: 20 * GIB,
    };
    let host = LimaHost::with_paths(paths.clone(), resources);
    let _cleanup = Cleanup {
        host: host.clone(),
        root: root.clone(),
    };
    let kubernetes = host.kubernetes().unwrap();
    let versions = block_on(kubernetes.versions(true)).unwrap();
    let stable = versions.stable().expect("a stable version").to_string();
    eprintln!("k3s stable: {stable}");
    host.set_kubernetes(KubernetesSettings {
        enabled: true,
        version: Some(stable.clone()),
        ..KubernetesSettings::default()
    });

    let started = Instant::now();
    let mut progress = host.start();
    while let Some(line) = block_on(progress.next()) {
        let line = line.expect("the engine starts");
        eprintln!("progress: {line}");
        assert!(!line.contains("Kubernetes did not start"), "{line}");
    }
    eprintln!("engine and k3s up in {:?}", started.elapsed());
    let status = block_on(kubernetes.status()).unwrap();
    assert_eq!(status.version(), Some(stable.as_str()), "{status:?}");
    let contexts = load_contexts(&user_kubeconfig_paths());
    assert!(contexts.names.iter().any(|name| name == CONTEXT));

    docker_build(&paths.docker_socket());
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let client = runtime.block_on(kube_client(&kubernetes.kubeconfig()));
    runtime.block_on(apply(client.clone()));
    runtime.block_on(wait_running(client));

    let forwarder = KubeForwarder::new(kubernetes.kubeconfig()).unwrap();
    let services = block_on(forwarder.services()).unwrap();
    assert!(services.iter().any(|service| service.name == NAME));
    let key = ForwardKey {
        namespace: "default".into(),
        service: NAME.into(),
        port: 80,
    };
    let forward = block_on(forwarder.forward(key.clone(), None)).unwrap();
    let body = http_get(forward.local_port);
    assert!(body.contains("hello from captain"), "{body}");
    forwarder.stop(&key);
    assert!(forwarder.forwards().is_empty());

    let mut reset = kubernetes.reset();
    while let Some(line) = block_on(reset.next()) {
        eprintln!("reset: {}", line.expect("the reset works"));
    }
    let services = block_on(forwarder.services()).unwrap();
    assert!(!services.iter().any(|service| service.name == NAME));
}

/// Builds `captain-agent-k8s:test`, a busybox web server, in the test VM.
fn docker_build(socket: &Path) {
    let dir = std::env::temp_dir().join(format!("{NAME}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("Dockerfile"),
        "FROM busybox\nRUN mkdir /www && echo 'hello from captain' > /www/index.html\n\
         CMD [\"httpd\", \"-f\", \"-p\", \"8080\", \"-h\", \"/www\"]\n",
    )
    .unwrap();
    let status = Command::new("docker")
        .env("DOCKER_HOST", format!("unix://{}", socket.display()))
        .args(["build", "-q", "-t", &format!("{NAME}:test")])
        .arg(&dir)
        .status()
        .expect("docker runs");
    std::fs::remove_dir_all(&dir).ok();
    assert!(status.success());
}

async fn kube_client(kubeconfig: &Path) -> kube::Client {
    let file = kube::config::Kubeconfig::read_from(kubeconfig).unwrap();
    let options = kube::config::KubeConfigOptions::default();
    let config = kube::Config::from_custom_kubeconfig(file, &options)
        .await
        .unwrap();
    kube::Client::try_from(config).unwrap()
}

/// A pod with the local image (never pulled) and a Service with a named target port.
async fn apply(client: kube::Client) {
    use k8s_openapi::api::core::v1::{Pod, Service};
    let pod: Pod = serde_json::from_value(serde_json::json!({
        "metadata": {"name": NAME, "labels": {"app": NAME}},
        "spec": {"containers": [{
            "name": "web", "image": format!("{NAME}:test"), "imagePullPolicy": "Never",
            "ports": [{"name": "http", "containerPort": 8080}]
        }]}
    }))
    .unwrap();
    let service: Service = serde_json::from_value(serde_json::json!({
        "metadata": {"name": NAME},
        "spec": {"selector": {"app": NAME}, "ports": [{"port": 80, "targetPort": "http"}]}
    }))
    .unwrap();
    let params = kube::api::PostParams::default();
    let pods: kube::Api<Pod> = kube::Api::default_namespaced(client.clone());
    pods.create(&params, &pod).await.unwrap();
    let services: kube::Api<Service> = kube::Api::default_namespaced(client);
    services.create(&params, &service).await.unwrap();
}

async fn wait_running(client: kube::Client) {
    use k8s_openapi::api::core::v1::Pod;
    let pods: kube::Api<Pod> = kube::Api::default_namespaced(client);
    let deadline = Instant::now() + Duration::from_secs(180);
    while Instant::now() < deadline {
        let pod = pods.get(NAME).await.unwrap();
        let phase = pod.status.and_then(|status| status.phase);
        if phase.as_deref() == Some("Running") {
            return;
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    panic!("the pod did not run");
}

fn http_get(port: u16) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(30)))
        .unwrap();
    stream
        .write_all(b"GET / HTTP/1.0\r\nHost: localhost\r\n\r\n")
        .unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    response
}

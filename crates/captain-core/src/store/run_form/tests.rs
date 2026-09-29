use super::{PortRow, RunForm, RunFormError};
use crate::model::{
    EnvVar, ExposedPort, ImageConfig, ImageDetail, PublishPort, RestartPolicy, RunSpec,
};
use crate::store::NameError;

fn nginx() -> ImageDetail {
    ImageDetail {
        config: ImageConfig {
            exposed_ports: vec![
                ExposedPort::parse("80/tcp").unwrap(),
                ExposedPort::parse("53/udp").unwrap(),
            ],
            env: vec![EnvVar::parse("NGINX_VERSION=1.27")],
            ..ImageConfig::default()
        },
        ..ImageDetail::default()
    }
}

fn row(port: &str, host: &str) -> PortRow {
    PortRow {
        container: ExposedPort::parse(port).unwrap(),
        host: host.into(),
    }
}

#[test]
fn prefills_ports_and_env_from_the_image() {
    let form = RunForm::from_image("nginx:1.27", &nginx());
    assert_eq!(form.image, "nginx:1.27");
    assert_eq!(form.ports, [row("80/tcp", "80"), row("53/udp", "53")]);
    assert_eq!(form.env, ["NGINX_VERSION=1.27"]);
    assert_eq!(form.restart, RestartPolicy::No);
    assert!(!form.auto_remove);
}

#[test]
fn builds_a_spec() {
    let mut form = RunForm::from_image("nginx:1.27", &nginx());
    form.name = " web ".into();
    form.ports[1].host = String::new();
    form.env.push(String::new());
    form.env.push("DEBUG=".into());
    form.auto_remove = true;
    form.restart = RestartPolicy::UnlessStopped;
    assert_eq!(form.to_spec(), Err(RunFormError::AutoRemoveWithRestart));
    form.auto_remove = false;
    let spec = form.to_spec().unwrap();
    assert_eq!(
        spec,
        RunSpec {
            image: "nginx:1.27".into(),
            name: Some("web".into()),
            ports: vec![PublishPort {
                host: 80,
                container: 80,
                protocol: "tcp".into(),
            }],
            env: vec![EnvVar::parse("NGINX_VERSION=1.27"), EnvVar::parse("DEBUG=")],
            auto_remove: false,
            restart: RestartPolicy::UnlessStopped,
        }
    );
    assert_eq!(spec.ports[0].container_key(), "80/tcp");
}

#[test]
fn an_empty_name_lets_the_engine_pick() {
    let form = RunForm::from_image("busybox", &ImageDetail::default());
    assert_eq!(form.to_spec().unwrap().name, None);
}

#[test]
fn rejects_bad_names() {
    let mut form = RunForm::from_image("busybox", &ImageDetail::default());
    form.name = "-web".into();
    assert_eq!(form.to_spec(), Err(RunFormError::Name(NameError::BadStart)));
    form.name = "my web".into();
    assert_eq!(
        form.to_spec(),
        Err(RunFormError::Name(NameError::BadChar(' ')))
    );
}

#[test]
fn rejects_bad_and_duplicate_host_ports() {
    let mut form = RunForm::from_image("nginx", &nginx());
    for bad in ["0", "65536", "http", "-1"] {
        form.ports[0].host = bad.into();
        assert_eq!(
            form.to_spec(),
            Err(RunFormError::Port("80/tcp".into())),
            "{bad}"
        );
    }
    form.ports = vec![row("80/tcp", "8080"), row("81/tcp", "8080")];
    assert_eq!(
        form.to_spec(),
        Err(RunFormError::DuplicatePort(8080, "tcp".into()))
    );
    // The same number on another protocol is a different port.
    form.ports = vec![row("53/tcp", "53"), row("53/udp", "53")];
    assert!(form.to_spec().is_ok());
}

#[test]
fn rejects_env_lines_without_a_key() {
    let mut form = RunForm::from_image("busybox", &ImageDetail::default());
    for bad in ["NOVALUE", "=value", "MY KEY=1"] {
        form.env = vec![bad.into()];
        assert_eq!(form.to_spec(), Err(RunFormError::Env(bad.into())), "{bad}");
    }
    form.env = vec!["URL=http://x?a=b".into()];
    assert_eq!(form.to_spec().unwrap().env[0].value, "http://x?a=b");
}

#[test]
fn errors_read_as_sentences() {
    assert_eq!(
        RunFormError::Port("80/tcp".into()).to_string(),
        "Host port for 80/tcp: enter a number from 1 to 65535, or leave it empty"
    );
    assert_eq!(
        RunFormError::Name(NameError::TooShort).to_string(),
        "Name: Use at least 2 characters"
    );
}

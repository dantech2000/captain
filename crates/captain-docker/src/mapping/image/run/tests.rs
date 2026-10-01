use bollard::models::{PortBinding, RestartPolicyNameEnum};
use captain_core::model::{EnvVar, PublishPort, RestartPolicy, RunSpec};

use super::run_body;

fn port(host: u16, container: u16, protocol: &str) -> PublishPort {
    PublishPort {
        host,
        container,
        protocol: protocol.into(),
    }
}

#[test]
fn maps_ports_env_volumes_and_policies() {
    let spec = RunSpec {
        image: "nginx:1.27".into(),
        name: Some("web".into()),
        ports: vec![port(8080, 80, "tcp"), port(8443, 443, "tcp")],
        env: vec![EnvVar::parse("A=1"), EnvVar::parse("B=x=y")],
        volumes: vec!["data:/data".into()],
        auto_remove: false,
        restart: RestartPolicy::UnlessStopped,
    };
    let body = run_body(&spec);
    assert_eq!(body.image.as_deref(), Some("nginx:1.27"));
    assert_eq!(body.env, Some(vec!["A=1".into(), "B=x=y".into()]));
    assert_eq!(
        body.exposed_ports,
        Some(vec!["443/tcp".into(), "80/tcp".into()])
    );

    let host = body.host_config.expect("host config");
    let bindings = host.port_bindings.expect("port bindings");
    assert_eq!(
        bindings["80/tcp"],
        Some(vec![PortBinding {
            host_ip: None,
            host_port: Some("8080".into()),
        }])
    );
    assert_eq!(host.binds, Some(vec!["data:/data".into()]));
    assert_eq!(host.auto_remove, Some(false));
    assert_eq!(
        host.restart_policy.and_then(|p| p.name),
        Some(RestartPolicyNameEnum::UNLESS_STOPPED)
    );
}

#[test]
fn one_container_port_can_have_two_host_ports() {
    let spec = RunSpec {
        image: "busybox".into(),
        ports: vec![port(8080, 80, "tcp"), port(9090, 80, "tcp")],
        ..RunSpec::default()
    };
    let bindings = run_body(&spec).host_config.unwrap().port_bindings.unwrap();
    assert_eq!(bindings["80/tcp"].as_ref().map(Vec::len), Some(2));
}

#[test]
fn a_bare_spec_sets_no_ports_or_env() {
    let spec = RunSpec {
        image: "busybox".into(),
        auto_remove: true,
        ..RunSpec::default()
    };
    let body = run_body(&spec);
    assert_eq!(body.env, None);
    assert_eq!(body.exposed_ports, None);
    let host = body.host_config.unwrap();
    assert_eq!(host.port_bindings, None);
    assert_eq!(host.auto_remove, Some(true));
    assert_eq!(
        host.restart_policy.and_then(|p| p.name),
        Some(RestartPolicyNameEnum::NO)
    );
}

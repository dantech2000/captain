use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::process::Command;

use captain_core::extension::{ExecRequest, ExecScope};

use super::{check, point_at};

fn exec(cmd: &str, env: &[(&str, &str)]) -> ExecRequest {
    ExecRequest {
        cmd: cmd.into(),
        args: vec!["ps".into()],
        cwd: None,
        env: env
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
        stream: false,
    }
}

#[test]
fn global_options_and_daemon_variables_are_refused() {
    for cmd in ["--host=tcp://evil:2375", "-H", "--context", "--config", ""] {
        assert!(check(ExecScope::Docker, &exec(cmd, &[])).is_err(), "{cmd}");
    }
    for name in [
        "DOCKER_HOST",
        "docker_context",
        "DOCKER_CONFIG",
        "DOCKER_TLS_VERIFY",
        "DOCKER_CERT_PATH",
        "DOCKER_CUSTOM_HEADERS",
    ] {
        assert!(
            check(ExecScope::Vm, &exec("ps", &[(name, "x")])).is_err(),
            "{name}"
        );
    }
    let allowed = exec(
        "run",
        &[
            ("DOCKER_API_VERSION", "1.45"),
            ("DOCKER_BUILDKIT", "1"),
            ("FOO", "bar"),
        ],
    );
    assert!(check(ExecScope::Docker, &allowed).is_ok());
}

#[test]
fn the_engine_wins_over_the_page_environment() {
    let env = BTreeMap::from([
        ("DOCKER_HOST".to_string(), "tcp://evil:2375".to_string()),
        ("DOCKER_CONTEXT".to_string(), "evil".to_string()),
    ]);
    let mut command = Command::new("docker");
    point_at(&mut command, "unix:///captain.sock", &env);
    let envs: BTreeMap<_, _> = command.get_envs().collect();
    assert_eq!(
        envs[OsStr::new("DOCKER_HOST")],
        Some(OsStr::new("unix:///captain.sock"))
    );
    assert_eq!(envs[OsStr::new("DOCKER_CONTEXT")], None);
}

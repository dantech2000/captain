use captain_core::daemon::{DaemonSettings, DaemonState};
use serde_json::json;

use super::{parse_state, read_args, write_args, write_input};

#[test]
fn reads_daemon_json_and_the_tcp_port() {
    let output = "{\n  \"features\": {\"cdi\": true}\n}\n\n#captain-tcp\n[Service]\nExecStart=\n\
                  ExecStart=/usr/bin/dockerd -H fd:// --containerd=/run/containerd/containerd.sock -H tcp://127.0.0.1:2376\n";
    assert_eq!(
        parse_state(output),
        Some(DaemonState {
            daemon_json: json!({"features": {"cdi": true}}),
            tcp_port: Some(2376),
        })
    );
}

#[test]
fn no_drop_in_means_no_tcp_and_bad_json_means_unknown() {
    let state = parse_state("{}\n\n#captain-tcp\n").unwrap();
    assert_eq!(state.tcp_port, None);
    assert_eq!(parse_state("{\"broken\"\n#captain-tcp\n"), None);
}

#[test]
fn write_runs_as_root_with_the_port_last_and_json_on_stdin() {
    let settings = DaemonSettings {
        tcp: true,
        ..DaemonSettings::default()
    };
    let state = settings.state();
    let args = write_args("captain", &state);
    assert_eq!(
        args[..6],
        ["shell", "--workdir", "/", "captain", "sudo", "-n"]
    );
    assert_eq!(args.last().map(String::as_str), Some("2375"));
    let input = write_input(&state);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&input).unwrap(),
        state.daemon_json
    );

    let off = write_args("captain", &DaemonSettings::default().state());
    assert_eq!(off.last().map(String::as_str), Some("0"));
    assert!(!read_args("captain").contains(&"sudo".to_string()));
}

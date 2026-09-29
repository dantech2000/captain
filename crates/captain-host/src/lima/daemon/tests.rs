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

/// Runs the write script against a fake root in a temporary folder, where the first
/// `systemctl daemon-reload` fails after `daemon.json` was replaced.
#[cfg(unix)]
#[test]
fn a_failure_after_the_backup_puts_both_files_back() {
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;

    let root = std::env::temp_dir().join(format!("captain-daemon-{}", std::process::id()));
    std::fs::remove_dir_all(&root).ok();
    let root_text = root.display().to_string();
    for dir in ["bin", "etc/docker", "lib/systemd/system"] {
        std::fs::create_dir_all(root.join(dir)).unwrap();
    }
    std::fs::write(root.join("etc/docker/daemon.json"), "{\"old\": true}\n").unwrap();
    std::fs::write(
        root.join("lib/systemd/system/docker.service"),
        "ExecStart=/usr/bin/dockerd -H fd://\n",
    )
    .unwrap();
    let systemctl = format!(
        "#!/bin/sh\necho \"$*\" >> {root_text}/calls\n\
         if [ \"$1\" = daemon-reload ] && [ ! -f {root_text}/failed ]; then touch {root_text}/failed; exit 1; fi\n"
    );
    for (name, body) in [
        ("systemctl", systemctl.as_str()),
        ("dockerd", "#!/bin/sh\n"),
    ] {
        let path = root.join("bin").join(name);
        std::fs::write(&path, body).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let script = super::WRITE_SCRIPT
        .replace("/etc/", &format!("{root_text}/etc/"))
        .replace("/lib/systemd/", &format!("{root_text}/lib/systemd/"));
    let path = format!(
        "{root_text}/bin:{}",
        std::env::var("PATH").unwrap_or_default()
    );
    let mut child = std::process::Command::new("/bin/sh")
        .args(["-c", &script, "sh", "2375"])
        .env("PATH", path)
        .stdin(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"{\"new\": true}\n")
        .unwrap();
    assert!(!child.wait().unwrap().success());

    let read = |path: &str| std::fs::read_to_string(root.join(path)).unwrap_or_default();
    assert_eq!(read("etc/docker/daemon.json"), "{\"old\": true}\n");
    assert!(
        !root
            .join("etc/systemd/system/docker.service.d/captain-tcp.conf")
            .exists()
    );
    assert_eq!(read("calls"), "daemon-reload\ndaemon-reload\n");
    std::fs::remove_dir_all(&root).ok();
}

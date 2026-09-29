use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use captain_core::ssh::SshTarget;

use std::sync::Mutex;

use super::{Slot, SshTunnel, open_in};

/// A temp dir with a stub `ssh` script. The stub logs its arguments to `calls`.
struct Stub(PathBuf);

impl Stub {
    fn new(test: &str, body: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("captain-stub-{test}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let script = dir.join("ssh");
        let calls = dir.join("calls");
        fs::write(
            &script,
            format!("#!/bin/sh\necho \"$@\" >> '{}'\n{body}\n", calls.display()),
        )
        .unwrap();
        fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
        Self(dir)
    }

    fn start(&self) -> Result<SshTunnel, String> {
        self.start_to(&target("box"))
    }

    fn start_to(&self, target: &SshTarget) -> Result<SshTunnel, String> {
        SshTunnel::start(self.0.join("ssh").into(), target.clone(), &self.0)
    }

    fn calls(&self) -> Vec<String> {
        fs::read_to_string(self.0.join("calls"))
            .unwrap_or_default()
            .lines()
            .map(String::from)
            .collect()
    }
}

impl Drop for Stub {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn target(host: &str) -> SshTarget {
    SshTarget::parse(&format!("ssh://me@{host}")).unwrap()
}

/// Makes the `-L` socket, then waits like `ssh -N`.
const TUNNEL: &str = r#"while [ $# -gt 0 ]; do [ "$1" = -L ] && sock="${2%%:*}"; shift; done
touch "$sock"
exec sleep 30"#;

fn wait_until(what: &str, check: impl Fn() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !check() {
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn is_running(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

#[test]
fn starts_restarts_and_stops() {
    let stub = Stub::new("lifecycle", TUNNEL);
    let tunnel = stub.start().unwrap();
    let dir = tunnel.socket().parent().unwrap().to_path_buf();
    let mode = fs::metadata(&dir).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o700);
    assert!(stub.calls()[0].contains("-L"), "{:?}", stub.calls());

    let first = tunnel.pid().unwrap();
    Command::new("kill")
        .arg(first.to_string())
        .status()
        .unwrap();
    wait_until("the restart", || stub.calls().len() == 2);
    wait_until("the new ssh", || {
        tunnel.pid().is_some_and(|pid| pid != first)
    });
    let second = tunnel.pid().unwrap();

    drop(tunnel);
    assert!(!is_running(second));
    assert!(!Path::new(&dir).exists());
}

#[test]
fn failed_login_explains_keys() {
    let stub = Stub::new(
        "denied",
        "echo 'me@box: Permission denied (publickey,password).' >&2\nexit 255",
    );
    let error = stub.start().err().unwrap();
    assert!(error.contains("ssh-add"), "{error}");
}

#[test]
fn stale_start_stops_its_own_tunnel() {
    let stub = Stub::new("stale", TUNNEL);
    let slot = Mutex::new(Slot::new());
    let (old, new) = (target("old"), target("new"));
    let mut newer = None;
    // The start for `old` finishes after a start for `new` began and finished.
    let stale = open_in(&slot, &old, || {
        newer = Some(open_in(&slot, &new, || stub.start_to(&new)).unwrap());
        stub.start_to(&old)
    });
    assert!(stale.is_err());
    let slot = slot.lock().unwrap();
    let tunnel = slot.tunnel.as_ref().unwrap();
    assert_eq!(tunnel.target, new);
    assert_eq!(Some(tunnel.socket.clone()), newer);
    assert!(tunnel.socket.exists());
}

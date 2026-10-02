use std::io::{Read, Write};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use rustix::io::Errno;
use rustix::process::{Pid, test_kill_process};

use crate::pty::{Pty, ShellCommand, spawn};

const GRACE: Duration = Duration::from_millis(200);

fn shell(script: &str) -> Pty {
    let command = ShellCommand {
        program: "/bin/sh".into(),
        args: vec!["-c".into(), script.into()],
        cwd: std::env::temp_dir(),
        ..ShellCommand::default()
    };
    spawn(&command, 80, 24).expect("spawn")
}

/// Reads until the output contains `marker`, and returns the output.
fn read_until(pty: &mut Pty, marker: &str) -> String {
    let started = Instant::now();
    let mut output = Vec::new();
    let mut buf = [0u8; 256];
    while !String::from_utf8_lossy(&output).contains(marker) {
        assert!(started.elapsed() < Duration::from_secs(5), "no {marker}");
        match pty.reader.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => output.extend_from_slice(&buf[..n]),
        }
    }
    String::from_utf8_lossy(&output).into_owned()
}

/// Waits a few seconds for `pid` to go, as its new parent reaps it.
fn gone(pid: Pid) -> bool {
    let started = Instant::now();
    while started.elapsed() < Duration::from_secs(3) {
        if test_kill_process(pid) == Err(Errno::SRCH) {
            return true;
        }
        thread::sleep(Duration::from_millis(20));
    }
    false
}

#[test]
fn kills_a_foreground_job_that_outlives_the_shell() {
    // Job control puts the job in a group of its own, in the foreground. The job
    // ignores the hangup; the shell does not.
    let mut pty = shell("set -m; sh -c 'trap \"\" HUP; echo job:$$; exec sleep 30'");
    let output = read_until(&mut pty, "job:");
    let pid: i32 = output
        .split("job:")
        .nth(1)
        .and_then(|rest| rest.split_whitespace().next())
        .and_then(|pid| pid.parse().ok())
        .expect("job pid");
    let Pty { control, child, .. } = pty;
    thread::spawn(move || child.wait());
    control.terminate(GRACE);
    assert!(gone(Pid::from_raw(pid).expect("pid")), "the job survived");
}

#[test]
fn ends_a_write_that_waits_for_room() {
    // The shell neither reads its input nor ends on a hangup, so the input fills up.
    let mut pty = shell("trap '' HUP; stty -icanon -echo; echo ready; exec sleep 30");
    read_until(&mut pty, "ready");
    let Pty {
        mut writer,
        control,
        child,
        ..
    } = pty;
    thread::spawn(move || child.wait());
    let (done_tx, done_rx) = mpsc::channel();
    thread::spawn(move || {
        let chunk = [b'x'; 4096];
        while writer.write_all(&chunk).is_ok() {}
        done_tx.send(()).ok();
    });
    thread::sleep(Duration::from_millis(100));
    control.terminate(GRACE);
    assert!(
        done_rx.recv_timeout(Duration::from_secs(2)).is_ok(),
        "the write still waits"
    );
}

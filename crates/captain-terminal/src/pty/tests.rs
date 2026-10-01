use std::io::Read;
use std::time::{Duration, Instant};

use super::{ShellCommand, spawn};

#[test]
fn runs_a_command_and_reads_its_output() {
    let command = ShellCommand {
        program: "/bin/sh".into(),
        args: vec!["-c".into(), "echo hi$GREETING".into()],
        cwd: std::env::temp_dir(),
        env: vec![("GREETING".into(), "-there".into())],
        env_remove: Vec::new(),
    };
    let mut pty = spawn(&command, 80, 24).expect("spawn");
    let started = Instant::now();
    let mut output = Vec::new();
    let mut buf = [0u8; 256];
    while !String::from_utf8_lossy(&output).contains("hi-there") {
        assert!(started.elapsed() < Duration::from_secs(5), "no output");
        match pty.reader.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => output.extend_from_slice(&buf[..n]),
        }
    }
    assert!(String::from_utf8_lossy(&output).contains("hi-there"));
    assert_eq!(pty.child.wait(), Some(0));
}

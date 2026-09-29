use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use super::stream_lines;

#[test]
fn dropping_a_quiet_stream_kills_the_command() {
    let pid_file = std::env::temp_dir().join(format!("captain-quiet-{}", std::process::id()));
    std::fs::remove_file(&pid_file).ok();
    let mut command = Command::new("sh");
    command
        .arg("-c")
        .arg(format!("echo $$ > '{}'; exec sleep 30", pid_file.display()));
    let stream = stream_lines(command, "sh");
    let deadline = Instant::now() + Duration::from_secs(10);
    let pid = loop {
        match std::fs::read_to_string(&pid_file) {
            Ok(pid) if pid.ends_with('\n') => break pid,
            _ => assert!(Instant::now() < deadline, "the command did not start"),
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    std::fs::remove_file(&pid_file).ok();

    drop(stream);

    let alive = Command::new("kill")
        .args(["-0", pid.trim()])
        .stderr(Stdio::null())
        .status()
        .unwrap()
        .success();
    assert!(!alive, "sleep {} still runs", pid.trim());
}

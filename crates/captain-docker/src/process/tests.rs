use std::process::Command;
use std::time::Duration;

use super::output_within;

#[test]
fn a_command_past_its_timeout_is_killed() {
    let pid_file = std::env::temp_dir().join(format!("captain-timeout-{}", std::process::id()));
    let mut command = Command::new("sh");
    command
        .arg("-c")
        .arg(format!("echo $$ > '{}'; exec sleep 30", pid_file.display()));

    let error = output_within(command, Duration::from_millis(300)).unwrap_err();

    assert!(error.contains("did not answer"), "{error}");
    let pid = std::fs::read_to_string(&pid_file).unwrap();
    std::fs::remove_file(&pid_file).ok();
    let alive = Command::new("kill")
        .args(["-0", pid.trim()])
        .stderr(std::process::Stdio::null())
        .status()
        .unwrap()
        .success();
    assert!(!alive, "sleep {} still runs", pid.trim());
}

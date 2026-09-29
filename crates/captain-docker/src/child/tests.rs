use std::process::Command;
use std::time::Duration;

use super::output_guarded;

#[test]
fn dropping_the_future_kills_the_command() {
    let pid_file = std::env::temp_dir().join(format!("captain-guarded-{}", std::process::id()));
    let mut command = Command::new("sh");
    command
        .arg("-c")
        .arg(format!("echo $$ > '{}'; exec sleep 30", pid_file.display()));

    let running = output_guarded(command);
    while std::fs::read_to_string(&pid_file).map_or(true, |pid| !pid.ends_with('\n')) {
        std::thread::sleep(Duration::from_millis(10));
    }
    drop(running);

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

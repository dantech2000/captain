use std::process::{Command, Stdio};
use std::time::Duration;

use super::{kill, own_group};

#[test]
fn a_kill_reaches_the_processes_the_command_started() {
    let pid_file = std::env::temp_dir().join(format!("captain-group-{}", std::process::id()));
    let mut command = Command::new("sh");
    // The shell starts a grandchild, as `docker` starts `docker-compose`.
    command.arg("-c").arg(format!(
        "sleep 30 & echo $! > '{}'; wait",
        pid_file.display()
    ));
    let mut child = own_group(&mut command).spawn().unwrap();
    while std::fs::read_to_string(&pid_file).map_or(true, |pid| !pid.ends_with('\n')) {
        std::thread::sleep(Duration::from_millis(10));
    }
    kill(&mut child);
    child.wait().unwrap();

    let pid = std::fs::read_to_string(&pid_file).unwrap();
    std::fs::remove_file(&pid_file).ok();
    // The grandchild is reaped by init, so give it a moment.
    let mut alive = true;
    for _ in 0..100 {
        alive = Command::new("kill")
            .args(["-0", pid.trim()])
            .stderr(Stdio::null())
            .status()
            .unwrap()
            .success();
        if !alive {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(!alive, "sleep {} still runs", pid.trim());
}

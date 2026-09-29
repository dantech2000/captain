//! Kills a `docker` command together with the plugins it starts. `docker compose`
//! and `docker buildx` run as child processes of the CLI, so killing only the CLI
//! leaves them running. On Unix, the command gets a process group of its own and a
//! kill reaches the whole group. Windows has no process groups; there a kill stops
//! only the `docker` process. See
//! <https://doc.rust-lang.org/std/os/unix/process/trait.CommandExt.html#tymethod.process_group>.

use std::process::{Child, Command};

/// Starts `command` in a new process group, on Unix.
pub fn own_group(command: &mut Command) -> &mut Command {
    #[cfg(unix)]
    std::os::unix::process::CommandExt::process_group(command, 0);
    command
}

/// Kills `child` and, on Unix, every process in its group. Reaping stays with the
/// caller.
pub fn kill(child: &mut Child) {
    #[cfg(unix)]
    {
        use rustix::process::{Pid, Signal, kill_process_group};
        // Fails harmlessly when the child did not start its own group.
        kill_process_group(Pid::from_child(child), Signal::KILL).ok();
    }
    child.kill().ok();
}

#[cfg(all(test, unix))]
mod tests;

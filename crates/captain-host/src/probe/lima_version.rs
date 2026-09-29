use std::path::Path;
use std::process::Command;
use std::time::Duration;

use captain_core::diagnostics::ToolProbe;

use super::output_within;
use crate::lima::{current_exe, locate_limactl};

const TIMEOUT: Duration = Duration::from_secs(5);

/// The version that `limactl --version` prints, from the `limactl` that Captain
/// Engine would use.
pub fn lima_version() -> ToolProbe {
    let exe = current_exe();
    let path = std::env::var_os("PATH");
    let Some(binary) = locate_limactl(exe.as_deref(), path.as_deref(), Path::is_file) else {
        return ToolProbe::Missing;
    };
    let mut command = Command::new(&binary);
    command.arg("--version");
    match output_within(command, TIMEOUT) {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let version = stdout.split_whitespace().last().unwrap_or_default();
            ToolProbe::Found(version.to_string())
        }
        Ok(_) => ToolProbe::Broken(format!("{} --version failed.", binary.display())),
        Err(why) => ToolProbe::Broken(format!("Cannot run {}: {why}", binary.display())),
    }
}

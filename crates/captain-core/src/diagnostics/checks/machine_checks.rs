use crate::GIB;
use crate::diagnostics::{Check, CheckId, CheckState, Fix, Platform};
use crate::format::bytes_label;

/// Below this much free space, images and volumes soon fill the disk.
pub const LOW_DISK: u64 = 10 * GIB;
/// Lima's logs have grown without limit before (rancher-desktop#1942 had a 264 GB
/// log), so more than this is worth a look.
pub const LARGE_LOGS: u64 = 100 * 1024 * 1024;

/// Free space on the disk that holds the home folder.
pub fn disk_space(platform: Platform, free: Option<u64>) -> Check {
    let id = CheckId::DiskSpace;
    if platform.windows {
        return Check::new(
            id,
            CheckState::NotApplicable,
            "Captain does not check it on Windows yet.",
        );
    }
    match free {
        None => Check::new(
            id,
            CheckState::Warning,
            "Captain cannot read the free space.",
        ),
        Some(free) if free < LOW_DISK => Check::new(
            id,
            CheckState::Warning,
            format!(
                "Only {} free. Images and volumes need room; keep at least {}.",
                bytes_label(free),
                bytes_label(LOW_DISK)
            ),
        ),
        Some(free) => Check::new(
            id,
            CheckState::Passed,
            format!("{} free.", bytes_label(free)),
        ),
    }
}

/// The size of Lima's logs in Captain Engine's instance folder.
pub fn lima_logs(platform: Platform, captain: bool, bytes: Option<u64>) -> Check {
    let id = CheckId::LimaLogs;
    if !platform.macos || !captain {
        return Check::new(
            id,
            CheckState::NotApplicable,
            "Only Captain Engine on macOS uses Lima.",
        );
    }
    match bytes {
        None => Check::new(
            id,
            CheckState::NotApplicable,
            "Captain Engine is not set up.",
        ),
        Some(bytes) if bytes > LARGE_LOGS => Check::new(
            id,
            CheckState::Warning,
            format!(
                "The logs use {}. Stop Captain Engine, then delete the *.log files.",
                bytes_label(bytes)
            ),
        )
        .with_fix(Fix::ShowEngineFiles),
        Some(bytes) => Check::new(
            id,
            CheckState::Passed,
            format!("The logs use {}.", bytes_label(bytes)),
        ),
    }
}

/// Rosetta runs x86_64 images in Captain Engine on Apple silicon.
pub fn rosetta(platform: Platform, captain: bool, installed: Option<bool>) -> Check {
    let id = CheckId::Rosetta;
    if !platform.apple_silicon {
        return Check::new(
            id,
            CheckState::NotApplicable,
            "Only Apple silicon Macs use Rosetta.",
        );
    }
    if !captain {
        return Check::new(
            id,
            CheckState::NotApplicable,
            "Captain uses another engine.",
        );
    }
    match installed {
        Some(true) => Check::new(id, CheckState::Passed, "Rosetta is installed."),
        Some(false) => Check::new(
            id,
            CheckState::Warning,
            "Rosetta is not installed. x86_64 images run slowly, and the first start can \
             wait at \"Installing rosetta\" (lima#1202).",
        )
        .with_fix(Fix::CopyCommand(
            "softwareupdate --install-rosetta --agree-to-license",
        )),
        None => Check::new(
            id,
            CheckState::Warning,
            "Captain cannot tell if Rosetta is installed.",
        ),
    }
}

#[cfg(test)]
mod tests;

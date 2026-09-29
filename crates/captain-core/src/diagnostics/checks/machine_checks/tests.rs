use super::{LARGE_LOGS, LOW_DISK, disk_space, lima_logs, rosetta};
use crate::diagnostics::{CheckState, Fix, Platform};

const MAC: Platform = Platform {
    macos: true,
    windows: false,
    apple_silicon: true,
};

#[test]
fn low_disk_space_warns() {
    assert_eq!(disk_space(MAC, Some(LOW_DISK)).state, CheckState::Passed);
    assert_eq!(
        disk_space(MAC, Some(LOW_DISK - 1)).state,
        CheckState::Warning
    );
}

#[test]
fn large_lima_logs_warn_and_show_the_files() {
    assert_eq!(
        lima_logs(MAC, true, Some(LARGE_LOGS)).state,
        CheckState::Passed
    );
    let large = lima_logs(MAC, true, Some(LARGE_LOGS + 1));
    assert_eq!(large.state, CheckState::Warning);
    assert_eq!(large.fix, Some(Fix::ShowEngineFiles));
}

#[test]
fn rosetta_applies_only_to_apple_silicon() {
    let intel = Platform {
        apple_silicon: false,
        ..MAC
    };
    assert_eq!(rosetta(intel, true, None).state, CheckState::NotApplicable);
    assert_eq!(rosetta(MAC, true, Some(false)).state, CheckState::Warning);
}

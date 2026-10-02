use super::{compose, lima};
use crate::diagnostics::{CheckState, Fix, Platform, ToolProbe};

const MAC: Platform = Platform {
    macos: true,
    windows: false,
    apple_silicon: true,
};

fn found(version: &str) -> ToolProbe {
    ToolProbe::Found(version.into())
}

#[test]
fn lima_needs_2_2_0_on_macos_for_captain_engine_only() {
    assert_eq!(lima(MAC, true, &found("2.2.0")).state, CheckState::Passed);
    let old = lima(MAC, true, &found("2.1.9"));
    assert_eq!(old.state, CheckState::Failed);
    assert_eq!(old.fix, Some(Fix::CopyCommand("brew upgrade lima")));
    let linux = Platform {
        macos: false,
        ..MAC
    };
    let missing = ToolProbe::Missing;
    assert_eq!(lima(linux, true, &missing).state, CheckState::NotApplicable);
    assert_eq!(lima(MAC, false, &missing).state, CheckState::NotApplicable);
}

#[test]
fn compose_without_docker_names_the_cli() {
    let check = compose(&ToolProbe::Missing, &ToolProbe::Missing);
    assert_eq!(check.detail, "Compose needs the docker CLI.");
}

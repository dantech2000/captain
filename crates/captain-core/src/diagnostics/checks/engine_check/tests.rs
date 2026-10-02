use super::engine;
use crate::HostStatus;
use crate::diagnostics::{CheckState, EngineProbe, Fix};

fn no_answer() -> EngineProbe {
    EngineProbe::NoAnswer("connection refused".into())
}

#[test]
fn the_engine_check_follows_the_probe_and_the_host() {
    let probe = EngineProbe::Answered {
        version: "28.1.1".into(),
        api_version: "1.49".into(),
    };
    let check = engine(&probe, Some(&HostStatus::Running));
    assert_eq!(check.state, CheckState::Passed);
    assert_eq!(check.detail, "Docker 28.1.1, API 1.49.");
    // A running VM without a socket is the sleep issue.
    let check = engine(&no_answer(), Some(&HostStatus::Running));
    assert_eq!(check.state, CheckState::Failed);
    assert!(check.detail.contains("lima#5420"), "{}", check.detail);
    assert_eq!(check.fix, Some(Fix::RestartEngine));
    // A stopped Captain Engine offers Start while connecting.
    let check = engine(&EngineProbe::Connecting, Some(&HostStatus::Stopped));
    assert_eq!(check.state, CheckState::Failed);
    assert_eq!(check.fix, Some(Fix::StartEngine));
    // Another engine shows the reason without a fix.
    let check = engine(&no_answer(), None);
    assert_eq!(check.state, CheckState::Failed);
    assert!(check.detail.ends_with("connection refused"));
    assert_eq!(check.fix, None);
}

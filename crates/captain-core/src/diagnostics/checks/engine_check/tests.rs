use super::engine;
use crate::HostStatus;
use crate::diagnostics::{CheckState, EngineProbe, Fix};

fn no_answer() -> EngineProbe {
    EngineProbe::NoAnswer("connection refused".into())
}

#[test]
fn an_answer_passes_with_the_api_version() {
    let probe = EngineProbe::Answered {
        version: "28.1.1".into(),
        api_version: "1.49".into(),
    };
    let check = engine(&probe, Some(&HostStatus::Running));
    assert_eq!(check.state, CheckState::Passed);
    assert_eq!(check.detail, "Docker 28.1.1, API 1.49.");
}

#[test]
fn a_running_vm_without_a_socket_is_the_sleep_issue() {
    let check = engine(&no_answer(), Some(&HostStatus::Running));
    assert_eq!(check.state, CheckState::Failed);
    assert!(check.detail.contains("lima#5420"), "{}", check.detail);
    assert_eq!(check.fix, Some(Fix::RestartEngine));
}

#[test]
fn a_stopped_captain_engine_offers_start_while_connecting() {
    let check = engine(&EngineProbe::Connecting, Some(&HostStatus::Stopped));
    assert_eq!(check.state, CheckState::Failed);
    assert_eq!(check.fix, Some(Fix::StartEngine));
}

#[test]
fn another_engine_shows_the_reason_without_a_fix() {
    let check = engine(&no_answer(), None);
    assert_eq!(check.state, CheckState::Failed);
    assert!(check.detail.ends_with("connection refused"));
    assert_eq!(check.fix, None);
}

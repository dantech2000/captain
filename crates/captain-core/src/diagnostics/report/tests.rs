use super::{evaluate, failure_count};
use crate::diagnostics::{CheckState, EngineProbe, Facts, MachineFacts, Platform, ToolProbe};

#[test]
fn linux_with_another_engine_fails_nothing_that_does_not_apply() {
    let facts = Facts {
        platform: Platform {
            macos: false,
            windows: false,
            apple_silicon: false,
        },
        captain_engine: None,
        engine: EngineProbe::Answered {
            version: "28.1.1".into(),
            api_version: "1.49".into(),
        },
        machine: MachineFacts {
            lima: ToolProbe::Missing,
            docker: ToolProbe::Found("28.1.1".into()),
            compose: ToolProbe::Found("v2.35.1".into()),
            free_disk: Some(u64::MAX),
            lima_logs: None,
            rosetta: None,
        },
    };
    let checks = evaluate(&facts);
    assert_eq!(failure_count(&checks), 0);
    let skipped = checks
        .iter()
        .filter(|check| check.state == CheckState::NotApplicable)
        .count();
    assert_eq!(skipped, 3);
}

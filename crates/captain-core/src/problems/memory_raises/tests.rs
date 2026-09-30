use super::MemoryRaises;
use crate::model::ContainerDetail;

#[test]
fn a_raise_hides_the_fix_until_a_run_with_the_new_limit_runs_out() {
    const MB: u64 = 1024 * 1024;
    let mut killed = ContainerDetail {
        id: "worker".into(),
        started_at: "2026-09-30T14:00:00Z".into(),
        oom_killed: true,
        memory_limit: 64 * MB,
        ..Default::default()
    };
    let mut raises = MemoryRaises::default();
    assert_eq!(raises.oom_limit(&killed), Some(64 * MB));

    raises.record(&killed);
    killed.memory_limit = 512 * MB;
    assert_eq!(raises.oom_limit(&killed), None);

    killed.started_at = "2026-09-30T14:05:00Z".into();
    assert_eq!(raises.oom_limit(&killed), Some(512 * MB));
}

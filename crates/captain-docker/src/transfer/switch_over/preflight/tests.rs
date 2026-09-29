use bollard::models::{
    ContainerInspectResponse, ContainerSummary, ContainerSummaryStateEnum, HostConfig, MountPoint,
};

use super::{auto_remove, other_writers};

#[test]
fn refuses_a_container_started_with_rm() {
    let inspect = |auto_remove| ContainerInspectResponse {
        host_config: Some(HostConfig {
            auto_remove,
            ..HostConfig::default()
        }),
        ..ContainerInspectResponse::default()
    };
    assert!(auto_remove("db", &inspect(Some(false))).is_ok());
    let refused = auto_remove("db", &inspect(Some(true))).expect_err("refused");
    assert!(refused.to_string().contains("--rm"));
}

#[test]
fn refuses_a_running_writer_that_the_switch_over_does_not_stop() {
    let container = |name: &str, state, rw| ContainerSummary {
        names: Some(vec![format!("/{name}")]),
        state: Some(state),
        mounts: Some(vec![MountPoint {
            name: Some("pgdata".into()),
            rw: Some(rw),
            ..MountPoint::default()
        }]),
        ..ContainerSummary::default()
    };
    let running = ContainerSummaryStateEnum::RUNNING;
    let stop = ["db".to_string()];
    // The item's own container, a reader, and a stopped writer are fine.
    let fine = [
        container("db", running, true),
        container("backup", running, false),
        container("old", ContainerSummaryStateEnum::EXITED, true),
    ];
    assert!(other_writers(&["pgdata"], &stop, &fine).is_ok());
    let shared = [
        container("db", running, true),
        container("worker", running, true),
    ];
    let refused = other_writers(&["pgdata"], &stop, &shared).expect_err("refused");
    assert!(
        refused
            .to_string()
            .starts_with("engine error: worker also runs")
    );
}

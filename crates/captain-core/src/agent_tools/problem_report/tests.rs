use std::collections::HashMap;
use std::time::Instant;

use super::problem_report;
use crate::agent_tools::test_fleet::fleet;
use crate::model::{ContainerDetail, ContainerState, Health};
use crate::store::Crash;

#[test]
fn names_the_out_of_memory_kill_first_with_its_fix() {
    let mut containers = fleet(4);
    containers[0].health = Some(Health::Unhealthy);
    let worker = containers[3].id.clone();
    let detail = ContainerDetail {
        exit_code: 137,
        oom_killed: true,
        memory_limit: 256 << 20,
        restart_count: 5,
        ..ContainerDetail::default()
    };
    let details = HashMap::from([(worker.clone(), detail)]);
    // A crash loop spends most of its time running; the event stream tells.
    let crash = |id: &str| {
        (id == worker).then(|| Crash {
            at: Instant::now(),
            out_of_memory: false,
        })
    };
    containers[3].state = ContainerState::Running;
    let report = problem_report(None, &containers, &details, &crash);
    let kinds: Vec<&str> = report.problems.iter().map(|p| p.kind.as_str()).collect();
    assert_eq!(kinds, ["out_of_memory", "unhealthy"]);
    let oom = &report.problems[0];
    assert_eq!(oom.container, "project0-worker-1");
    assert_eq!(oom.exit_code, Some(137));
    assert_eq!(oom.fixes[0], "Raise its memory limit to 512 MB.");
    assert!(report.text().contains("out of memory at 256 MB"));
}

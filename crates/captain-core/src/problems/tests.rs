use std::collections::HashMap;

use super::{ExitFacts, Problem, first_problem, raised_memory};
use crate::diagnostics::{Check, CheckId, CheckState, Fix};
use crate::model::{Container, ContainerState, Health};

fn container(name: &str, state: ContainerState, health: Option<Health>) -> Container {
    Container {
        id: format!("{name}-id"),
        name: name.into(),
        image: "app:latest".into(),
        state,
        status: String::new(),
        ports: Vec::new(),
        created: 0,
        compose_project: None,
        health,
        kube_namespace: None,
        extension: None,
        compose: Default::default(),
    }
}

#[test]
fn the_worst_problem_wins_the_warning_card() {
    let checks = [
        Check::new(CheckId::Compose, CheckState::Failed, "missing"),
        Check::new(CheckId::Engine, CheckState::Failed, "down").with_fix(Fix::StartEngine),
    ];
    let unhealthy = container("api", ContainerState::Running, Some(Health::Unhealthy));
    let restarting = container("cache", ContainerState::Restarting, None);
    let oom = container("worker", ContainerState::Restarting, None);
    let facts = HashMap::from([(
        "worker-id".to_string(),
        ExitFacts {
            oom_killed: true,
            memory_limit: 256 << 20,
            restart_count: 3,
            raised: false,
        },
    )]);
    let all = [unhealthy.clone(), restarting.clone(), oom];

    let engine = first_problem(Some("vz failed"), &checks, &all, &facts, &|_| None);
    assert_eq!(
        engine,
        Some(Problem::EngineFailed {
            why: "vz failed".into(),
            fix: Fix::StartEngine
        })
    );
    let memory = first_problem(None, &checks, &all, &facts, &|_| None);
    assert_eq!(
        memory,
        Some(Problem::OutOfMemory {
            id: "worker-id".into(),
            name: "worker".into(),
            limit: 256 << 20,
            restarts: 3,
            raised: false
        })
    );
    let two = [unhealthy.clone(), restarting];
    let first = first_problem(None, &checks, &two, &facts, &|_| None);
    assert_eq!(
        first.and_then(|p| p.container().map(|(_, n)| n.to_string())),
        Some("cache".into())
    );
    let first = first_problem(None, &checks, &[unhealthy], &facts, &|_| None);
    assert!(matches!(first, Some(Problem::Unhealthy { .. })));
    let first = first_problem(None, &checks, &[], &facts, &|_| None);
    assert!(matches!(first, Some(Problem::FailedCheck(check)) if check.id == CheckId::Compose));
    assert_eq!(first_problem(None, &[], &[], &facts, &|_| None), None);
}

#[test]
fn an_out_of_memory_line_names_the_limit_and_the_restarts() {
    let problem = Problem::OutOfMemory {
        id: "db-id".into(),
        name: "db".into(),
        limit: 512 * 1024 * 1024,
        restarts: 3,
        raised: false,
    };
    assert_eq!(
        problem.line(),
        "db keeps restarting: out of memory at 512 MB, restarted 3 times."
    );
}

#[test]
fn raised_memory_doubles_the_limit_with_a_512_mb_floor() {
    const MIB: i64 = 1024 * 1024;
    assert_eq!(raised_memory(64 * MIB), 512 * MIB as u64);
    assert_eq!(raised_memory(1024 * MIB), 2048 * MIB as u64);
}

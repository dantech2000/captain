use super::ContainerFilter;
use crate::model::{Container, ContainerState};

fn container(state: ContainerState) -> Container {
    Container {
        id: "abc".into(),
        name: "c".into(),
        image: "i".into(),
        state,
        status: String::new(),
        ports: Vec::new(),
        created: 0,
        compose_project: None,
        health: None,
    }
}

#[test]
fn all_matches_everything() {
    assert!(ContainerFilter::All.matches(&container(ContainerState::Exited)));
    assert!(ContainerFilter::All.matches(&container(ContainerState::Running)));
}

#[test]
fn running_matches_active_states() {
    assert!(ContainerFilter::Running.matches(&container(ContainerState::Paused)));
    assert!(!ContainerFilter::Running.matches(&container(ContainerState::Exited)));
}

#[test]
fn stopped_matches_inactive_states() {
    assert!(ContainerFilter::Stopped.matches(&container(ContainerState::Exited)));
    assert!(ContainerFilter::Stopped.matches(&container(ContainerState::Created)));
    assert!(!ContainerFilter::Stopped.matches(&container(ContainerState::Running)));
}

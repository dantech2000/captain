use super::ContainerStore;
use crate::model::{Container, ContainerState};

fn container(name: &str, state: ContainerState) -> Container {
    Container {
        id: format!("{name}-0123456789abcdef"),
        name: name.into(),
        image: "nginx:latest".into(),
        state,
        status: String::new(),
        ports: Vec::new(),
        created: 0,
        compose_project: None,
    }
}

#[test]
fn new_store_is_empty() {
    let store = ContainerStore::default();
    assert!(store.is_empty());
    assert_eq!(store.len(), 0);
    assert!(store.get(0).is_none());
}

#[test]
fn replace_sorts_active_first_then_by_name() {
    let mut store = ContainerStore::default();
    store.replace(vec![
        container("zeta", ContainerState::Exited),
        container("beta", ContainerState::Running),
        container("alpha", ContainerState::Exited),
        container("gamma", ContainerState::Paused),
    ]);

    let names: Vec<&str> = store.containers().iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["beta", "gamma", "alpha", "zeta"]);
}

#[test]
fn replace_drops_old_containers() {
    let mut store = ContainerStore::default();
    store.replace(vec![container("old", ContainerState::Running)]);
    store.replace(vec![container("new", ContainerState::Running)]);

    assert_eq!(store.len(), 1);
    assert_eq!(store.get(0).map(|c| c.name.as_str()), Some("new"));
}

#[test]
fn active_count_counts_running_paused_and_restarting() {
    let mut store = ContainerStore::default();
    store.replace(vec![
        container("a", ContainerState::Running),
        container("b", ContainerState::Paused),
        container("c", ContainerState::Restarting),
        container("d", ContainerState::Exited),
        container("e", ContainerState::Created),
    ]);

    assert_eq!(store.active_count(), 3);
}

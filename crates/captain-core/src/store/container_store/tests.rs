use super::ContainerStore;
use crate::model::{Container, ContainerState};
use crate::store::ContainerFilter;

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
        compose: Default::default(),
        health: None,
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

fn in_project(name: &str, project: Option<&str>, state: ContainerState) -> Container {
    Container {
        compose_project: project.map(Into::into),
        compose: Default::default(),
        ..container(name, state)
    }
}

#[test]
fn groups_put_projects_first_and_standalone_last() {
    let mut store = ContainerStore::default();
    store.replace(vec![
        in_project("solo", None, ContainerState::Running),
        in_project("web", Some("shop"), ContainerState::Running),
        in_project("post", Some("blog"), ContainerState::Exited),
        in_project("db", Some("shop"), ContainerState::Running),
    ]);

    let groups = store.groups(ContainerFilter::All);
    let names: Vec<Option<&str>> = groups.iter().map(|g| g.project.as_deref()).collect();
    assert_eq!(names, [Some("blog"), Some("shop"), None]);
    assert_eq!(groups[1].containers.len(), 2);
    assert_eq!(groups[1].running_count(), 2);
}

#[test]
fn groups_apply_the_filter_and_drop_empty_groups() {
    let mut store = ContainerStore::default();
    store.replace(vec![
        in_project("web", Some("shop"), ContainerState::Running),
        in_project("post", Some("blog"), ContainerState::Exited),
    ]);

    let groups = store.groups(ContainerFilter::Running);
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].project.as_deref(), Some("shop"));
}

#[test]
fn projects_skip_standalone_containers() {
    let mut store = ContainerStore::default();
    store.replace(vec![
        in_project("web", Some("shop"), ContainerState::Running),
        in_project("solo", None, ContainerState::Running),
    ]);

    assert_eq!(store.projects().len(), 1);
    assert!(store.find(&store.containers()[0].id).is_some());
}

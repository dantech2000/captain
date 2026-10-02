use super::ContainerStore;
use crate::model::{Container, ContainerState};
use crate::store::{ContainerFilter, GroupKey};

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
        kube_namespace: None,
        extension: None,
    }
}

#[test]
fn replace_sorts_active_first_then_by_name_and_counts_the_active() {
    let mut store = ContainerStore::default();
    store.replace(vec![
        container("zeta", ContainerState::Exited),
        container("beta", ContainerState::Running),
        container("alpha", ContainerState::Exited),
        container("gamma", ContainerState::Paused),
    ]);

    let names: Vec<&str> = store.containers().iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["beta", "gamma", "alpha", "zeta"]);

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
fn groups_put_projects_first_standalone_last_and_follow_the_filter() {
    let mut store = ContainerStore::default();
    store.replace(vec![
        in_project("solo", None, ContainerState::Running),
        in_project("web", Some("shop"), ContainerState::Running),
        in_project("post", Some("blog"), ContainerState::Exited),
        in_project("db", Some("shop"), ContainerState::Running),
    ]);

    let groups = store.groups(ContainerFilter::All, false);
    let names: Vec<Option<&str>> = groups.iter().map(|g| g.project()).collect();
    assert_eq!(names, [Some("blog"), Some("shop"), None]);
    assert_eq!(groups[1].containers.len(), 2);
    assert_eq!(groups[1].running_count(), 2);
    // Projects skip standalone containers.
    assert_eq!(store.projects().len(), 2);

    // A filter drops groups it empties.
    let groups = store.groups(ContainerFilter::Running, false);
    let names: Vec<Option<&str>> = groups.iter().map(|g| g.project()).collect();
    assert_eq!(names, [Some("shop"), None]);
}

fn in_namespace(name: &str, namespace: &str) -> Container {
    Container {
        kube_namespace: Some(namespace.into()),
        extension: None,
        ..container(name, ContainerState::Running)
    }
}

#[test]
fn kubernetes_containers_hide_or_group_by_namespace() {
    let mut store = ContainerStore::default();
    store.replace(vec![
        in_namespace("k8s_coredns", "kube-system"),
        in_project("web", Some("shop"), ContainerState::Running),
        in_namespace("k8s_app", "default"),
        in_project("solo", None, ContainerState::Running),
        in_namespace("k8s_traefik", "kube-system"),
    ]);

    let hidden = store.groups(ContainerFilter::All, false);
    let keys: Vec<&GroupKey> = hidden.iter().map(|g| &g.key).collect();
    assert_eq!(
        keys,
        [&GroupKey::Project("shop".into()), &GroupKey::Standalone]
    );
    assert_eq!(store.shown(false).count(), 2);
    assert_eq!(store.kubernetes_count(), 3);

    let shown = store.groups(ContainerFilter::All, true);
    let keys: Vec<&GroupKey> = shown.iter().map(|g| &g.key).collect();
    assert_eq!(
        keys,
        [
            &GroupKey::Project("shop".into()),
            &GroupKey::Namespace("default".into()),
            &GroupKey::Namespace("kube-system".into()),
            &GroupKey::Standalone,
        ]
    );
    assert_eq!(shown[2].containers.len(), 2);
}

#[test]
fn extension_backends_hide_until_shown() {
    let mut backend = in_project(
        "portainer",
        Some("captain-ext-acme"),
        ContainerState::Running,
    );
    backend.extension = Some("acme".into());
    let mut store = ContainerStore::default();
    store.replace(vec![backend, container("web", ContainerState::Running)]);
    let names = |store: &ContainerStore| -> Vec<String> {
        store.containers().iter().map(|c| c.name.clone()).collect()
    };
    assert_eq!(names(&store), ["web"]);
    assert_eq!((store.active_count(), store.projects().len()), (1, 0));
    assert!(store.is_hidden("portainer-0123456789abcdef", None));
    assert!(store.is_hidden("new", Some("captain-ext-acme-backend-1")));
    // Explicit navigation still finds a hidden backend for the inspector.
    let id = "portainer-0123456789abcdef";
    assert!(store.find(id).is_none() && store.find_any(id).is_some());
    store.set_show_extensions(true);
    assert_eq!(names(&store), ["portainer", "web"]);
    assert!(!store.is_hidden("portainer-0123456789abcdef", None));
}

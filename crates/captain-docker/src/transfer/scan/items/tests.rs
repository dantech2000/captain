use std::collections::HashMap;
use std::path::Path;

use bollard::models::{ContainerSummary, ContainerSummaryStateEnum, MountPoint, Network};
use captain_core::migration::MigrationItem;

use super::{containers, networks, volumes};

fn network(name: &str, driver: &str, scope: &str) -> Network {
    Network {
        name: Some(name.into()),
        driver: Some(driver.into()),
        scope: Some(scope.into()),
        ..Network::default()
    }
}

fn summary(id: &str, name: &str, size_rw: i64, labels: &[(&str, &str)]) -> ContainerSummary {
    ContainerSummary {
        id: Some(id.into()),
        names: Some(vec![format!("/{name}")]),
        image: Some("app:1".into()),
        size_rw: Some(size_rw),
        labels: Some(
            labels
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect::<HashMap<_, _>>(),
        ),
        ..ContainerSummary::default()
    }
}

#[test]
fn keeps_user_networks_only() {
    let items = networks(vec![
        network("bridge", "bridge", "local"),
        network("host", "host", "local"),
        network("none", "null", "local"),
        network("backend", "bridge", "local"),
        network("mesh", "overlay", "swarm"),
    ]);
    assert_eq!(
        items,
        [MigrationItem::Network {
            name: "backend".into(),
            driver: "bridge".into()
        }]
    );
}

#[test]
fn groups_projects_and_skips_helpers() {
    let project = [
        ("com.docker.compose.project", "shop"),
        ("com.docker.compose.service", "db"),
        ("com.docker.compose.project.working_dir", "/src/shop"),
        ("com.docker.compose.project.config_files", "compose.yaml"),
    ];
    let items = containers(
        vec![
            summary("a", "shop-db-1", 42, &project),
            summary("b", "web", 0, &[]),
            summary("c", "captain-migrate-read-1", 0, &[]),
        ],
        |path: &Path| path.starts_with("/src/shop"),
    );
    assert_eq!(items.len(), 2);
    match &items[0] {
        MigrationItem::ComposeProject {
            name,
            files_exist,
            containers,
            changed,
            ..
        } => {
            assert_eq!(name, "shop");
            assert!(files_exist);
            assert_eq!(containers, &["shop-db-1"]);
            assert_eq!(changed, &["shop-db-1"]);
        }
        other => panic!("expected a project, got {other:?}"),
    }
    assert!(
        matches!(&items[1], MigrationItem::Container { name, size_rw: 0, .. } if name == "web")
    );
}

#[test]
fn projects_list_running_services_and_volumes() {
    fn labels(service: &str) -> [(&str, &str); 2] {
        [
            ("com.docker.compose.project", "stokecrm"),
            ("com.docker.compose.service", service),
        ]
    }
    let member = |id: &str, service: &str, state, volume: Option<&str>| ContainerSummary {
        state: Some(state),
        mounts: volume.map(|name| {
            vec![
                MountPoint {
                    typ: Some("volume".into()),
                    name: Some(name.into()),
                    ..Default::default()
                },
                MountPoint {
                    typ: Some("bind".into()),
                    name: None,
                    ..Default::default()
                },
            ]
        }),
        ..summary(id, &format!("stokecrm-{service}"), 0, &labels(service))
    };
    let items = containers(
        vec![
            member(
                "a",
                "postgres",
                ContainerSummaryStateEnum::RUNNING,
                Some("stokecrm-pgdata"),
            ),
            // A profile service that ran once and exited.
            member("b", "backup", ContainerSummaryStateEnum::EXITED, None),
        ],
        |_| false,
    );
    match &items[0] {
        MigrationItem::ComposeProject {
            running_services,
            running_containers,
            volumes,
            ..
        } => {
            assert_eq!(running_services, &["postgres"]);
            assert_eq!(running_containers, &["stokecrm-postgres"]);
            assert_eq!(volumes, &["stokecrm-pgdata"]);
        }
        other => panic!("expected a project, got {other:?}"),
    }
    assert!(items[0].can_switch_over());
}

#[test]
fn project_files_must_exist() {
    let labels = [
        ("com.docker.compose.project", "shop"),
        ("com.docker.compose.project.working_dir", "/gone"),
        ("com.docker.compose.project.config_files", "compose.yaml"),
    ];
    let items = containers(vec![summary("a", "shop-web-1", 0, &labels)], |_| false);
    assert!(matches!(
        &items[0],
        MigrationItem::ComposeProject {
            files_exist: false,
            ..
        }
    ));
}

#[test]
fn volumes_list_the_running_containers_that_use_them() {
    let with_mount = |name: &str, state, volume: &str| ContainerSummary {
        names: Some(vec![format!("/{name}")]),
        state: Some(state),
        mounts: Some(vec![MountPoint {
            name: Some(volume.to_string()),
            ..Default::default()
        }]),
        ..Default::default()
    };
    let containers = [
        with_mount("db", ContainerSummaryStateEnum::RUNNING, "pgdata"),
        with_mount("old-db", ContainerSummaryStateEnum::EXITED, "pgdata"),
        with_mount("cache", ContainerSummaryStateEnum::RUNNING, "redis"),
    ];
    let volume = |name: &str| captain_core::model::Volume {
        name: name.into(),
        ..Default::default()
    };
    let items = volumes(vec![volume("pgdata"), volume("empty")], &containers);

    let users: Vec<Vec<String>> = items
        .iter()
        .map(|item| match item {
            MigrationItem::Volume {
                used_by_running, ..
            } => used_by_running.clone(),
            _ => unreachable!(),
        })
        .collect();
    assert_eq!(users, [vec!["db".to_string()], Vec::<String>::new()]);
}

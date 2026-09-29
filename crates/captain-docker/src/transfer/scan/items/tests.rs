use std::collections::HashMap;
use std::path::Path;

use bollard::models::{ContainerSummary, Network};
use captain_core::migration::MigrationItem;

use super::{containers, networks};

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

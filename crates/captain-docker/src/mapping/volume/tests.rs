use std::collections::HashMap;

use bollard::models::{
    ContainerSummary, ContainerSummaryStateEnum, MountPoint, SystemDataUsageResponse,
    Volume as DockerVolume, VolumePruneResponse, VolumeUsageData, VolumesDiskUsage,
};
use captain_core::model::ContainerState;
use serde_json::json;

use super::{volume, volume_prune, volume_prune_filters, volume_usage, volume_users};

fn docker_volume(name: &str) -> DockerVolume {
    DockerVolume {
        name: name.into(),
        driver: "local".into(),
        mountpoint: format!("/var/lib/docker/volumes/{name}/_data"),
        created_at: Some("2026-09-28T10:00:00Z".into()),
        labels: HashMap::from([("com.docker.compose.project".into(), "shop".into())]),
        ..DockerVolume::default()
    }
}

#[test]
fn reads_usage_from_df_items() {
    let response = SystemDataUsageResponse {
        volume_usage: Some(VolumesDiskUsage {
            items: Some(vec![
                json!({"Name": "pgdata", "Driver": "local", "Mountpoint": "/x",
                       "Labels": {}, "Options": {}, "Scope": "local",
                       "UsageData": {"Size": 2048, "RefCount": 1}}),
                json!({"not": "a volume"}),
            ]),
            ..VolumesDiskUsage::default()
        }),
        ..SystemDataUsageResponse::default()
    };
    let usage = volume_usage(response);
    assert_eq!(usage.len(), 1);
    assert_eq!(usage["pgdata"].size, 2048);
    assert_eq!(usage["pgdata"].ref_count, 1);
}

#[test]
fn maps_fields_and_usage() {
    let usage = HashMap::from([(
        "pgdata".to_string(),
        VolumeUsageData {
            size: 4096,
            ref_count: 2,
        },
    )]);
    let v = volume(docker_volume("pgdata"), &usage);
    assert_eq!(v.name, "pgdata");
    assert_eq!(v.driver, "local");
    assert_eq!(v.created_date(), "2026-09-28");
    assert_eq!(v.compose_project.as_deref(), Some("shop"));
    assert_eq!(v.size_bytes, Some(4096));
    assert_eq!(v.containers, Some(2));
}

#[test]
fn unknown_usage_stays_unknown() {
    let v = volume(docker_volume("other"), &HashMap::new());
    assert_eq!(v.size_bytes, None);
    assert_eq!(v.containers, None);

    let usage = HashMap::from([(
        "other".to_string(),
        VolumeUsageData {
            size: -1,
            ref_count: -1,
        },
    )]);
    let v = volume(docker_volume("other"), &usage);
    assert_eq!(v.size_bytes, None);
    assert_eq!(v.containers, None);
}

fn summary(name: &str, mounts: Vec<MountPoint>) -> ContainerSummary {
    ContainerSummary {
        id: Some(format!("id-{name}")),
        names: Some(vec![format!("/{name}")]),
        state: Some(ContainerSummaryStateEnum::RUNNING),
        mounts: Some(mounts),
        ..ContainerSummary::default()
    }
}

fn mount(name: Option<&str>, destination: &str, rw: bool) -> MountPoint {
    MountPoint {
        name: name.map(Into::into),
        destination: Some(destination.into()),
        rw: Some(rw),
        ..MountPoint::default()
    }
}

#[test]
fn volume_users_pick_the_named_mount_and_sort_by_name() {
    let containers = vec![
        summary("web", vec![mount(Some("pgdata"), "/backup", false)]),
        summary(
            "db",
            vec![
                mount(None, "/etc/app", true),
                mount(Some("pgdata"), "/var/lib/postgresql/data", true),
            ],
        ),
        // Matched by destination path only, not by volume name.
        summary("other", vec![mount(None, "/pgdata", true)]),
    ];
    let users = volume_users(containers, "pgdata");
    assert_eq!(users.len(), 2);
    assert_eq!(users[0].name, "db");
    assert_eq!(users[0].container_id, "id-db");
    assert_eq!(users[0].state, ContainerState::Running);
    assert_eq!(users[0].destination, "/var/lib/postgresql/data");
    assert!(!users[0].read_only);
    assert_eq!(users[1].name, "web");
    assert!(users[1].read_only);
}

#[test]
fn prune_filters_add_all_and_label_only_when_asked() {
    assert!(volume_prune_filters(false, None).is_empty());
    let filters = volume_prune_filters(true, Some("captain-agent-test=1"));
    assert_eq!(filters["all"], ["true"]);
    assert_eq!(filters["label"], ["captain-agent-test=1"]);
}

#[test]
fn prune_response_maps_names_and_space() {
    let report = volume_prune(VolumePruneResponse {
        volumes_deleted: Some(vec!["a".into(), "b".into()]),
        space_reclaimed: Some(2048),
    });
    assert_eq!(report.removed, ["a", "b"]);
    assert_eq!(report.reclaimed_bytes, 2048);
    assert_eq!(
        volume_prune(VolumePruneResponse::default()).reclaimed_bytes,
        0
    );
}

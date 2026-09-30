use super::*;
use crate::model::{BuildCacheRecord, DiskContainer, Volume};

const NOW: i64 = 1_790_000_000;
const DAY: i64 = 24 * 60 * 60;

fn image(id: &str, tags: &[&str], containers: usize) -> Image {
    Image {
        id: id.into(),
        repo_tags: tags.iter().map(|t| t.to_string()).collect(),
        size: 100,
        created: NOW - 40 * DAY,
        containers,
        dangling: tags.is_empty(),
    }
}

fn container(name: &str, image: &str, state: ContainerState, age_days: i64) -> DiskContainer {
    DiskContainer {
        id: name.into(),
        name: name.into(),
        image_id: image.into(),
        state,
        status: String::new(),
        created: NOW - age_days * DAY,
        size_rw: 10,
        volumes: vec![format!("{name}_data")],
    }
}

fn volume(name: &str, containers: Option<usize>) -> Volume {
    Volume {
        name: name.into(),
        size_bytes: Some(50),
        containers,
        ..Volume::default()
    }
}

fn cache(id: &str, age_days: i64, in_use: bool, kind: &str) -> BuildCacheRecord {
    BuildCacheRecord {
        id: id.into(),
        kind: kind.into(),
        size: 30,
        in_use,
        last_used: Some(NOW - age_days * DAY),
        ..BuildCacheRecord::default()
    }
}

fn names(plan: &ReclaimPlan, group: ReclaimGroup) -> Vec<String> {
    plan.group(group).map(|item| item.name.clone()).collect()
}

#[test]
fn each_group_selects_only_what_no_container_uses() {
    let usage = DiskUsage {
        images: vec![
            // The engine's count says 0, but a stopped container uses it.
            image("sha256:api", &["api:dev"], 0),
            image("sha256:web", &["web:1"], 2),
            image("sha256:old", &["postgres:16", "postgres:latest"], 0),
            image("sha256:aaaabbbbcccc1", &[], 0),
            image("sha256:used-dangling", &[], 1),
        ],
        containers: vec![
            container("api", "sha256:api", ContainerState::Exited, 10),
            container("web", "sha256:web", ContainerState::Running, 10),
            container("fresh", "sha256:web", ContainerState::Exited, 1),
        ],
        volumes: vec![
            // The count is stale, but the container list says `api` mounts it.
            volume("api_data", Some(0)),
            volume("old_mysql_data", Some(0)),
            volume("unknown", None),
        ],
        build_cache: vec![
            cache("old", 20, false, "regular"),
            cache("old-in-use", 20, true, "regular"),
            cache("old-internal", 20, false, "internal"),
            cache("recent", 2, false, "regular"),
        ],
        ..DiskUsage::default()
    };
    let plan = ReclaimPlan::new(&usage, NOW);

    assert_eq!(names(&plan, ReclaimGroup::OldBuildCache), ["old"]);
    assert_eq!(
        names(&plan, ReclaimGroup::DanglingImages),
        ["<none> aaaabbbbcccc"]
    );
    assert_eq!(
        names(&plan, ReclaimGroup::UnusedImages),
        ["postgres:16 (2 tags)"]
    );
    assert_eq!(names(&plan, ReclaimGroup::OldStoppedContainers), ["api"]);
    assert_eq!(
        names(&plan, ReclaimGroup::UnusedVolumes),
        ["old_mysql_data"]
    );
}

use futures::executor::block_on;

use super::*;
use crate::model::{BuildCacheRecord, DiskUsage, Image};
use crate::storage::ReclaimGroup;
use crate::{FakeEngine, FakeImages};

fn daemon(id: &str) -> EngineInfo {
    EngineInfo {
        id: id.into(),
        endpoint: "unix:///captain/docker.sock".into(),
        ..EngineInfo::default()
    }
}

fn image(id: &str, tags: &[&str], containers: usize) -> Image {
    Image {
        id: id.into(),
        repo_tags: tags.iter().map(|t| t.to_string()).collect(),
        size: 100,
        containers,
        ..Image::default()
    }
}

fn record(id: &str, size: u64) -> BuildCacheRecord {
    BuildCacheRecord {
        id: id.into(),
        size,
        ..BuildCacheRecord::default()
    }
}

fn item(group: ReclaimGroup, name: &str, size: u64, target: ReclaimTarget) -> ReclaimItem {
    ReclaimItem {
        group,
        name: name.into(),
        size,
        time: 0,
        target,
    }
}

fn engine() -> FakeEngine {
    FakeEngine {
        info: Some(daemon("A")),
        images: FakeImages {
            // `app:latest` moved from `sha256:old` to `sha256:new` since the preview,
            // and a container started to use `sha256:used`.
            images: vec![
                image("sha256:old", &[], 0),
                image("sha256:new", &["app:latest"], 1),
                image("sha256:used", &["db:1"], 1),
            ],
            ..FakeImages::default()
        },
        disk: DiskUsage {
            build_cache: vec![record("previewed", 30), record("newlyold", 70)],
            ..DiskUsage::default()
        },
        ..FakeEngine::default()
    }
}

fn run(engine: FakeEngine, items: Vec<ReclaimItem>) -> Result<CleanupReport, String> {
    block_on(run_cleanup(Arc::new(engine), &daemon("A"), items))
}

#[test]
fn a_different_daemon_removes_nothing() {
    let mut other = engine();
    other.info = Some(daemon("B"));
    let target = ReclaimTarget::Volume {
        name: "data".into(),
    };
    let items = vec![item(ReclaimGroup::UnusedVolumes, "data", 50, target)];
    assert!(run(other, items).unwrap_err().contains("different engine"));
}

#[test]
fn images_go_by_previewed_id_and_only_while_unused() {
    let items = vec![
        item(
            ReclaimGroup::UnusedImages,
            "app:latest",
            100,
            ReclaimTarget::Image {
                id: "sha256:old".into(),
            },
        ),
        item(
            ReclaimGroup::UnusedImages,
            "db:1",
            100,
            ReclaimTarget::Image {
                id: "sha256:used".into(),
            },
        ),
    ];
    let report = run(engine(), items).unwrap();
    assert_eq!(report.freed_bytes, 100);
    assert_eq!(report.failures, ["db:1: a container uses it now"]);
}

#[test]
fn build_cache_prunes_only_previewed_records() {
    let target = ReclaimTarget::BuildCache {
        id: "previewed".into(),
    };
    let items = vec![item(ReclaimGroup::OldBuildCache, "RUN make", 30, target)];
    assert_eq!(run(engine(), items).unwrap().freed_bytes, 30);
}

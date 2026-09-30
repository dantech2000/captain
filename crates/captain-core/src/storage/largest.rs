use super::{ReclaimGroup, ReclaimPlan, ReclaimTarget};
use crate::model::{DiskUsage, Image};
use crate::snapshot::Snapshot;

/// The icon of a row in the "Largest first" list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    Image,
    Volume,
    BuildCache,
    Snapshot,
    Container,
}

/// Who uses an item, for the middle column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemUse {
    /// The names of the containers, running or stopped, that use it.
    UsedBy(Vec<String>),
    /// Untagged images that no container uses.
    Dangling,
    /// An image that no container uses. The engine does not say when one last did.
    Unused,
    /// Build cache that no build used since this Unix time.
    NotUsedSince(i64),
    /// A volume that no container uses. It may hold data.
    UnusedVolume,
    /// A container that runs, with the engine's status, for example "Up 3 hours".
    Running(String),
    /// A stopped container, with the engine's status, for example "Exited (0) 5 days ago".
    Stopped(String),
    /// Build cache that a build used in the last 14 days, or that one uses now.
    RecentBuildCache,
    Snapshot,
}

/// One row of the "Largest first" list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LargeItem {
    pub kind: ItemKind,
    pub name: String,
    pub used: ItemUse,
    pub size: u64,
    /// True if a cleanup group removes it.
    pub reclaimable: bool,
}

/// The `limit` largest items: each image, volume, container, and snapshot, with the
/// dangling images as one row and the build cache as one row for old records and
/// one for the rest.
pub fn largest_items(
    usage: &DiskUsage,
    plan: &ReclaimPlan,
    snapshots: &[Snapshot],
    limit: usize,
) -> Vec<LargeItem> {
    let mut items = Vec::new();
    let (dangling, images): (Vec<&Image>, Vec<&Image>) = usage
        .images
        .iter()
        .partition(|image| image.dangling && !image.in_use() && users(usage, image).is_empty());
    if !dangling.is_empty() {
        items.push(LargeItem {
            kind: ItemKind::Image,
            name: format!("<none> × {} dangling", dangling.len()),
            used: ItemUse::Dangling,
            size: dangling.iter().map(|image| image.size).sum(),
            reclaimable: true,
        });
    }
    for image in images {
        let names = users(usage, image);
        let unused = names.is_empty() && !image.in_use();
        items.push(LargeItem {
            kind: ItemKind::Image,
            name: image_name(image),
            used: if unused {
                ItemUse::Unused
            } else {
                ItemUse::UsedBy(names)
            },
            size: image.size,
            reclaimable: unused,
        });
    }
    for volume in &usage.volumes {
        let names: Vec<String> = usage
            .containers
            .iter()
            .filter(|c| c.volumes.contains(&volume.name))
            .map(|c| c.name.clone())
            .collect();
        let unused = plan.group(ReclaimGroup::UnusedVolumes).any(
            |item| matches!(&item.target, ReclaimTarget::Volume { name } if *name == volume.name),
        );
        items.push(LargeItem {
            kind: ItemKind::Volume,
            name: volume.display_name().to_string(),
            used: if unused {
                ItemUse::UnusedVolume
            } else {
                ItemUse::UsedBy(names)
            },
            size: volume.size_bytes.unwrap_or(0),
            reclaimable: unused,
        });
    }
    items.extend(build_cache_rows(usage, plan));
    for container in &usage.containers {
        let old = plan
            .group(ReclaimGroup::OldStoppedContainers)
            .any(|item| item.name == container.name);
        items.push(LargeItem {
            kind: ItemKind::Container,
            name: container.name.clone(),
            used: if container.state.is_active() {
                ItemUse::Running(container.status.clone())
            } else {
                ItemUse::Stopped(container.status.clone())
            },
            size: container.size_rw,
            reclaimable: old,
        });
    }
    items.extend(snapshots.iter().map(|snapshot| LargeItem {
        kind: ItemKind::Snapshot,
        name: snapshot.metadata.name.clone(),
        used: ItemUse::Snapshot,
        size: snapshot.metadata.disk_allocated,
        reclaimable: false,
    }));
    items.retain(|item| item.size > 0);
    items.sort_by_key(|item| std::cmp::Reverse(item.size));
    items.truncate(limit);
    items
}

/// `node:22`, or `node:22 (3 tags)` for an image with more tags.
pub(super) fn image_name(image: &Image) -> String {
    match image.repo_tags.len() {
        0 => format!("<none> {}", image.short_id()),
        1 => image.display_name(),
        n => format!("{} ({n} tags)", image.display_name()),
    }
}

/// The names of the containers that use `image`.
fn users(usage: &DiskUsage, image: &Image) -> Vec<String> {
    usage
        .containers
        .iter()
        .filter(|c| c.image_id == image.id)
        .map(|c| c.name.clone())
        .collect()
}

/// One row for the build cache that the cleanup removes, dated by its newest use,
/// and one for the rest.
fn build_cache_rows(usage: &DiskUsage, plan: &ReclaimPlan) -> Vec<LargeItem> {
    let old = plan.group_bytes(ReclaimGroup::OldBuildCache);
    let mut rows = Vec::new();
    if let Some(since) = plan
        .group(ReclaimGroup::OldBuildCache)
        .map(|i| i.time)
        .max()
    {
        rows.push(LargeItem {
            kind: ItemKind::BuildCache,
            name: "Build cache, older than 14 days".into(),
            used: ItemUse::NotUsedSince(since),
            size: old,
            reclaimable: true,
        });
    }
    rows.push(LargeItem {
        kind: ItemKind::BuildCache,
        name: "Build cache, recent".into(),
        used: ItemUse::RecentBuildCache,
        size: usage.build_cache_bytes.saturating_sub(old),
        reclaimable: false,
    });
    rows
}

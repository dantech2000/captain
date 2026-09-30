use std::collections::HashSet;

use super::largest::image_name;
use super::{BUILD_CACHE_AGE, ReclaimGroup, STOPPED_AGE};
use crate::model::{ContainerState, DiskUsage, Image, parse_rfc3339};

/// What removes one item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReclaimTarget {
    /// One build prune with the age filter removes every old record at once.
    BuildCache,
    /// Each tag goes in turn, and the last one removes the image. An untagged image
    /// goes by ID. The engine refuses a tag that a container still needs.
    Image {
        id: String,
        tags: Vec<String>,
    },
    /// Each stopped container goes by ID, so only the previewed ones go.
    Container {
        id: String,
    },
    Volume {
        name: String,
    },
}

/// One item that a cleanup group removes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReclaimItem {
    pub group: ReclaimGroup,
    pub name: String,
    pub size: u64,
    /// Unix seconds: when a build last used it for build cache, else when it was made.
    pub time: i64,
    pub target: ReclaimTarget,
}

/// Every item each cleanup group would remove, largest first within a group. Items
/// that a container uses, running or stopped, are never in it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReclaimPlan {
    items: Vec<ReclaimItem>,
}

impl ReclaimPlan {
    /// The plan for `usage` at `now`, in Unix seconds.
    pub fn new(usage: &DiskUsage, now: i64) -> Self {
        let used_images: HashSet<&str> = usage
            .containers
            .iter()
            .map(|c| c.image_id.as_str())
            .collect();
        let used_volumes: HashSet<&str> = usage
            .containers
            .iter()
            .flat_map(|c| c.volumes.iter().map(String::as_str))
            .collect();
        let cache_cutoff = now - BUILD_CACHE_AGE.as_secs() as i64;
        let stopped_cutoff = now - STOPPED_AGE.as_secs() as i64;

        let mut items = Vec::new();
        let unused_images = usage
            .images
            .iter()
            .filter(|i| !i.in_use() && !used_images.contains(i.id.as_str()));
        for image in unused_images {
            let group = if image.dangling {
                ReclaimGroup::DanglingImages
            } else {
                ReclaimGroup::UnusedImages
            };
            items.push(image_item(group, image));
        }
        let old_cache = usage.build_cache.iter().filter(|r| {
            !r.in_use && !r.is_internal() && r.last_active() > 0 && r.last_active() < cache_cutoff
        });
        items.extend(old_cache.map(|record| ReclaimItem {
            group: ReclaimGroup::OldBuildCache,
            name: if record.description.is_empty() {
                record.id.chars().take(12).collect()
            } else {
                record.description.clone()
            },
            size: record.size,
            time: record.last_active(),
            target: ReclaimTarget::BuildCache,
        }));
        let old_stopped = usage.containers.iter().filter(|c| {
            !c.state.is_active()
                && c.state != ContainerState::Removing
                && c.created < stopped_cutoff
        });
        items.extend(old_stopped.map(|container| ReclaimItem {
            group: ReclaimGroup::OldStoppedContainers,
            name: container.name.clone(),
            size: container.size_rw,
            time: container.created,
            target: ReclaimTarget::Container {
                id: container.id.clone(),
            },
        }));
        // An unknown container count counts as used.
        let unused_volumes = usage
            .volumes
            .iter()
            .filter(|v| v.is_unused() && !used_volumes.contains(v.name.as_str()));
        items.extend(unused_volumes.map(|volume| ReclaimItem {
            group: ReclaimGroup::UnusedVolumes,
            name: volume.display_name().to_string(),
            size: volume.size_bytes.unwrap_or(0),
            time: parse_rfc3339(&volume.created).unwrap_or(0),
            target: ReclaimTarget::Volume {
                name: volume.name.clone(),
            },
        }));
        items.sort_by(|a, b| a.group.cmp(&b.group).then(b.size.cmp(&a.size)));
        Self { items }
    }

    /// The items of one group, largest first.
    pub fn group(&self, group: ReclaimGroup) -> impl Iterator<Item = &ReclaimItem> {
        self.items.iter().filter(move |item| item.group == group)
    }

    pub fn group_bytes(&self, group: ReclaimGroup) -> u64 {
        self.group(group).map(|item| item.size).sum()
    }

    /// The items of the checked groups, in panel order.
    pub fn selected<'a>(
        &'a self,
        checked: &'a [ReclaimGroup],
    ) -> impl Iterator<Item = &'a ReclaimItem> {
        self.items
            .iter()
            .filter(move |item| checked.contains(&item.group))
    }

    /// The bytes the checked groups free.
    pub fn total_bytes(&self, checked: &[ReclaimGroup]) -> u64 {
        self.selected(checked).map(|item| item.size).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

fn image_item(group: ReclaimGroup, image: &Image) -> ReclaimItem {
    ReclaimItem {
        group,
        name: image_name(image),
        size: image.size,
        time: image.created,
        target: ReclaimTarget::Image {
            id: image.id.clone(),
            tags: image.repo_tags.clone(),
        },
    }
}

#[cfg(test)]
mod tests;

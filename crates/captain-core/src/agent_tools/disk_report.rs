//! The `disk_usage` answer: the Storage page's categories, its largest items, and
//! what each cleanup group would free. It only reads; agents cannot clean up.

use schemars::JsonSchema;
use serde::Serialize;

use super::untrusted::plain;
use crate::format::{age_label, bytes_label};
use crate::model::{DiskUsage, kube_display_name};
use crate::snapshot::Snapshot;
use crate::storage::{DiskBreakdown, ItemKind, ItemUse, ReclaimGroup, ReclaimPlan, largest_items};

/// How many of the largest items the answer lists.
const LARGEST: usize = 10;

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct DiskReport {
    /// For example `18.2 GB`.
    pub used: String,
    pub used_bytes: u64,
    /// The size of Captain Engine's disk; absent for other engines.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity: Option<String>,
    pub categories: Vec<DiskCategory>,
    /// The largest images, volumes, containers, build cache, and snapshots.
    pub largest: Vec<DiskItem>,
    /// Captain's cleanup groups. Nothing that a container uses is in any group.
    pub cleanup: Vec<CleanupGroup>,
    /// What Storage > Clean up frees with its default choices (all but volumes).
    pub default_cleanup: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct DiskCategory {
    pub name: String,
    pub size: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct DiskItem {
    /// image, volume, build_cache, container, or snapshot.
    pub kind: String,
    pub name: String,
    pub size: String,
    /// Who uses it, or why it is unused.
    pub used: String,
    /// True if a cleanup group removes it.
    pub reclaimable: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct CleanupGroup {
    pub group: String,
    pub items: usize,
    pub frees: String,
    /// True for volumes: removing them loses data.
    pub has_data: bool,
}

/// The report for `usage` at `now` (Unix seconds). `snapshots` and `capacity` are
/// for Captain Engine; other engines pass `None`.
pub fn disk_report(
    usage: &DiskUsage,
    snapshots: Option<&[Snapshot]>,
    capacity: Option<u64>,
    now: i64,
) -> DiskReport {
    let snapshot_bytes = snapshots.map(|list| list.iter().map(|s| s.metadata.disk_allocated).sum());
    let breakdown = DiskBreakdown::new(usage, snapshot_bytes, capacity);
    let plan = ReclaimPlan::new(usage, now);
    let largest = largest_items(usage, &plan, snapshots.unwrap_or_default(), LARGEST);
    let default_groups: Vec<ReclaimGroup> = ReclaimGroup::ALL
        .into_iter()
        .filter(|group| group.checked_by_default())
        .collect();
    DiskReport {
        used: bytes_label(breakdown.used),
        used_bytes: breakdown.used,
        capacity: breakdown.capacity.map(bytes_label),
        categories: breakdown
            .categories
            .iter()
            .map(|share| DiskCategory {
                name: share.category.label().into(),
                size: bytes_label(share.bytes),
                bytes: share.bytes,
            })
            .collect(),
        largest: largest
            .into_iter()
            .map(|item| DiskItem {
                kind: kind_name(item.kind).into(),
                name: item.name,
                size: bytes_label(item.size),
                used: used_label(&item.used, now),
                reclaimable: item.reclaimable,
            })
            .collect(),
        cleanup: ReclaimGroup::ALL
            .into_iter()
            .map(|group| CleanupGroup {
                group: group.label().into(),
                items: plan.group(group).count(),
                frees: bytes_label(plan.group_bytes(group)),
                has_data: group.has_data(),
            })
            .collect(),
        default_cleanup: bytes_label(plan.total_bytes(&default_groups)),
    }
}

fn kind_name(kind: ItemKind) -> &'static str {
    match kind {
        ItemKind::Image => "image",
        ItemKind::Volume => "volume",
        ItemKind::BuildCache => "build_cache",
        ItemKind::Snapshot => "snapshot",
        ItemKind::Container => "container",
    }
}

fn used_label(used: &ItemUse, now: i64) -> String {
    match used {
        ItemUse::UsedBy(names) if names.is_empty() => "in use".into(),
        ItemUse::UsedBy(names) => {
            // Kubernetes names are long; `pod/container` is what lists show.
            let shown: Vec<String> = names
                .iter()
                .map(|name| kube_display_name(name).unwrap_or_else(|| name.clone()))
                .collect();
            format!("used by {}", shown.join(", "))
        }
        ItemUse::Dangling => "dangling: untagged and unused".into(),
        ItemUse::Unused => "no container uses it".into(),
        ItemUse::NotUsedSince(time) => format!("last used {}", age_label(*time, now)),
        ItemUse::UnusedVolume => "no container uses it; it may hold data".into(),
        ItemUse::Running(status) | ItemUse::Stopped(status) => status.clone(),
        ItemUse::RecentBuildCache => "used by a build in the last 14 days".into(),
        ItemUse::Snapshot => "snapshot".into(),
    }
}

impl DiskReport {
    /// The report as lines, for clients that show only text.
    pub fn text(&self) -> String {
        let mut lines = vec![match &self.capacity {
            Some(capacity) => format!("{} used of {capacity}.", self.used),
            None => format!("{} used.", self.used),
        }];
        let categories: Vec<String> = self
            .categories
            .iter()
            .map(|c| format!("{} {}", c.name, c.size))
            .collect();
        lines.push(categories.join(" · "));
        lines.push("Largest:".into());
        for item in &self.largest {
            lines.push(plain(&format!(
                "  {} {} ({}) · {}",
                item.kind, item.name, item.size, item.used
            )));
        }
        lines.push(format!(
            "Clean up with the default choices frees {}.",
            self.default_cleanup
        ));
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests;

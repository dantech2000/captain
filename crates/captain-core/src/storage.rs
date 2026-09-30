//! Disk use and cleanup for the Storage page: the categories of the disk bar, the
//! largest items and who uses them, and what each cleanup group removes. See
//! docs/features/0031-storage.md.

mod breakdown;
mod cleanup;
mod largest;
mod reclaim_group;
mod reclaim_plan;

pub use breakdown::{Category, CategoryShare, DiskBreakdown};
pub use cleanup::{CleanupReport, run_cleanup};
pub use largest::{ItemKind, ItemUse, LargeItem, largest_items};
pub use reclaim_group::{BUILD_CACHE_AGE, ReclaimGroup, STOPPED_AGE};
pub use reclaim_plan::{ReclaimItem, ReclaimPlan, ReclaimTarget};

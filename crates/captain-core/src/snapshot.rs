//! Snapshots of Captain Engine: named copies of the stopped VM's files. This module
//! has the metadata, the name and space rules, and the trait the host implements.
//! See docs/adr/0012-snapshots.md and docs/features/0023-snapshots.md.

mod metadata;
mod name;
mod space;
mod store;

pub use metadata::{COMPLETE_FILE, METADATA_FILE, Snapshot, SnapshotMetadata, find, newest_first};
pub use name::{MAX_NAME_CHARS, check_name};
pub use space::{STEP_SPACE, check_space};
pub use store::{EngineSnapshots, SnapshotList};

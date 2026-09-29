//! The Snapshots page: save, restore, and delete named copies of Captain Engine.
//! The files are handled by `captain-host`. See docs/features/0023-snapshots.md.

mod create_dialog;
mod delete_dialog;
mod restore_dialog;
mod snapshot_event;
mod snapshot_row;
mod snapshot_steps;
mod snapshots_model;
mod snapshots_view;

pub use snapshot_event::SnapshotEvent;
pub use snapshots_model::SnapshotsModel;
pub use snapshots_view::SnapshotsView;

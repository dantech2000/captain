//! The Storage page: what fills the engine's disk, who uses each item, and a cleanup
//! with a preview. The model also feeds the status bar's Disk segment and runs the
//! weekly build-cache cleanup. See docs/features/0031-storage.md.

mod cleanup_steps;
mod disk_header;
mod largest_list;
mod preview_dialog;
mod reclaim_panel;
mod storage_event;
mod storage_model;
mod storage_view;
mod weekly;

pub use disk_header::category_color;
pub use storage_event::StorageEvent;
pub use storage_model::{StorageModel, init, storage_model};
pub use storage_view::StorageView;

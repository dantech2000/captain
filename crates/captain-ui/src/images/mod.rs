//! The Images page. [`ImagesState`] holds the image list, the selected image's
//! details, and runs remove, prune, pull, run, tag, push, and build and scan dialogs. It follows engine events on its own,
//! so `workspace` stays about containers.

mod actions;
mod build_dialog;
mod column_header;
mod detail_loading;
mod empty_state;
mod field;
mod header;
mod image_row;
mod images_state;
mod images_view;
mod inspector;
mod loading;
mod pull_status;
mod push;
mod push_dialog;
mod run_dialog;
mod scan_dialog;
mod started;
mod started_notice;
mod tag_dialog;
mod toolbar;
mod toolbar_button;

pub(crate) use field::field;
pub use images_state::ImagesState;
pub use images_view::ImagesView;
pub use started::Started;

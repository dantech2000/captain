//! The Images page. [`ImagesState`] holds the image list, the selected image's
//! details, and runs remove, prune, pull, and run. It follows engine events on its own,
//! so `workspace` stays about containers.

mod actions;
mod column_header;
mod detail_loading;
mod empty_state;
mod header;
mod image_row;
mod images_state;
mod images_view;
mod inspector;
mod loading;
mod pull_status;
mod run_dialog;
mod started;
mod started_notice;
mod toolbar;
mod toolbar_button;

pub use images_state::ImagesState;
pub use images_view::ImagesView;
pub use started::Started;

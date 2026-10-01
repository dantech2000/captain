//! The New sheet and the projects Captain remembers: Open a folder now; Run an
//! image, templates, and Paste a docker run command next. See
//! docs/features/0040-new-projects.md.

mod keys;
mod known_model;
mod new_sheet;
mod open_folder;
mod options;
mod remove_dialog;
mod sheet_view;

pub use keys::{NewProject, init};
pub use known_model::{known_model, known_projects};
pub use new_sheet::open;
pub use remove_dialog::open as open_remove_dialog;

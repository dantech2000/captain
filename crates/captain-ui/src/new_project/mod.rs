//! The New sheet and the projects Captain remembers: Run an image, Start from a
//! template, Open a folder, and Paste a docker run command. See
//! docs/features/0040-new-projects.md.

mod finish;
mod form;
mod host;
mod hub;
mod keys;
mod known_model;
mod name_check;
mod new_sheet;
mod open_folder;
mod options;
mod paste;
mod remove_dialog;
mod run_image;
mod sheet_view;
mod template;

pub use host::{host, set_host};
pub use hub::set_docker_hub;
pub use keys::{NewProject, init};
pub use known_model::{known_model, known_projects};
pub use new_sheet::{open, open_run_image};
pub use remove_dialog::open as open_remove_dialog;

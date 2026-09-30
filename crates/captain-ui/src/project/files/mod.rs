//! The Files tab of the Project page: the project's Compose files and Dockerfiles
//! in an editor with checks, and Save and apply with a preview. See
//! docs/features/0039-compose-and-dockerfile-editor.md.

mod apply;
mod checks;
mod editor_panel;
mod file_editor;
mod files_page;
mod files_state;
mod grammar;
mod preview_dialog;
mod problem_list;
mod providers;

pub use file_editor::FileEditor;
pub use files_page::render;
pub use files_state::FilesState;

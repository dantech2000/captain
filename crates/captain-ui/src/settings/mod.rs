//! The Settings page, and the settings global that the rest of the UI reads.

mod about_section;
mod admin_access;
mod agents_section;
mod appearance_section;
mod cli_tools_state;
mod context_actions;
mod context_rows;
mod disk_dialog;
mod endpoint_picker;
mod engine_menu;
mod engine_resources;
mod engine_section;
mod engine_sheet;
mod engine_source;
mod file_section;
mod file_watch;
mod kube_dialogs;
mod kube_form;
mod kubernetes_section;
mod listen;
mod open_file;
mod options_entries;
mod options_sheet;
mod page_section;
mod path_step;
mod reset_dialog;
mod settings_view;
mod startup_section;
mod store;
mod system;
mod terminal_section;
mod terminal_sheet;
mod terminal_steps;
mod theme_card;

pub use engine_source::{
    ContextJob, DetectedEndpoint, EngineSource, engine_source, init_engine_source, reconnect_to,
    retry,
};
pub use file_watch::{file_problem, file_watch, init as init_file_watch, toast_file_problems};
pub use open_file::open_settings_file;
pub use settings_view::SettingsView;
pub use store::{
    SettingsStore, apply_appearance, current, init, observe, settings_file_path, theme_family,
    update,
};
pub use system::{SystemIntegration, init as init_system};

//! The Settings page, and the settings global that the rest of the UI reads.

mod about_section;
mod admin_access_section;
mod appearance_section;
mod behavior_section;
mod captain_engine_section;
mod context_actions;
mod context_rows;
mod daemon_form;
mod daemon_section;
mod endpoint_picker;
mod engine_resources;
mod engine_section;
mod engine_source;
mod kube_dialogs;
mod kube_form;
mod kubernetes_section;
mod reset_dialog;
mod settings_view;
mod store;
mod system;
mod theme_card;

pub use engine_source::{
    ContextJob, DetectedEndpoint, EngineSource, engine_source, init_engine_source, reconnect_to,
    retry,
};
pub use settings_view::SettingsView;
pub use store::{SettingsStore, apply_appearance, current, init, observe, theme_family, update};
pub use system::{SystemIntegration, init as init_system};

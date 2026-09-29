//! The Settings page, and the settings global that the rest of the UI reads.

mod about_section;
mod appearance_section;
mod captain_engine_section;
mod endpoint_picker;
mod engine_resources;
mod engine_section;
mod engine_source;
mod reset_dialog;
mod settings_view;
mod store;

pub use engine_source::{
    DetectedEndpoint, EngineSource, engine_source, init_engine_source, reconnect, retry,
};
pub use settings_view::SettingsView;
pub use store::{accent, apply_appearance, current, init, update};

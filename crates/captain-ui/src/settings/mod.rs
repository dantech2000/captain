//! The Settings page, and the settings global that the rest of the UI reads.

mod about_section;
mod appearance_section;
mod endpoint_picker;
mod engine_section;
mod engine_source;
mod settings_view;
mod store;

pub use engine_source::{DetectedEndpoint, EngineSource, init_engine_source, retry};
pub use settings_view::SettingsView;
pub use store::{accent, apply_appearance, init};

//! User settings: appearance, accent color, the engine choice, the engine
//! endpoint override, and app behavior.
//! The app picks where the file lives. See ADR 0004.

mod accent;
mod app_settings;
mod appearance;
mod engine_choice;
mod storage;

pub use accent::Accent;
pub use app_settings::{SETTINGS_VERSION, Settings};
pub use appearance::Appearance;
pub use engine_choice::EngineChoice;
pub use storage::SettingsError;

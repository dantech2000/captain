//! User settings: appearance, accent color, and the engine endpoint override.
//! The app picks where the file lives. See ADR 0004.

mod accent;
mod app_settings;
mod appearance;
mod storage;

pub use accent::Accent;
pub use app_settings::{SETTINGS_VERSION, Settings};
pub use appearance::Appearance;
pub use storage::SettingsError;

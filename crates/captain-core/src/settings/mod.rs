//! User settings: appearance, color theme, the engine choice, the engine
//! endpoint override, and app behavior.
//! The app picks where the file lives. See ADR 0004 and ADR 0013.

mod app_settings;
mod appearance;
mod edit;
mod engine_choice;
mod jsonc;
mod overrides;
mod problem;
mod reference;
mod reference_markdown;
mod schema;
mod storage;
mod theme_family;
mod validate;

pub use app_settings::{SETTINGS_VERSION, Settings};
pub use appearance::Appearance;
pub use engine_choice::EngineChoice;
pub use problem::{FileProblem, REFERENCE_URL, doc_link};
pub use reference::{GROUPS, ReferenceEntry, reference_entries};
pub use reference_markdown::reference_markdown;
pub use schema::{SCHEMA_FILE, settings_schema, settings_schema_text, write_schema};
pub use storage::{LoadedSettings, SettingsError};
pub use theme_family::ThemeFamily;

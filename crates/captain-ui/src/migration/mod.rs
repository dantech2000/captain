//! The Migration Assistant: copies data from another engine into the connected one,
//! through the Docker API, without changing the old engine. See
//! docs/adr/0009-migration.md and docs/features/0014-migration-assistant.md.

mod assistant;
mod backend;
mod choose_step;
mod connect;
mod open;
mod plan_row;
mod review_step;
mod run_row;
mod run_step;
mod runner;
mod summary_step;
mod view;

pub use assistant::MigrationAssistant;
pub use open::open;

gpui_kit::actions!(captain, [OpenMigrationAssistant]);

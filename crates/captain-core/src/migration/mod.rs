//! The Migration Assistant's model: what the source engine holds, what the user
//! picked, in what order it copies, and how each step went. See
//! docs/adr/0009-migration.md.

mod backend;
mod disk;
mod estimate;
mod item;
mod plan;
mod run;
mod status;
mod step;
mod target;

pub use backend::{MigrationBackend, MigrationSession, SourceOption, TransferEvent};
pub use disk::{DiskCheck, DiskVerdict};
pub use estimate::{DEFAULT_THROUGHPUT, Estimate, duration_label};
pub use item::MigrationItem;
pub use plan::{ImageChoice, MigrationPlan, PlanEntry};
pub use run::{MigrationRun, RunEntry, RunSummary};
pub use status::StepStatus;
pub use step::Step;
pub use target::is_captain_engine;

/// The prefix of every helper container and snapshot image the assistant makes.
pub const HELPER_PREFIX: &str = "captain-migrate";

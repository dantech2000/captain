//! Diagnostics: checks that find common problems with the engine and this computer.
//! The checks are pure functions of [`Facts`]; the app gathers the facts. See
//! docs/features/0016-diagnostics.md.

mod check;
mod checks;
mod facts;
mod report;
mod version;

pub use check::{Check, CheckId, CheckState, Fix};
pub use facts::{EngineProbe, Facts, MachineFacts, Platform, ToolProbe};
pub use report::{evaluate, failure_count};
pub use version::{MINIMUM_LIMA_VERSION, Version, parse_version};

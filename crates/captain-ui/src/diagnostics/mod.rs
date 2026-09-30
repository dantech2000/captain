//! The Diagnostics page: checks for common problems, and troubleshooting actions.
//! The checks are pure logic in `captain_core::diagnostics`. See
//! docs/features/0016-diagnostics.md.

mod check_row;
mod diagnostics_model;
mod diagnostics_view;
mod engine_probe;
mod fix;
mod troubleshooting;

pub use diagnostics_model::{
    DiagnosticsModel, DiagnosticsSetup, diagnostics_model, failures, init,
};
pub use diagnostics_view::DiagnosticsView;
pub use fix::run_suggested as run_suggested_fix;

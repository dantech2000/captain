//! One function per check. Each reads only the facts it needs.

mod engine_check;
mod machine_checks;
mod settings_check;
mod tool_checks;

pub use engine_check::engine;
pub use machine_checks::{disk_space, lima_logs, rosetta};
pub use settings_check::settings_file;
pub use tool_checks::{compose, docker_cli, lima};
